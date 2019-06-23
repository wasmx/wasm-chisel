use std::collections::HashSet;

use parity_wasm::elements::{FuncBody, Instruction, Module};

/// A function dependency graph is represented as a list of "edges", or pairs of function indices
/// (a, b) where a calls b.

/// An edge, where the function at the left index calls the function at the right
/// index.
#[derive(PartialEq, Eq, Hash)]
pub struct Edge(u32, u32);

/// Container struct for the function dependency graph.
pub struct DepGraph {
    edges: HashSet<Edge>,
}

/// Private interface for managing the function dependency graph
pub trait DepGraphManager {
    /// Recursive graph builder. Requires import section length in order to resolve the correct
    /// function body.
    fn probe(&mut self, idx: u32, imports_len: u32, bodies: &[FuncBody]);
    fn add_edge(&mut self, dep: Edge) -> bool;
}

/// Public interface for building function dependency graphs.
pub trait DepGraphBuilder: DepGraphManager {
    /// Builds the dependency graph.
    fn build(module: &Module, entry_idx: u32) -> Result<Self, ()>
    where
        Self: std::marker::Sized;
}

impl DepGraph {
    pub fn new() -> Self {
        DepGraph {
            edges: HashSet::new(),
        }
    }

    #[cfg(test)]
    pub fn edgecount(&self) -> usize {
        self.edges.len()
    }
    // TODO: better access methods
}

impl DepGraphManager for DepGraph {
    /// Recursively searches function bodies for calls to other functions and adds edges
    /// accordingly.
    fn probe(&mut self, idx: u32, imports_len: u32, bodies: &[FuncBody]) {
        // If the function is an import, then just backtrack.
        if idx < imports_len {
            return;
        }

        // Overflow case handled by the previous early return condition.
        let code_idx: usize = (idx - imports_len) as usize;

        assert!((code_idx) < bodies.len());
        let func_body = &bodies[code_idx];

        for instr in func_body.code().elements().iter() {
            if let Instruction::Call(call_idx) = instr {
                if self.add_edge(Edge::from((idx, *call_idx))) {
                    self.probe(*call_idx, imports_len, bodies);
                } else {
                    // If the edge already exists then begin backtracking.
                    return;
                }
            }
            // TODO: Support for call_indirect
        }
    }

    /// Simply inserts an edge into the graph. Returns false if it was duplicate.
    fn add_edge(&mut self, dep: Edge) -> bool {
        self.edges.insert(dep)
    }
}

impl DepGraphBuilder for DepGraph {
    fn build(module: &Module, entry_idx: u32) -> Result<Self, ()> {
        if let Some(code_section) = module.code_section() {
            let mut ret = DepGraph::new();

            let imports_len: u32 = if let Some(section) = module.import_section() {
                section.entries().len() as u32
            } else {
                0
            };

            ret.probe(entry_idx, imports_len, &code_section.bodies());

            Ok(ret)
        } else {
            Err(())
        }
    }
}

