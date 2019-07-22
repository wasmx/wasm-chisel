use super::{imports::ImportList, ModuleError, ModulePreset, ModuleTranslator};

use parity_wasm::elements::*;

pub struct RemapImports<'a> {
    /// A list of import sets to remap.
    interfaces: Vec<ImportInterface<'a>>,
}

/// A pair containing a list of imports for RemapImports to remap against, and an optional string with which all
/// imports are expected to be prefixed.
pub struct ImportInterface<'a>(ImportList<'a>, Option<&'a str>);

impl<'a> ModulePreset for RemapImports<'a> {
    fn with_preset(preset: &str) -> Result<Self, ()> {
        let mut interface_set: Vec<ImportInterface> = Vec::new();

        // Accept a comma-separated list of presets.
        let presets: String = preset
            .chars()
            .filter(|c| *c != '_' && *c != ' ' && *c != '\n' && *c != '\t')
            .collect();
        for preset_individual in presets.split(',') {
            match preset_individual {
                "ewasm" => interface_set.push(ImportInterface::new(
                    ImportList::with_preset("ewasm").expect("Missing ewasm preset"),
                    Some("ethereum_"),
                )),
                "eth2" => interface_set.push(ImportInterface::new(
                    ImportList::with_preset("eth2").expect("Missing eth2 preset"),
                    Some("eth2_"),
                )),
                "debug" => interface_set.push(ImportInterface::new(
                    ImportList::with_preset("debug").expect("Missing debug preset"),
                    Some("debug_"),
                )),
                "bignum" => interface_set.push(ImportInterface::new(
                    ImportList::with_preset("bignum").expect("Missing bignum preset"),
                    Some("bignum_"),
                )),
                _ => return Err(()),
            }
        }

        Ok(RemapImports {
            interfaces: interface_set,
        })
    }
}

impl<'a> ModuleTranslator for RemapImports<'a> {
    fn translate_inplace(&self, module: &mut Module) -> Result<bool, ModuleError> {
        let mut was_mutated = false;

        if let Some(section) = module.import_section_mut() {
            for interface in self.interfaces.iter() {
                *section = ImportSection::with_entries(
                    section
                        .entries()
                        .iter()
                        .map(|e| self.remap_from_list(e, &mut was_mutated, interface))
                        .collect(),
                );
            }
        }

        Ok(was_mutated)
    }

    fn translate(&self, module: &Module) -> Result<Option<Module>, ModuleError> {
        let mut new_module = module.clone();
        let mut was_mutated = false;

        if let Some(section) = new_module.import_section_mut() {
            // Iterate over entries and remap if needed.
            for interface in self.interfaces.iter() {
                *section = ImportSection::with_entries(
                    section
                        .entries()
                        .iter()
                        .map(|e| self.remap_from_list(e, &mut was_mutated, interface))
                        .collect(),
                );
            }
        }

        if was_mutated {
            Ok(Some(new_module))
        } else {
            Ok(None)
        }
    }
}

impl<'a> ImportInterface<'a> {
    pub fn new(imports: ImportList<'a>, prefix: Option<&'a str>) -> Self {
        ImportInterface(imports, prefix)
    }

    pub fn prefix(&self) -> Option<&str> {
        self.1
    }

    pub fn imports(&self) -> &ImportList<'a> {
        &self.0
    }
}

impl<'a> RemapImports<'a> {
    fn new(interfaces: Vec<ImportInterface<'a>>) -> Self {
        RemapImports {
            interfaces: interfaces,
        }
    }

