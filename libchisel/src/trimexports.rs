use super::*;

use parity_wasm::builder::*;
use parity_wasm::elements::*;

/// Struct containing a list of valid exports.
struct ExportWhitelist {
    pub entries: Vec<ExportEntry>,
}

/// Wrapper struct implementing ModuleTranslator.
/// Removes any exports that are noncompliant with a specified interface.
pub struct TrimExports {
    whitelist: ExportWhitelist,
}

/// Helper that compares the enum variant of two WebAssembly exports.
fn cmp_internal_variant(a: &Internal, b: &Internal) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

// FIXME: Use the real implementation once the parity-wasm version is bumped
fn export_section_mut(module: &mut Module) -> Option<&mut ExportSection> {
    for section in module.sections_mut() {
        if let &mut Section::Export(ref mut export_section) = section {
            return Some(export_section);
        }
    }
    None
}

impl ExportWhitelist {
    /// Constructs an empty whitelist. Mostly useless.
    fn new() -> Self {
        ExportWhitelist {
            entries: Vec::new(),
        }
    }

    /// Constructs a whitelist with the ewasm export interface preset.
    fn ewasm() -> Self {
        ExportWhitelist {
            entries: vec![
                //NOTE: function signatures are not checked yet
                ExportEntry::new("main".to_string(), Internal::Function(0)),
                ExportEntry::new("memory".to_string(), Internal::Memory(0)),
            ],
        }
    }

    /// Constructs a whitelist with the parity wasm export interface preset.
    fn pwasm() -> Self {
        ExportWhitelist {
            entries: vec![ExportEntry::new("_call".to_string(), Internal::Function(0))],
        }
    }

    /// Looks up a given export entry in the whitelist and returns true if it is valid.
    fn lookup(&mut self, export: &ExportEntry) -> bool {
        if let Some(thing) = self.entries.iter().find(|matched_export| {
            export.field() == matched_export.field()
                && cmp_internal_variant(export.internal(), matched_export.internal())
        }) {
            true
        } else {
            false
        }
    }
}

impl TrimExports {
    /// Constructs an empty `trimexports` context.
    fn new() -> Self {
        TrimExports {
            whitelist: ExportWhitelist::new(),
        }
    }

    /// Takes a given preset string and constructs a context with the
    /// corresponding whitelist.
    pub fn with_preset(preset: &str) -> Self {
        match preset {
            "ewasm" => TrimExports {
                whitelist: ExportWhitelist::ewasm(),
            },
            "pwasm" => TrimExports {
                whitelist: ExportWhitelist::pwasm(),
            },
            _ => TrimExports::new(),
        }
    }

    /// Iterates over the export section, if there is one, and removes
    /// unnecessary entries.
    fn trim_exports(&mut self, module: &mut Module) -> bool {
        if let Some(section) = export_section_mut(module) {
            let new_section = ExportSection::with_entries(
                section
                    .entries()
                    .iter()
                    .cloned()
                    .filter(|entry| self.whitelist.lookup(entry))
                    .collect(),
            );

            if new_section.entries().len() < section.entries().len() {
                *section = new_section;
                return true;
            }

            false
        } else {
            false
        }
    }
}

impl ModuleTranslator for TrimExports {
    fn translate(mut self, module: &mut Module) -> Result<bool, String> {
        Ok(self.trim_exports(module))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_wasm::builder;

    // Smoke tests
    #[test]
    fn builder_all_exports_good_ewasm() {
        let mut module = builder::module()
            .function()
            .signature()
            .build()
            .body()
            .build()
            .build()
            .export()
            .field("main")
            .internal()
            .func(0)
            .build()
            .export()
            .field("memory")
            .internal()
            .memory(0)
            .build()
            .build();

        let trimmer = TrimExports::with_preset("ewasm");
        let did_change = trimmer.translate(&mut module).unwrap();
        assert_eq!(false, did_change);
    }

    #[test]
    fn builder_one_wrong_mem_export_ewasm() {
        let mut module = builder::module()
            .function()
            .signature()
            .build()
            .body()
            .build()
            .build()
            .export()
            .field("main")
            .internal()
            .func(0)
            .build()
            .export()
            .field("memory")
            .internal()
            .memory(0)
            .build()
            .export()
            .field("foo")
            .internal()
            .memory(0)
            .build()
            .build();

        let trimmer = TrimExports::with_preset("ewasm");
        let did_change = trimmer.translate(&mut module).unwrap();
        assert_eq!(true, did_change);
    }

    #[test]
    fn builder_no_export_ewasm() {
        let mut module = builder::module()
            .function()
            .signature()
            .build()
            .body()
            .build()
            .build()
            .build();

        let trimmer = TrimExports::with_preset("ewasm");
        let did_change = trimmer.translate(&mut module).unwrap();
        assert_eq!(false, did_change);
    }

    #[test]
    fn builder_all_exports_good_pwasm() {
        let mut module = builder::module()
            .function()
            .signature()
            .build()
            .body()
            .build()
            .build()
            .export()
            .field("_call")
            .internal()
            .func(0)
            .build()
            .build();

        let trimmer = TrimExports::with_preset("pwasm");
        let did_change = trimmer.translate(&mut module).unwrap();
        assert_eq!(false, did_change);
    }
}
