use std::collections::HashMap;

use super::{ModuleError, ModulePreset, ModuleTranslator};
use parity_wasm::elements::*;


#[derive(Default)]
pub struct Translations {
    translations: HashMap<String, String>,
}

impl ModulePreset for Translations {
    fn with_preset(preset: &str) -> Result<Self, ()> {
        match preset {
            "ewasm" => {
                let trans: HashMap<String, String> = [
                    (
                        "_main".to_string(),
                        "main".to_string()
                    )                ]
                .iter()
                .cloned()
                .collect();
                Ok(Translations {
                    translations: trans
                })
            }
            _ => Err(()),
        }
    }
}

impl Translations {
/*
    fn insert(&mut self, from_module: &str, from_field: &str, to_module: &str, to_field: &str) {
        self.translations.insert(
            ImportPair::new(from_module, from_field),
            ImportPair::new(to_module, to_field),
        );
    }
*/

    //    fn get_simple(&self, module: &str, field: &str) -> Option<&str, &str> {
    //        if let Some(translation) = self.translations.get(&ImportPair::new(module, field)) {
    //            Some(translation.module.clone(), translation.field.clone())
    //        } else {
    //            None
    //        }
    //    }

    fn get(&self, export: &String) -> Option<&String> {
        self.translations.get(export)
    }
}

pub struct RemapExports {
    translations: Translations,
}

impl ModulePreset for RemapExports {
    fn with_preset(preset: &str) -> Result<Self, ()> {
        match preset {
            "ewasm" => Ok(RemapExports {
                translations: Translations::with_preset("ewasm").unwrap(),
            }),
            _ => Err(()),
        }
    }
}

impl ModuleTranslator for RemapExports {
    fn translate_inplace(&self, module: &mut Module) -> Result<bool, ModuleError> {
        Ok(rename_exports(module, &self.translations))
    }

    fn translate(&self, module: &Module) -> Result<Option<Module>, ModuleError> {
        let mut ret = module.clone();
        let modified = rename_exports(&mut ret, &self.translations);
        if modified {
            return Ok(Some(ret));
        }
        Ok(None)
    }
}

fn rename_exports(module: &mut Module, translations: &Translations) -> bool {
    let mut ret = false;
    if let Some(section) = module.export_section_mut() {
        for entry in section.entries_mut().iter_mut() {
            if let Some(replacement) =
                translations.get(&entry.field().to_string())
            {
                ret = true;
                *entry = ExportEntry::new(replacement.clone(),
                    *entry.internal());
            }
        }
    }
    ret
}
