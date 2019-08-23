use super::{ChiselModule, ModuleError, ModuleKind, ModuleTranslator};

use parity_wasm::elements::*;

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

fn names_section_index_for(module: &Module) -> Option<usize> {
    module.sections().iter().position(|e| {
        match e {
            // The default case, when the section was not parsed by parity-wasm
            Section::Custom(_section) => _section.name() == "name",
            // This is the case, when the section was parsed by parity-wasm
            Section::Name(_) => true,
            _ => false,
        }
    })
}

fn custom_section_index_for(module: &Module, name: &str) -> Option<usize> {
    module.sections().iter().position(|e| match e {
        Section::Custom(_section) => _section.name() == name,
        _ => false,
    })
}

impl DropSection {
    fn find_index(&self, module: &Module) -> Option<usize> {
        match &self {
            DropSection::NamesSection => names_section_index_for(module),
            DropSection::CustomSectionByName(name) => custom_section_index_for(module, &name),
            DropSection::CustomSectionByIndex(index) => Some(*index),
            DropSection::UnknownSectionByIndex(index) => Some(*index),
        }
    }

    fn drop_section(&self, module: &mut Module) -> Result<bool, ModuleError> {
        let index = self.find_index(&module);
        if index.is_none() {
            return Ok(false);
        }
        let index = index.unwrap();

        let sections = module.sections_mut();
        if index >= sections.len() {
            return Ok(false);
        }
        sections.remove(index);

        Ok(true)
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
    use super::*;
    use parity_wasm::builder;

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
}
