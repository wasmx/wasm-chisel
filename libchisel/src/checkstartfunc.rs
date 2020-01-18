use std::collections::HashMap;

use parity_wasm::elements::Module;

use super::{ChiselModule, ModuleError, ModuleKind, ModuleValidator};

/// Struct on which ModuleValidator is implemented.
pub struct CheckStartFunc {
    start_required: bool,
}

impl CheckStartFunc {
    pub fn new(is_start_required: bool) -> Self {
        CheckStartFunc {
            start_required: is_start_required,
        }
    }
}

impl<'a> ChiselModule<'a> for CheckStartFunc {
    type ObjectReference = &'a dyn ModuleValidator;

    fn id(&'a self) -> String {
        "checkstartfunc".to_string()
    }

    fn kind(&'a self) -> ModuleKind {
        ModuleKind::Validator
    }

    fn as_abstract(&'a self) -> Self::ObjectReference {
        self as Self::ObjectReference
    }

    fn with_defaults() -> Result<Self, ModuleError> {
        Err(ModuleError::NotSupported)
    }

    fn with_config(config: &HashMap<String, String>) -> Result<Self, ModuleError> {
        let require_start = if let Some(value) = config.get("require_start") {
            value == "true"
        } else {
            false
        };
        Ok(CheckStartFunc {
            start_required: require_start,
        })
    }
}

impl ModuleValidator for CheckStartFunc {
    fn validate(&self, module: &Module) -> Result<bool, ModuleError> {
        Ok(module.start_section().is_some() == self.start_required)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_required_good() {
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            0x08, 0x01, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = Module::from_bytes(&wasm).unwrap();
        let checker = CheckStartFunc::new(true);

        let result = checker.validate(&module).unwrap();
        assert_eq!(true, result);
    }

    #[test]
    fn start_required_bad() {
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            0x08, 0x01, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = Module::from_bytes(&wasm).unwrap();
        let checker = CheckStartFunc::new(false);

        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn start_not_required_good() {
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = Module::from_bytes(&wasm).unwrap();
        let checker = CheckStartFunc::new(false);

        let result = checker.validate(&module).unwrap();
        assert_eq!(true, result);
    }

    #[test]
    fn start_not_required_bad() {
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = Module::from_bytes(&wasm).unwrap();
        let checker = CheckStartFunc::new(true);

        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }
}
