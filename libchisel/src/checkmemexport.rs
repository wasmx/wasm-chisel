use super::ModuleValidator;
use parity_wasm::elements::{Internal, Module};

/// Struct on which ModuleValidator is implemented.
pub struct CheckMemExport<'a> {
    export_name: &'a str,
}

impl<'a> CheckMemExport<'a> {
    /// ewasm preset.
    pub fn ewasm() -> Self {
        CheckMemExport {
            export_name: "memory",
        }
    }
}

impl<'a> ModuleValidator for CheckMemExport<'a> {
    fn validate(self, module: &Module) -> Result<bool, String> {
        Ok(has_mem_export(module, self.export_name))
    }
}

/// Checks if a memory is exported with the given name.
fn has_mem_export(module: &Module, field_str: &str) -> bool {
    if let Some(section) = module.export_section() {
        if let Some(export) = section.entries().iter().find(|e| e.field() == field_str) {
            match *export.internal() {
                Internal::Memory(index) => true,
                _ => false,
            }
        } else {
            false
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_wasm::elements::deserialize_buffer;

    #[test]
    fn has_mem_export_good() {
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x05, 0x03, 0x01, 0x00, 0x00, 0x07,
            0x0a, 0x01, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = CheckMemExport::ewasm();

        let result = checker.validate(&module).unwrap();

        assert_eq!(true, result);
    }

    #[test]
    fn has_mem_export_malformed() {
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x07, 0x0a, 0x01, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79,
            0x00, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = CheckMemExport::ewasm();

        let result = checker.validate(&module).unwrap();

        assert_eq!(false, result);
    }

    #[test]
    fn no_mem_export() {
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = CheckMemExport::ewasm();

        let result = checker.validate(&module).unwrap();

        assert_eq!(false, result);
    }
}