impl From<(u32, u32)> for Edge {
    fn from(tuple: (u32, u32)) -> Self {
        Edge(tuple.0, tuple.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_wasm::elements::deserialize_buffer;

    #[test]
    fn one_dep_main() {
        // wast:
        // (module
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main
        //     (call $otherfunc)
        //   )
        //   (func $otherfunc)
        // )

        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x03, 0x02, 0x00, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x11, 0x02, 0x04,
            0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02,
            0x00, 0x0a, 0x09, 0x02, 0x04, 0x00, 0x10, 0x01, 0x0b, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let g = DepGraph::build(&module, 0).unwrap();

        assert!(g.edgecount() == 1);
    }

    #[test]
    fn dep_chain2_main() {
        // (module
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main
        //     (call $otherfunc)
        //   )
        //   (func $otherfunc
        //      (call $otherfunc1)
        //   )
        //   (func $otherfunc1)
        // )

        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x04, 0x03, 0x00, 0x00, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x11, 0x02,
            0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79,
            0x02, 0x00, 0x0a, 0x0e, 0x03, 0x04, 0x00, 0x10, 0x01, 0x0b, 0x04, 0x00, 0x10, 0x02,
            0x0b, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let g = DepGraph::build(&module, 0).unwrap();

        assert!(g.edgecount() == 2);
    }

    #[test]
    fn mutual_recursion() {
        // wast:
        // (module
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main
        //     (call $otherfunc)
        //   )
        //   (func $otherfunc
        //     (call $main)
        //   )
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x03, 0x02, 0x00, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x11, 0x02, 0x04,
            0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02,
            0x00, 0x0a, 0x0b, 0x02, 0x04, 0x00, 0x10, 0x01, 0x0b, 0x04, 0x00, 0x10, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let g = DepGraph::build(&module, 0).unwrap();

        assert!(g.edgecount() == 2);
    }

    #[test]
    fn main_calls_self_recursion() {
        // wast:
        //   (module
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main
        //     (call $main)
        //   )
        // )

        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x11, 0x02, 0x04, 0x6d,
            0x61, 0x69, 0x6e, 0x00, 0x00, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00,
            0x0a, 0x06, 0x01, 0x04, 0x00, 0x10, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let g = DepGraph::build(&module, 0).unwrap();

        assert!(g.edgecount() == 1);
    }

    #[test]
    fn arbitrary_graph() {
        // wast:
        // (module
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main
        //     (call $main_child1)
        //     (call $main_child2)
        //   )
        //
        //   (func $main_child1
        //     (call $child1_child1)
        //     (call $child1_child2)
        //     (call $child1_child3)
        //   )
        //
        //   (func $main_child2)
        //
        //   (func $child1_child1
        //     (call $main_child1)
        //   )
        //
        //   (func $child1_child2
        //     (call $child1_child2)
        //   )
        //
        //   (func $child1_child3)
        // )

        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x07, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01,
            0x07, 0x11, 0x02, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, 0x06, 0x6d, 0x65, 0x6d,
            0x6f, 0x72, 0x79, 0x02, 0x00, 0x0a, 0x21, 0x06, 0x06, 0x00, 0x10, 0x01, 0x10, 0x02,
            0x0b, 0x08, 0x00, 0x10, 0x03, 0x10, 0x04, 0x10, 0x05, 0x0b, 0x02, 0x00, 0x0b, 0x04,
            0x00, 0x10, 0x01, 0x0b, 0x04, 0x00, 0x10, 0x04, 0x0b, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let g = DepGraph::build(&module, 0).unwrap();

        assert!(g.edgecount() == 7);
    }

    #[test]
    fn arbitary_graph_2() {
        // wast:
        // (module
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main
        //     (call $main_child1)
        //     (call $main_child2)
        //   )
        //
        //   (func $main_child1
        //     (call $child1_child1)
        //     (call $child1_child2)
        //     (call $child1_child3)
        //   )
        //
        //   (func $main_child2
        //     (call $child1_child2)
        //     (call $child1_child3)
        //   )
        //
        //   (func $child1_child1
        //     (call $main_child1)
        //   )
        //
        //   (func $child1_child2
        //     (call $child1_child2)
        //   )
        //
        //   (func $child1_child3
        //     (call $main_child2)
        //   )
        // )

        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x07, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01,
            0x07, 0x11, 0x02, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, 0x06, 0x6d, 0x65, 0x6d,
            0x6f, 0x72, 0x79, 0x02, 0x00, 0x0a, 0x27, 0x06, 0x06, 0x00, 0x10, 0x01, 0x10, 0x02,
            0x0b, 0x08, 0x00, 0x10, 0x03, 0x10, 0x04, 0x10, 0x05, 0x0b, 0x06, 0x00, 0x10, 0x04,
            0x10, 0x05, 0x0b, 0x04, 0x00, 0x10, 0x01, 0x0b, 0x04, 0x00, 0x10, 0x04, 0x0b, 0x04,
            0x00, 0x10, 0x02, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let g = DepGraph::build(&module, 0).unwrap();

        assert!(g.edgecount() == 10);
    }

    #[test]
    fn arbitrary_graph_with_imports() {
        // wast:
        // (module
        //   (import "ethereum" "useGas" (func $useGas (param i64)))
        //   (import "ethereum" "getBlockGasLimit" (func $getBlockGasLimit (result i64)))
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //
        //   (func $main
        //     (call $main_child1)
        //     (call $main_child2)
        //   )
        //
        //   (func $main_child1
        //     (call $child1_child1)
        //     (call $child1_child2)
        //     (call $child1_child3)
        //   )
        //
        //   (func $main_child2
        //     (call $child1_child2)
        //     (i64.store (i32.const 0) (call $getBlockGasLimit))
        //     (call $child1_child3)
        //   )
        //
        //   (func $child1_child1
        //     (call $useGas (i64.const 420))
        //     (i64.store (i32.const 0) (call $getBlockGasLimit))
        //     (call $main_child1)
        //   )
        //
        //   (func $child1_child2
        //     (call $useGas (i64.const 1337))
        //     (call $child1_child2)
        //   )
        //
        //   (func $child1_child3
        //     (i64.store (i32.const 0) (call $getBlockGasLimit))
        //     (call $main_child2)
        //   )
        // )

        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x0c, 0x03, 0x60, 0x01, 0x7e,
            0x00, 0x60, 0x00, 0x01, 0x7e, 0x60, 0x00, 0x00, 0x02, 0x2f, 0x02, 0x08, 0x65, 0x74,
            0x68, 0x65, 0x72, 0x65, 0x75, 0x6d, 0x06, 0x75, 0x73, 0x65, 0x47, 0x61, 0x73, 0x00,
            0x00, 0x08, 0x65, 0x74, 0x68, 0x65, 0x72, 0x65, 0x75, 0x6d, 0x10, 0x67, 0x65, 0x74,
            0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x47, 0x61, 0x73, 0x4c, 0x69, 0x6d, 0x69, 0x74, 0x00,
            0x01, 0x03, 0x07, 0x06, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x05, 0x03, 0x01, 0x00,
            0x01, 0x07, 0x11, 0x02, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x02, 0x06, 0x6d, 0x65,
            0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x0a, 0x46, 0x06, 0x06, 0x00, 0x10, 0x03, 0x10,
            0x04, 0x0b, 0x08, 0x00, 0x10, 0x05, 0x10, 0x06, 0x10, 0x07, 0x0b, 0x0d, 0x00, 0x10,
            0x06, 0x41, 0x00, 0x10, 0x01, 0x37, 0x03, 0x00, 0x10, 0x07, 0x0b, 0x10, 0x00, 0x42,
            0xa4, 0x03, 0x10, 0x00, 0x41, 0x00, 0x10, 0x01, 0x37, 0x03, 0x00, 0x10, 0x03, 0x0b,
            0x09, 0x00, 0x42, 0xb9, 0x0a, 0x10, 0x00, 0x10, 0x06, 0x0b, 0x0b, 0x00, 0x41, 0x00,
            0x10, 0x01, 0x37, 0x03, 0x00, 0x10, 0x04, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let g = DepGraph::build(&module, 2).unwrap();

        assert!(g.edgecount() == 15);
    }
}
