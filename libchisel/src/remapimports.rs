use std::collections::HashMap;

use super::{ModuleError, ModulePreset, ModuleTranslator};
use parity_wasm::elements::*;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct ImportPair {
    pub module: String,
    pub field: String,
}

impl ImportPair {
    fn new(module: &str, field: &str) -> ImportPair {
        ImportPair {
            module: module.to_string(),
            field: field.to_string(),
        }
    }
}

#[derive(Default)]
pub struct Translations {
    translations: HashMap<ImportPair, ImportPair>,
}

impl ModulePreset for Translations {
    fn with_preset(preset: &str) -> Result<Self, ()> {
        match preset {
            "ewasm" => {
                let trans: HashMap<ImportPair, ImportPair> = [
                    (
                        ImportPair::new("env", "ethereum_useGas"),
                        ImportPair::new("ethereum", "useGas"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_getGasLeft"),
                        ImportPair::new("ethereum", "getGasLeft"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_getAddress"),
                        ImportPair::new("ethereum", "getAddress"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_getBlockHash"),
                        ImportPair::new("ethereum", "getBlockHash"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_getBlockCoinbase"),
                        ImportPair::new("ethereum", "getBlockCoinbase"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_getBlockDifficulty"),
                        ImportPair::new("ethereum", "getBlockDifficulty"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_getBlockGasLimit"),
                        ImportPair::new("ethereum", "getBlockGasLimit"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_getBlockNumber"),
                        ImportPair::new("ethereum", "getBlockNumber"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_getBlockTimestamp"),
                        ImportPair::new("ethereum", "getBlockTimestamp"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_getExternalBalance"),
                        ImportPair::new("ethereum", "getExternalBalance"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_getTxGasPrice"),
                        ImportPair::new("ethereum", "getTxGasPrice"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_getTxOrigin"),
                        ImportPair::new("ethereum", "getTxOrigin"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_getCaller"),
                        ImportPair::new("ethereum", "getCaller"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_getCallDataSize"),
                        ImportPair::new("ethereum", "getCallDataSize"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_getCallValue"),
                        ImportPair::new("ethereum", "getCallValue"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_callDataCopy"),
                        ImportPair::new("ethereum", "callDataCopy"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_getCodeSize"),
                        ImportPair::new("ethereum", "getCodeSize"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_getExternalCodeSize"),
                        ImportPair::new("ethereum", "getExternalCodeSize"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_externalCodeCopy"),
                        ImportPair::new("ethereum", "externalCodeCopy"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_codeCopy"),
                        ImportPair::new("ethereum", "codeCopy"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_getReturnDataSize"),
                        ImportPair::new("ethereum", "getReturnDataSize"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_returnDataCopy"),
                        ImportPair::new("ethereum", "returnDataCopy"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_create"),
                        ImportPair::new("ethereum", "create"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_call"),
                        ImportPair::new("ethereum", "call"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_callCode"),
                        ImportPair::new("ethereum", "callCode"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_callDelegate"),
                        ImportPair::new("ethereum", "callDelegate"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_callStatic"),
                        ImportPair::new("ethereum", "callStatic"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_storageLoad"),
                        ImportPair::new("ethereum", "storageLoad"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_log"),
                        ImportPair::new("ethereum", "log"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_storageStore"),
                        ImportPair::new("ethereum", "storageStore"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_revert"),
                        ImportPair::new("ethereum", "revert"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_finish"),
                        ImportPair::new("ethereum", "finish"),
                    ),
                    (
                        ImportPair::new("env", "ethereum_selfDestruct"),
                        ImportPair::new("ethereum", "selfDestruct"),
                    ),
                ]
                .iter()
                .cloned()
                .collect();
                Ok(Translations {
                    translations: trans,
                })
            }
            _ => Err(()),
        }
    }
}

impl Translations {
    fn get(&self, pair: &ImportPair) -> Option<&ImportPair> {
        self.translations.get(&pair)
    }
}

pub struct RemapImports {
    translations: Translations,
}

impl ModulePreset for RemapImports {
    fn with_preset(preset: &str) -> Result<Self, ()> {
        match preset {
            "ewasm" => Ok(RemapImports {
                translations: Translations::with_preset("ewasm").unwrap(),
            }),
            _ => Err(()),
        }
    }
}

impl ModuleTranslator for RemapImports {
    fn translate_inplace(&self, module: &mut Module) -> Result<bool, ModuleError> {
        Ok(rename_imports(module, &self.translations))
    }

    fn translate(&self, module: &Module) -> Result<Option<Module>, ModuleError> {
        let mut ret = module.clone();
        let modified = rename_imports(&mut ret, &self.translations);
        if modified {
            return Ok(Some(ret));
        }
        Ok(None)
    }
}

fn rename_imports(module: &mut Module, translations: &Translations) -> bool {
    let mut ret = false;
    if let Some(section) = module.import_section_mut() {
        for entry in section.entries_mut().iter_mut() {
            if let Some(replacement) =
                translations.get(&ImportPair::new(entry.module(), entry.field()))
            {
                ret = true;
                *entry = ImportEntry::new(
                    replacement.module.clone(),
                    replacement.field.clone(),
                    *entry.external(),
                )
            }
        }
    }
    ret
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
}
