use std::collections::HashMap;

use parity_wasm::elements::Module;

use super::{ChiselModule, ModuleConfig, ModuleError, ModuleKind, ModulePreset, ModuleTranslator};

// FIXME: change level names
pub enum BinaryenOptimiser {
    O0, // Baseline aka no changes
    O1,
    O2,
    O3,
    O4,
    Os,
    Oz,
}

impl<'a> ChiselModule<'a> for BinaryenOptimiser {
    type ObjectReference = &'a dyn ModuleTranslator;

    fn id(&'a self) -> String {
        "binaryenopt".to_string()
    }

    fn kind(&'a self) -> ModuleKind {
        ModuleKind::Translator
    }

    fn as_abstract(&'a self) -> Self::ObjectReference {
        self as Self::ObjectReference
    }
}

impl ModuleConfig for BinaryenOptimiser {
    fn with_defaults() -> Result<Self, ModuleError> {
        Ok(BinaryenOptimiser::O2)
    }

    fn with_config(config: &HashMap<String, String>) -> Result<Self, ModuleError> {
        if let Some(preset) = config.get("preset") {
            BinaryenOptimiser::with_preset(preset)
        } else {
            Err(ModuleError::NotSupported)
        }
    }
}

impl ModulePreset for BinaryenOptimiser {
    fn with_preset(preset: &str) -> Result<Self, ModuleError> {
        match preset {
            "O0" => Ok(BinaryenOptimiser::O0),
            "O1" => Ok(BinaryenOptimiser::O1),
            "O2" => Ok(BinaryenOptimiser::O2),
            "O3" => Ok(BinaryenOptimiser::O3),
            "O4" => Ok(BinaryenOptimiser::O4),
            "Os" => Ok(BinaryenOptimiser::Os),
            "Oz" => Ok(BinaryenOptimiser::Oz),
            _ => Err(ModuleError::NotSupported),
        }
    }
}

impl ModuleTranslator for BinaryenOptimiser {
    fn translate_inplace(&self, _module: &mut Module) -> Result<bool, ModuleError> {
        Err(ModuleError::NotSupported)
    }

    fn translate(&self, module: &Module) -> Result<Option<Module>, ModuleError> {
        let has_names_section = module.has_names_section();

        // FIXME: could just move this into `BinaryenOptimiser`
        let config = match &self {
            BinaryenOptimiser::O0 => binaryen::CodegenConfig {
                optimization_level: 0,
                shrink_level: 0,
                debug_info: has_names_section,
            },
            BinaryenOptimiser::O1 => binaryen::CodegenConfig {
                optimization_level: 1,
                shrink_level: 0,
                debug_info: has_names_section,
            },
            BinaryenOptimiser::O2 => binaryen::CodegenConfig {
                optimization_level: 2,
                shrink_level: 0,
                debug_info: has_names_section,
            },
            BinaryenOptimiser::O3 => binaryen::CodegenConfig {
                optimization_level: 3,
                shrink_level: 0,
                debug_info: has_names_section,
            },
            BinaryenOptimiser::O4 => binaryen::CodegenConfig {
                optimization_level: 4,
                shrink_level: 0,
                debug_info: has_names_section,
            },
            BinaryenOptimiser::Os => binaryen::CodegenConfig {
                optimization_level: 2,
                shrink_level: 1,
                debug_info: has_names_section,
            },
            BinaryenOptimiser::Oz => binaryen::CodegenConfig {
                optimization_level: 2,
                shrink_level: 2,
                debug_info: has_names_section,
            },
        };

        let serialized = module.clone().to_bytes()?;
        let output = binaryen_optimiser(&serialized, &config)?;
        let output = Module::from_bytes(&output)?;
        Ok(Some(output))
    }
}

fn binaryen_optimiser(
    input: &[u8],
    config: &binaryen::CodegenConfig,
) -> Result<Vec<u8>, ModuleError> {
    match binaryen::Module::read(&input) {
        Ok(module) => {
            // NOTE: this is a global setting...
            binaryen::set_global_codegen_config(&config);
            module.optimize();
            Ok(module.write())
        }
        Err(_) => Err(ModuleError::Custom(
            "Failed to deserialise binary with binaryen".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_o0() {
        let input: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            0x08, 0x01, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let expected: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            0x0a, 0x05, 0x01, 0x03, 0x00, 0x01, 0x0b,
        ];

        let module = Module::from_bytes(&input).unwrap();
        let translator = BinaryenOptimiser::with_preset("O0").unwrap();
        let result = translator.translate(&module).unwrap().unwrap();
        let serialized = result.to_bytes().unwrap();
        assert_eq!(expected, serialized);
    }
}
