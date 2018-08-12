extern crate parity_wasm;
#[macro_use]
extern crate log;

use parity_wasm::elements::*;

use std::env;

// FIXME: There is no `Module::import_section_mut()`
fn import_section_mut(module: &mut Module) -> Option<&mut ImportSection> {
    for section in module.sections_mut() {
        if let &mut Section::Import(ref mut import_section) = section { return Some(import_section); }
    }
    None
}

pub fn rename_imports(module: &mut Module) {
    if let Some(section) = import_section_mut(module) {
        for entry in section.entries_mut().iter_mut() {
            if entry.module() == "env" && entry.field() == "ethereum_return" {
              *entry = ImportEntry::new("ethereum".to_string(), "finish".to_string(), *entry.external())
            }
        }
    }
}

pub fn cleanup() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 3 {
        println!("Usage: {} in.wasm out.wasm", args[0]);
        return;
    }

    let module = parity_wasm::deserialize_file(&args[1]).expect("Failed to load module");

    if let Some(section) = module.function_section() {
        for (i, entry) in section.entries().iter().enumerate() {
            debug!("function {:?}", i);
        }
    }

    if let Some(section) = module.code_section() {
        for (i, entry) in section.bodies().iter().enumerate() {
            for opcode in entry.code().elements() {
              debug!("opcode {:?}", opcode)
              // iterate opcodes..
            }
        }
    }

    parity_wasm::serialize_to_file(&args[2], module).expect("Failed to write module");
}

#[cfg(test)]
mod tests {
    use parity_wasm;

    #[test]
    fn smoke_test() {
        let mut module = parity_wasm::deserialize_file("src/test.wasm").expect("failed");
        ::rename_imports(&mut module);
        parity_wasm::serialize_to_file("src/test-out.wasm", module).expect("failed");
    }
}