    /// Takes an import entry and returns either the same entry or a remapped version if it exists.
    /// Sets the mutation flag if was remapped.
    fn remap_from_list(
        &self,
        entry: &ImportEntry,
        mutflag: &mut bool,
        interface: &ImportInterface,
    ) -> ImportEntry {
        match interface.prefix() {
            Some(prefix) => {
                let prefix_len = prefix.len();
                if entry.field().len() > prefix_len && prefix == &entry.field()[..prefix_len] {
                    // Look for a matching remappable import and mutate if found.
                    if let Some(import) = interface
                        .imports()
                        .lookup_by_field(&entry.field()[prefix_len..])
                    {
                        *mutflag = true;
                        return ImportEntry::new(
                            import.module().into(),
                            import.field().into(),
                            entry.external().clone(),
                        );
                    }
                }
                entry.clone()
            }
            None => {
                if let Some(import) = interface.imports().lookup_by_field(&entry.field()) {
                    *mutflag = true;
                    ImportEntry::new(
                        import.module().into(),
                        import.field().into(),
                        entry.external().clone(),
                    )
                } else {
                    entry.clone()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verifyimports::*;
    use crate::{ModulePreset, ModuleTranslator, ModuleValidator};
    use parity_wasm;
    use rustc_hex::FromHex;

    #[test]
    fn smoke_test() {
        let input = FromHex::from_hex(
            "
            0061736d0100000001050160017e0002170103656e760f65746865726575
            6d5f7573654761730000
        ",
        )
        .unwrap();
        let mut module = parity_wasm::deserialize_buffer(&input).expect("failed");
        let did_change = RemapImports::with_preset("ewasm")
            .unwrap()
            .translate_inplace(&mut module)
            .unwrap();
        let output = parity_wasm::serialize(module).expect("failed");
        let expected = FromHex::from_hex(
            "
            0061736d0100000001050160017e0002130108657468657265756d067573
            654761730000
        ",
        )
        .unwrap();
        assert_eq!(output, expected);
        assert!(did_change);
    }

    #[test]
    fn remap_did_mutate() {
        // wast:
        // (module
        //   (import "env" "ethereum_useGas" (func (param i64)))
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x60, 0x01, 0x7e,
            0x00, 0x60, 0x00, 0x00, 0x02, 0x17, 0x01, 0x03, 0x65, 0x6e, 0x76, 0x0f, 0x65, 0x74,
            0x68, 0x65, 0x72, 0x65, 0x75, 0x6d, 0x5f, 0x75, 0x73, 0x65, 0x47, 0x61, 0x73, 0x00,
            0x00, 0x03, 0x02, 0x01, 0x01, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x11, 0x02, 0x04,
            0x6d, 0x61, 0x69, 0x6e, 0x00, 0x01, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02,
            0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = parity_wasm::deserialize_buffer(&wasm).unwrap();

        let new = RemapImports::with_preset("ewasm")
            .unwrap()
            .translate(&module)
            .expect("Module internal error");

        assert!(new.is_some());
    }

    #[test]
    fn remap_did_mutate_verify() {
        // wast:
        // (module
        //   (import "env" "ethereum_useGas" (func (param i64)))
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x60, 0x01, 0x7e,
            0x00, 0x60, 0x00, 0x00, 0x02, 0x17, 0x01, 0x03, 0x65, 0x6e, 0x76, 0x0f, 0x65, 0x74,
            0x68, 0x65, 0x72, 0x65, 0x75, 0x6d, 0x5f, 0x75, 0x73, 0x65, 0x47, 0x61, 0x73, 0x00,
            0x00, 0x03, 0x02, 0x01, 0x01, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x11, 0x02, 0x04,
            0x6d, 0x61, 0x69, 0x6e, 0x00, 0x01, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02,
            0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = parity_wasm::deserialize_buffer(&wasm).unwrap();

        let new = RemapImports::with_preset("ewasm")
            .unwrap()
            .translate(&module)
            .expect("Module internal error");

        assert!(new.is_some());

        let verified = VerifyImports::with_preset("ewasm")
            .unwrap()
            .validate(&new.unwrap())
            .unwrap();

        assert_eq!(verified, true);
    }

    #[test]
    fn remap_did_mutate_verify_explicit_type_section() {
        // wast:
        // (module
        //   (type (;0;) (func (result i64)))
        //   (import "env" "ethereum_getGasLeft" (func (;0;) (type 0)))
        //   (memory 1)
        //   (func $main)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        // )

        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x60, 0x00, 0x01,
            0x7e, 0x60, 0x00, 0x00, 0x02, 0x1b, 0x01, 0x03, 0x65, 0x6e, 0x76, 0x13, 0x65, 0x74,
            0x68, 0x65, 0x72, 0x65, 0x75, 0x6d, 0x5f, 0x67, 0x65, 0x74, 0x47, 0x61, 0x73, 0x4c,
            0x65, 0x66, 0x74, 0x00, 0x00, 0x03, 0x02, 0x01, 0x01, 0x05, 0x03, 0x01, 0x00, 0x01,
            0x07, 0x11, 0x02, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x01, 0x06, 0x6d, 0x65, 0x6d,
            0x6f, 0x72, 0x79, 0x02, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = parity_wasm::deserialize_buffer(&wasm).unwrap();

        let new = RemapImports::with_preset("ewasm")
            .unwrap()
            .translate(&module)
            .expect("Module internal error");

        assert!(new.is_some());

        let verified = VerifyImports::with_preset("ewasm")
            .unwrap()
            .validate(&new.unwrap())
            .unwrap();

        assert_eq!(verified, true);
    }

    #[test]
    fn remap_mutated_multiple_interfaces() {
        // wast:
        // (module
        //   (type (;0;) (func (result i64)))
        //   (type (;1;) (func (param i32 i32 i32)))
        //   (type (;2;) (func (param i32)))
        //   (import "env" "ethereum_getGasLeft" (func (;0;) (type 0)))
        //   (import "env" "bignum_mul256" (func (;1;) (type 1)))
        //   (import "env" "debug_printStorage" (func (;2;) (type 2)))
        //   (memory 1)
        //   (func $main)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        // )

        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x12, 0x04, 0x60, 0x00, 0x01,
            0x7e, 0x60, 0x03, 0x7f, 0x7f, 0x7f, 0x00, 0x60, 0x01, 0x7f, 0x00, 0x60, 0x00, 0x00,
            0x02, 0x48, 0x03, 0x03, 0x65, 0x6e, 0x76, 0x13, 0x65, 0x74, 0x68, 0x65, 0x72, 0x65,
            0x75, 0x6d, 0x5f, 0x67, 0x65, 0x74, 0x47, 0x61, 0x73, 0x4c, 0x65, 0x66, 0x74, 0x00,
            0x00, 0x03, 0x65, 0x6e, 0x76, 0x0d, 0x62, 0x69, 0x67, 0x6e, 0x75, 0x6d, 0x5f, 0x6d,
            0x75, 0x6c, 0x32, 0x35, 0x36, 0x00, 0x01, 0x03, 0x65, 0x6e, 0x76, 0x12, 0x64, 0x65,
            0x62, 0x75, 0x67, 0x5f, 0x70, 0x72, 0x69, 0x6e, 0x74, 0x53, 0x74, 0x6f, 0x72, 0x61,
            0x67, 0x65, 0x00, 0x02, 0x03, 0x02, 0x01, 0x03, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07,
            0x11, 0x02, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x03, 0x06, 0x6d, 0x65, 0x6d, 0x6f,
            0x72, 0x79, 0x02, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = parity_wasm::deserialize_buffer(&wasm).unwrap();

        let new = RemapImports::with_preset("ewasm, bignum, debug")
            .unwrap()
            .translate(&module)
            .expect("Module internal error")
            .expect("Module was not mutated");

        let verifier = VerifyImports::with_preset("ewasm, bignum, debug").unwrap();

        assert_eq!(verifier.validate(&new), Ok(true));
    }

    #[test]
    fn no_prefix() {
        // wast:
        // (module
        //   (type (;0;) (func (result i64)))
        //   (type (;1;) (func (param i32 i32 i32)))
        //   (type (;2;) (func (param i32)))
        //   (import "env" "getGasLeft" (func (;0;) (type 0)))
        //   (import "env" "mul256" (func (;1;) (type 1)))
        //   (import "env" "printStorage" (func (;2;) (type 2)))
        //   (memory 1)
        //   (func $main)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        // )

        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x12, 0x04, 0x60, 0x00, 0x01,
            0x7e, 0x60, 0x03, 0x7f, 0x7f, 0x7f, 0x00, 0x60, 0x01, 0x7f, 0x00, 0x60, 0x00, 0x00,
            0x02, 0x32, 0x03, 0x03, 0x65, 0x6e, 0x76, 0x0a, 0x67, 0x65, 0x74, 0x47, 0x61, 0x73,
            0x4c, 0x65, 0x66, 0x74, 0x00, 0x00, 0x03, 0x65, 0x6e, 0x76, 0x06, 0x6d, 0x75, 0x6c,
            0x32, 0x35, 0x36, 0x00, 0x01, 0x03, 0x65, 0x6e, 0x76, 0x0c, 0x70, 0x72, 0x69, 0x6e,
            0x74, 0x53, 0x74, 0x6f, 0x72, 0x61, 0x67, 0x65, 0x00, 0x02, 0x03, 0x02, 0x01, 0x03,
            0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x11, 0x02, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00,
            0x03, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x0a, 0x04, 0x01, 0x02,
            0x00, 0x0b,
        ];

        let module = parity_wasm::deserialize_buffer(&wasm).unwrap();

        let interfaces_noprefix = vec![
            ImportInterface::new(ImportList::with_preset("ewasm").unwrap(), None),
            ImportInterface::new(ImportList::with_preset("bignum").unwrap(), None),
            ImportInterface::new(ImportList::with_preset("debug").unwrap(), None),
        ];

        let new = RemapImports::new(interfaces_noprefix)
            .translate(&module)
            .expect("Module internal error")
            .expect("Module was not mutated");

        let verifier = VerifyImports::with_preset("ewasm, bignum, debug").unwrap();

        assert_eq!(verifier.validate(&new), Ok(true));
    }
}
