use super::ModuleValidator;
use parity_wasm::elements::{
    ExportEntry, ExportSection, External, FunctionSection, FunctionType, ImportSection, Internal,
    Module, Type,
};

/// Module struct on which to implement ModuleValidator.
pub struct CheckFuncExport {
    main_funcsig: FunctionType,
}

impl CheckFuncExport {
    /// ewasm preset. "main" takes no arguments and has no return value.
    pub fn ewasm() -> Self {
        CheckFuncExport {
            main_funcsig: FunctionType::default(),
        }
    }
}

impl ModuleValidator for CheckFuncExport {
    fn validate(self, module: &Module) -> Result<bool, String> {
        Ok(has_func_export(module, "main", self.main_funcsig))
    }
}

/// Returns whether a module has a function export of a given signature and name.
fn has_func_export(module: &Module, field_str: &str, sig: FunctionType) -> bool {
    if let Some(section) = module.export_section() {
        match func_export_index_by_name(section, field_str) {
            Some(index) => if let Some(resolved) = func_sig_by_index(module, index) {
                sig == *resolved
            } else {
                false
            },
            None => false,
        }
    } else {
        false
    }
}

/// Resolve's a function's signature from its internal index.
fn func_sig_by_index(module: &Module, index: u32) -> Option<&FunctionType> {
    if let Some(s_funcs) = module.function_section() {
        match (module.type_section(), module.import_section()) {
            // If we have function imports in the section, subtract them from the function's index
            // because their signatures are not in the types section.
            (Some(s_types), Some(s_imports)) => match s_types.types()
                [func_type_ref(&s_funcs, index - func_import_section_len(s_imports))]
            {
                Type::Function(ref ret) => Some(ret),
                _ => None,
            },
            // If no function imports are present, no need to subtract them.
            (Some(s_types), None) => match s_types.types()[func_type_ref(&s_funcs, index)] {
                Type::Function(ref ret) => Some(ret),
                _ => None,
            },
            (None, Some(s_imports)) => None,
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
        }).count() as u32
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
    fn main_export_good() {
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = CheckFuncExport::ewasm();

        let result = checker.validate(&module).unwrap();

        assert_eq!(true, result);
    }

    #[test]
    fn main_export_has_param() {
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x01, 0x7f,
            0x00, 0x03, 0x02, 0x01, 0x00, 0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00,
            0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = CheckFuncExport::ewasm();

        let result = checker.validate(&module).unwrap();

        assert_eq!(false, result);
    }

    #[test]
    fn no_main_export() {
        let wasm: Vec<u8> = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = CheckFuncExport::ewasm();

        let result = checker.validate(&module).unwrap();

        assert_eq!(false, result);
    }

    #[test]
    fn main_export_has_return() {
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01,
            0x7f, 0x03, 0x02, 0x01, 0x00, 0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00,
            0x00, 0x0a, 0x06, 0x01, 0x04, 0x00, 0x41, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = CheckFuncExport::ewasm();

        let result = checker.validate(&module).unwrap();

        assert_eq!(false, result);
    }

    #[test]
    fn main_export_is_mem() {
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x05, 0x03, 0x01, 0x00, 0x00, 0x07,
            0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x02, 0x00,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = CheckFuncExport::ewasm();

        let result = checker.validate(&module).unwrap();

        assert_eq!(false, result);
    }
}
