use std::collections::HashMap;
use std::error::Error;

use parity_wasm::elements::{Module, Section};

use super::{ChiselModule, ModuleConfig, ModuleError, ModuleKind, ModuleTranslator};

/// Enum on which ModuleTranslator is implemented.
pub enum DropSection {
    NamesSection,
    /// Name of the custom section.
    CustomSectionByName(String),
    /// Index of the custom section.
    CustomSectionByIndex(usize),
    /// Index of the unknown section.
    UnknownSectionByIndex(usize),
}

impl<'a> ChiselModule<'a> for DropSection {
    type ObjectReference = &'a dyn ModuleTranslator;

    fn id(&'a self) -> String {
        "dropsection".to_string()
    }

    fn kind(&'a self) -> ModuleKind {
        ModuleKind::Translator
    }

    fn as_abstract(&'a self) -> Self::ObjectReference {
        self as Self::ObjectReference
    }
}

impl From<std::num::ParseIntError> for ModuleError {
    fn from(error: std::num::ParseIntError) -> Self {
        ModuleError::Custom(error.description().to_string())
    }
}

impl ModuleConfig for DropSection {
    fn with_defaults() -> Result<Self, ModuleError> {
        Err(ModuleError::NotSupported)
    }

    fn with_config(config: &HashMap<String, String>) -> Result<Self, ModuleError> {
        if let Some((key, val)) = config.iter().next() {
            return match key.as_str() {
                "names" => Ok(DropSection::NamesSection),
                "custom_by_name" => Ok(DropSection::CustomSectionByName(val.clone())),
                "custom_by_index" => {
                    Ok(DropSection::CustomSectionByIndex(str::parse::<usize>(val)?))
                }
                "unknown_by_index" => Ok(DropSection::UnknownSectionByIndex(str::parse::<usize>(
                    val,
                )?)),
                _ => Err(ModuleError::NotSupported),
            };
        } else {
            Err(ModuleError::NotFound)
        }
    }
}

// TODO: consider upstreaming this
fn custom_section_index_for(module: &Module, name: &str) -> Option<usize> {
    module.sections().iter().position(|e| match e {
        Section::Custom(_section) => _section.name() == name,
        Section::Name(_) => name == "name", // If the names section was parsed by parity-wasm, it is distinct from a custom section.
        _ => false,
    })
}

impl DropSection {
    fn find_index(&self, module: &Module) -> Option<usize> {
        match &self {
            DropSection::NamesSection => custom_section_index_for(module, "name"),
            DropSection::CustomSectionByName(name) => custom_section_index_for(module, &name),
            DropSection::CustomSectionByIndex(index) => Some(*index),
            DropSection::UnknownSectionByIndex(index) => Some(*index),
        }
    }

    fn drop_section(&self, module: &mut Module) -> Result<bool, ModuleError> {
        if let Some(index) = self.find_index(&module) {
            let sections = module.sections_mut();
            if index < sections.len() {
                sections.remove(index);
                return Ok(true);
            }
        }

        Ok(false)
    }
}

