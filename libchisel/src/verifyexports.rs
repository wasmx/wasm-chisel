use super::{ModuleError, ModulePreset, ModuleValidator};
use parity_wasm::elements::{
    ExportSection, External, FunctionSection, FunctionType, ImportSection, Internal, Module, Type,
};

/// Enum representing a type of export and any extra data to check.
pub enum ExportType<'a> {
    Function(&'a str, FunctionType),
    Global(&'a str),
    Memory(&'a str),
    Table(&'a str),
}

/// Trait over ExportType that lets a caller check if it is exported in a given module.
trait IsExported {
    fn is_exported(&self, module: &Module) -> bool;
}

/// Struct on which ModuleValidator is implemented.
pub struct VerifyExports<'a> {
    entries: Vec<ExportType<'a>>,
    allow_unlisted: bool,
}

impl<'a> ModulePreset for VerifyExports<'a> {
    fn with_preset(preset: &str) -> Result<Self, ()> {
        match preset {
            "ewasm" => Ok(VerifyExports {
                entries: vec![
                    ExportType::Function("main", FunctionType::default()),
                    ExportType::Memory("memory"),
                ],
                allow_unlisted: false,
            }),
            _ => Err(()),
        }
    }
}

impl<'a> ModuleValidator for VerifyExports<'a> {
    fn validate(&self, module: &Module) -> Result<bool, ModuleError> {
        // FIXME: This validating algorithm runs in O(n^2). Needs to be optimized
        let required_exports_not_found = self
            .entries
            .iter()
            .map(|e| e.is_exported(module))
            .find(|e| *e == false)
            .is_some();

        if required_exports_not_found {
            return Ok(false);
        }

        let module_export_count = if let Some(section) = module.export_section() {
            section.entries().len()
        } else {
            0
        };

        if self.entries.len() != module_export_count {
            Ok(self.allow_unlisted)
        } else {
            Ok(true)
        }
    }
}

impl<'a> IsExported for ExportType<'a> {
    fn is_exported(&self, module: &Module) -> bool {
        if let Some(section) = module.export_section() {
            match self {
                ExportType::Function(field, sig) => has_func_export(module, field, sig),
                ExportType::Global(field) => has_global_export(section, field),
                ExportType::Memory(field) => has_memory_export(section, field),
                ExportType::Table(field) => has_table_export(section, field),
            }
        } else {
            false
        }
    }
}

// NOTE: has_*_export is implemented with repeating code because you can't implement a trait for
// enum variants, as they are not types. Furthermore, having one helper for non-func exports would
// be ugly because information about the export type must still be passed down to check that an
// export is of the correct kind.

/// Checks if a global is exported with the given name.
fn has_global_export(section: &ExportSection, field: &str) -> bool {
    if let Some(ref export) = section.entries().iter().find(|e| e.field() == field) {
        match export.internal() {
            Internal::Global(_index) => true,
            _ => false,
        }
    } else {
        false
    }
}

/// Checks if a memory is exported with the given name.
fn has_memory_export(section: &ExportSection, field: &str) -> bool {
    if let Some(ref export) = section.entries().iter().find(|e| e.field() == field) {
        match export.internal() {
            Internal::Memory(_index) => true,
            _ => false,
        }
    } else {
        false
    }
}

/// Checks if a table is exported with the given name.
fn has_table_export(section: &ExportSection, field: &str) -> bool {
    if let Some(ref export) = section.entries().iter().find(|e| e.field() == field) {
        match export.internal() {
            Internal::Table(_index) => true,
            _ => false,
        }
    } else {
        false
    }
}

// NOTE: this is kind of hacked on. It works, but a refactor would make it more in line with the other
// helpers.
/// Checks if a function is exported with the given name.
fn has_func_export(module: &Module, field: &str, sig: &FunctionType) -> bool {
    if let Some(section) = module.export_section() {
        match func_export_index_by_name(section, field) {
            Some(index) => {
                if let Some(resolved) = func_sig_by_index(module, index) {
                    *sig == *resolved
                } else {
                    false
                }
            }
            None => false,
        }
    } else {
        false
    }
}

/// Resolves a function's signature from its internal index.
fn func_sig_by_index(module: &Module, index: u32) -> Option<&FunctionType> {
    if let Some(func_section) = module.function_section() {
        match (module.type_section(), module.import_section()) {
            // If we have function imports in the section, subtract them from the function's index
            // because their signatures are not in the types section.
            (Some(type_section), Some(import_section)) => match type_section.types()[func_type_ref(
                &func_section,
                index - func_import_section_len(import_section),
            )] {
                Type::Function(ref ret) => Some(ret),
                _ => None,
            },
            // If no function imports are present, no need to subtract them.
            (Some(type_section), None) => {
                match type_section.types()[func_type_ref(&func_section, index)] {
                    Type::Function(ref ret) => Some(ret),
                    _ => None,
                }
            }
            (None, Some(import_section)) => None,
            (None, None) => None,
        }
    } else {
        None
    }
}

/// Returns the internal reference to a function's type signature.
fn func_type_ref(funcs: &FunctionSection, func_index: u32) -> usize {
    funcs.entries()[func_index as usize].type_ref() as usize
}

/// Returns the number of functions in the function section that are imported.
fn func_import_section_len(imports: &ImportSection) -> u32 {
    imports
        .entries()
        .iter()
        .filter(|e| match e.external() {
            &External::Function(_) => true,
            _ => false,
        })
        .count() as u32
}

/// Resolves a function export's index by name. Can be trivially adjusted for
/// all types of exports.
fn func_export_index_by_name(exports: &ExportSection, field_str: &str) -> Option<u32> {
    if let Some(entry) = exports.entries().iter().find(|e| e.field() == field_str) {
        match entry.internal() {
            Internal::Function(index) => Some(*index),
            _ => None,
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_wasm::elements::deserialize_buffer;

    #[test]
    fn no_exports() {
        // wast:
        // (module)
        let wasm: Vec<u8> = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyExports::with_preset("ewasm").unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn all_exports_good_ewasm() {
        // wast:
        // (module
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x11, 0x02, 0x04, 0x6d,
            0x61, 0x69, 0x6e, 0x00, 0x00, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00,
            0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyExports::with_preset("ewasm").unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(true, result);
    }

    #[test]
    fn main_export_returns_i32_ewasm() {
        // wast:
        // (module
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main (result i32)
        //     (i32.const 0)
        //   )
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01,
            0x7f, 0x03, 0x02, 0x01, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x11, 0x02, 0x04,
            0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02,
            0x00, 0x0a, 0x06, 0x01, 0x04, 0x00, 0x41, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyExports::with_preset("ewasm").unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn main_export_takes_param_i32_ewasm() {
        // wast:
        // (module
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main (param i32))
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x01, 0x7f,
            0x00, 0x03, 0x02, 0x01, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x11, 0x02, 0x04,
            0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02,
            0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyExports::with_preset("ewasm").unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn missing_mem_export_ewasm() {
        // wast:
        // (module
        //   (memory 1)
        //   (export "main" (func $main))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x01, 0x7f,
            0x00, 0x03, 0x02, 0x01, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x08, 0x01, 0x04,
            0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyExports::with_preset("ewasm").unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn missing_main_export_ewasm() {
        // wast:
        // (module
        //   (memory 1)
        //   (export "memory" (memory 0))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x01, 0x7f,
            0x00, 0x03, 0x02, 0x01, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x0a, 0x01, 0x06,
            0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyExports::with_preset("ewasm").unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn mem_export_points_to_main_ewasm() {
        // wast:
        // (module
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (func $main))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x11, 0x02, 0x06, 0x6d,
            0x65, 0x6d, 0x6f, 0x72, 0x79, 0x00, 0x00, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyExports::with_preset("ewasm").unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn main_export_points_to_mem_ewasm() {
        // wast:
        // (module
        //   (memory 1)
        //   (export "main" (memory 0))
        //   (export "memory" (memory 0))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x11, 0x02, 0x06, 0x6d,
            0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x02, 0x00,
            0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyExports::with_preset("ewasm").unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn main_export_spelled_wrong_ewasm() {
        // wast:
        // (module
        //   (memory 1)
        //   (export "man" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x10, 0x02, 0x06, 0x6d,
            0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x03, 0x6d, 0x61, 0x6e, 0x00, 0x00, 0x0a,
            0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyExports::with_preset("ewasm").unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn extra_export_disallowed_ewasm() {
        // wast:
        // (module
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (export "foo" (func $main))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x1a, 0x03, 0x06, 0x6d,
            0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            0x06, 0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72, 0x00, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00,
            0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyExports::with_preset("ewasm").unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn extra_export_allowed_ewasm() {
        // wast:
        // (module
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (export "foo" (func $main))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x1a, 0x03, 0x06, 0x6d,
            0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            0x06, 0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72, 0x00, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00,
            0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyExports {
            entries: vec![
                ExportType::Function("main", FunctionType::default()),
                ExportType::Memory("memory"),
            ],
            allow_unlisted: true,
        };
        let result = checker.validate(&module).unwrap();
        assert_eq!(true, result);
    }
}
