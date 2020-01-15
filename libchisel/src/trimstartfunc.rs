use std::collections::HashMap;

use parity_wasm::elements::Module;

use super::{ChiselModule, ModuleConfig, ModuleError, ModuleKind, ModulePreset, ModuleTranslator};

pub struct TrimStartFunc;

impl TrimStartFunc {
    fn trim_startfunc(&self, module: &mut Module) -> bool {
        if let Some(_start_section) = module.start_section() {
            module.clear_start_section();
            true
        } else {
            false
        }
    }
}

impl<'a> ChiselModule<'a> for TrimStartFunc {
    type ObjectReference = &'a dyn ModuleTranslator;

    fn id(&'a self) -> String {
        "trimstartfunc".to_string()
    }

    fn kind(&'a self) -> ModuleKind {
        ModuleKind::Translator
    }

    fn as_abstract(&'a self) -> Self::ObjectReference {
        self as Self::ObjectReference
    }
}

impl ModuleConfig for TrimStartFunc {
    fn with_defaults() -> Result<Self, ModuleError> {
        Err(ModuleError::NotSupported)
    }

    fn with_config(config: &HashMap<String, String>) -> Result<Self, ModuleError> {
        if let Some(preset) = config.get("preset") {
            TrimStartFunc::with_preset(preset)
        } else {
            Err(ModuleError::NotSupported)
        }
    }
}

impl ModulePreset for TrimStartFunc {
    fn with_preset(preset: &str) -> Result<Self, ModuleError> {
        match preset {
            "ewasm" => Ok(TrimStartFunc {}),
            _ => Err(ModuleError::NotSupported),
        }
    }
}

impl ModuleTranslator for TrimStartFunc {
    fn translate_inplace(&self, module: &mut Module) -> Result<bool, ModuleError> {
        Ok(self.trim_startfunc(module))
    }

    fn translate(&self, _module: &Module) -> Result<Option<Module>, ModuleError> {
        Err(ModuleError::NotSupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_removed() {
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            0x08, 0x01, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let mut module = Module::from_bytes(&wasm).unwrap();

        let trimmer = TrimStartFunc::with_preset("ewasm").unwrap();
        trimmer.translate_inplace(&mut module).unwrap();

        let result = module.to_bytes().unwrap();
        let expect: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        assert_eq!(expect, result);
    }

    #[test]
    fn start_not_removed() {
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let mut module = Module::from_bytes(&wasm).unwrap();

        let trimmer = TrimStartFunc::with_preset("ewasm").unwrap();
        trimmer.translate_inplace(&mut module).unwrap();

        let result = module.to_bytes().unwrap();

        // result is equal to initial wasm (not changed)
        assert_eq!(result, wasm);
    }
}