impl<'a> ModuleTranslator for DropSection {
    fn translate_inplace(&self, module: &mut Module) -> Result<bool, ModuleError> {
        Ok(self.drop_section(module)?)
    }

    fn translate(&self, module: &Module) -> Result<Option<Module>, ModuleError> {
        let mut ret = module.clone();
        if self.drop_section(&mut ret)? {
            Ok(Some(ret))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use parity_wasm::builder;
    use parity_wasm::elements::CustomSection;
    use rustc_hex::FromHex;

    use super::*;

    #[test]
    fn keep_intact() {
        let mut module = builder::module().build();
        let name = "empty".to_string();
        let dropper = DropSection::CustomSectionByName(name);
        let did_change = dropper.translate_inplace(&mut module).unwrap();
        assert_eq!(did_change, false);
    }

    #[test]
    fn keep_intact_custom_section() {
        let mut module = builder::module()
            .with_section(Section::Custom(CustomSection::new(
                "test".to_string(),
                vec![],
            )))
            .build();
        let name = "empty".to_string();
        let dropper = DropSection::CustomSectionByName(name);
        let did_change = dropper.translate_inplace(&mut module).unwrap();
        assert_eq!(did_change, false);
    }

    #[test]
    fn remove_custom_section() {
        let mut module = builder::module()
            .with_section(Section::Custom(CustomSection::new(
                "test".to_string(),
                vec![],
            )))
            .build();
        let name = "test".to_string();
        let dropper = DropSection::CustomSectionByName(name);
        let did_change = dropper.translate_inplace(&mut module).unwrap();
        assert_eq!(did_change, true);
    }

    #[test]
    fn remove_custom_section_by_index() {
        let mut module = builder::module()
            .with_section(Section::Custom(CustomSection::new(
                "test".to_string(),
                vec![],
            )))
            .build();
        let dropper = DropSection::CustomSectionByIndex(0);
        let did_change = dropper.translate_inplace(&mut module).unwrap();
        assert_eq!(did_change, true);
    }

    #[test]
    fn remove_oob_custom_section_by_index() {
        let mut module = builder::module()
            .with_section(Section::Custom(CustomSection::new(
                "test".to_string(),
                vec![],
            )))
            .build();
        let dropper = DropSection::CustomSectionByIndex(1);
        let did_change = dropper.translate_inplace(&mut module).unwrap();
        assert_eq!(did_change, false);
    }

    #[test]
    fn remove_custom_unknown_by_index() {
        let mut module = builder::module()
            .with_section(Section::Custom(CustomSection::new(
                "test".to_string(),
                vec![],
            )))
            .build();
        let dropper = DropSection::UnknownSectionByIndex(0);
        let did_change = dropper.translate_inplace(&mut module).unwrap();
        assert_eq!(did_change, true);
    }

    #[test]
    fn remove_oob_unknown_section_by_index() {
        let mut module = builder::module()
            .with_section(Section::Custom(CustomSection::new(
                "test".to_string(),
                vec![],
            )))
            .build();
        let dropper = DropSection::UnknownSectionByIndex(1);
        let did_change = dropper.translate_inplace(&mut module).unwrap();
        assert_eq!(did_change, false);
    }

    #[test]
    fn names_section_index() {
        let input = FromHex::from_hex(
            "0061736d010000000104016000000303020000070801046d61696e00010a
            0a020300010b040010000b0014046e616d65010d0200047465737401046d
            61696e",
        )
        .unwrap();

        let mut module = Module::from_bytes(&input).unwrap();
        assert!(custom_section_index_for(&module, "name").is_some());
        module = module.parse_names().expect("Should not fail");
        assert!(custom_section_index_for(&module, "name").is_some());
    }

    #[test]
    fn missing_name_section_index() {
        let input = FromHex::from_hex(
            "0061736d010000000104016000000303020000070801046d61696e00010a
            0a020300010b040010000b",
        )
        .unwrap();

        let module = Module::from_bytes(&input).unwrap();
        assert!(custom_section_index_for(&module, "name").is_none());
    }

    #[test]
    fn dropped_name_section_parsed() {
        let input = FromHex::from_hex(
            "0061736d010000000104016000000303020000070801046d61696e00010a
            0a020300010b040010000b0014046e616d65010d0200047465737401046d
            61696e",
        )
        .unwrap();

        let mut module = Module::from_bytes(&input)
            .unwrap()
            .parse_names()
            .expect("Should not fail");
        let mut module1 = module.clone();

        assert!(custom_section_index_for(&module, "name").is_some());

        let dropper = DropSection::CustomSectionByName("name".to_string());
        let dropper1 = DropSection::NamesSection;

        assert!(dropper
            .translate_inplace(&mut module)
            .expect("Should not fail"));
        assert!(dropper1
            .translate_inplace(&mut module1)
            .expect("Should not fail"));

        assert!(custom_section_index_for(&module, "name").is_none());
        assert!(custom_section_index_for(&module1, "name").is_none());
    }

    #[test]
    fn dropped_name_section_index_unparsed() {
        let input = FromHex::from_hex(
            "0061736d010000000104016000000303020000070801046d61696e00010a
            0a020300010b040010000b0014046e616d65010d0200047465737401046d
            61696e",
        )
        .unwrap();

        let mut module = Module::from_bytes(&input).unwrap();
        let mut module1 = module.clone();

        assert!(custom_section_index_for(&module, "name").is_some());

        let dropper = DropSection::CustomSectionByName("name".to_string());
        let dropper1 = DropSection::NamesSection;

        assert!(dropper
            .translate_inplace(&mut module)
            .expect("Should not fail"));
        assert!(dropper1
            .translate_inplace(&mut module1)
            .expect("Should not fail"));

        assert!(custom_section_index_for(&module, "name").is_none());
        assert!(custom_section_index_for(&module1, "name").is_none());
    }
}
