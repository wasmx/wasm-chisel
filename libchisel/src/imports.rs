use super::ModulePreset;

use parity_wasm::elements::{FunctionType, ValueType};

pub struct ImportList<'a>(Vec<ImportType<'a>>);

/// Enum internally representing a type of import.
#[derive(Clone)]
pub enum ImportType<'a> {
    Function(&'a str, &'a str, FunctionType),
    Global(&'a str, &'a str),
    Memory(&'a str, &'a str),
    Table(&'a str, &'a str),
}

impl<'a> ImportList<'a> {
    pub fn entries(&'a self) -> &'a Vec<ImportType<'a>> {
        &self.0
    }

    pub fn with_entries(entries: Vec<ImportType<'a>>) -> Self {
        ImportList(entries)
    }
}

impl<'a> ModulePreset for ImportList<'a> {
    fn with_preset(preset: &str) -> Result<Self, ()>
    where
        Self: Sized,
    {
        match preset {
            "ewasm" => Ok(ImportList(vec![
                ImportType::Function(
                    "ethereum",
                    "useGas",
                    FunctionType::new(vec![ValueType::I64], None),
                ),
                ImportType::Function(
                    "ethereum",
                    "getGasLeft",
                    FunctionType::new(vec![], Some(ValueType::I64)),
                ),
                ImportType::Function(
                    "ethereum",
                    "getAddress",
                    FunctionType::new(vec![ValueType::I32], None),
                ),
                ImportType::Function(
                    "ethereum",
                    "getExternalBalance",
                    FunctionType::new(vec![ValueType::I32, ValueType::I32], None),
                ),
                ImportType::Function(
                    "ethereum",
                    "getBlockHash",
                    FunctionType::new(vec![ValueType::I64, ValueType::I32], Some(ValueType::I32)),
                ),
                ImportType::Function(
                    "ethereum",
                    "call",
                    FunctionType::new(
                        vec![
                            ValueType::I64,
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                        ],
                        Some(ValueType::I32),
                    ),
                ),
                ImportType::Function(
                    "ethereum",
                    "callCode",
                    FunctionType::new(
                        vec![
                            ValueType::I64,
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                        ],
                        Some(ValueType::I32),
                    ),
                ),
                ImportType::Function(
                    "ethereum",
                    "callDelegate",
                    FunctionType::new(
                        vec![
                            ValueType::I64,
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                        ],
                        Some(ValueType::I32),
                    ),
                ),
                ImportType::Function(
                    "ethereum",
                    "callStatic",
                    FunctionType::new(
                        vec![
                            ValueType::I64,
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                        ],
                        Some(ValueType::I32),
                    ),
                ),
                ImportType::Function(
                    "ethereum",
                    "create",
                    FunctionType::new(
                        vec![
                            ValueType::I64,
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                        ],
                        Some(ValueType::I32),
                    ),
                ),
                ImportType::Function(
                    "ethereum",
                    "callDataCopy",
                    FunctionType::new(vec![ValueType::I32, ValueType::I32, ValueType::I32], None),
                ),
                ImportType::Function(
                    "ethereum",
                    "getCallDataSize",
                    FunctionType::new(vec![], Some(ValueType::I32)),
                ),
                ImportType::Function(
                    "ethereum",
                    "getCodeSize",
                    FunctionType::new(vec![], Some(ValueType::I32)),
                ),
                ImportType::Function(
                    "ethereum",
                    "getExternalCodeSize",
                    FunctionType::new(vec![ValueType::I32], Some(ValueType::I32)),
                ),
                ImportType::Function(
                    "ethereum",
                    "externalCodeCopy",
                    FunctionType::new(
                        vec![
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                        ],
                        None,
                    ),
                ),
                ImportType::Function(
                    "ethereum",
                    "codeCopy",
                    FunctionType::new(vec![ValueType::I32, ValueType::I32, ValueType::I32], None),
                ),
                ImportType::Function(
                    "ethereum",
                    "getCaller",
                    FunctionType::new(vec![ValueType::I32], None),
                ),
                ImportType::Function(
                    "ethereum",
                    "getCallValue",
                    FunctionType::new(vec![ValueType::I32], None),
                ),
                ImportType::Function(
                    "ethereum",
                    "getBlockDifficulty",
                    FunctionType::new(vec![ValueType::I32], None),
                ),
                ImportType::Function(
                    "ethereum",
                    "getBlockCoinbase",
                    FunctionType::new(vec![ValueType::I32], None),
                ),
                ImportType::Function(
                    "ethereum",
                    "getBlockNumber",
                    FunctionType::new(vec![], Some(ValueType::I64)),
                ),
                ImportType::Function(
                    "ethereum",
                    "getBlockGasLimit",
                    FunctionType::new(vec![], Some(ValueType::I64)),
                ),
                ImportType::Function(
                    "ethereum",
                    "getBlockTimestamp",
                    FunctionType::new(vec![], Some(ValueType::I64)),
                ),
                ImportType::Function(
                    "ethereum",
                    "getTxGasPrice",
                    FunctionType::new(vec![ValueType::I32], None),
                ),
                ImportType::Function(
                    "ethereum",
                    "getTxOrigin",
                    FunctionType::new(vec![ValueType::I32], None),
                ),
                ImportType::Function(
                    "ethereum",
                    "storageStore",
                    FunctionType::new(vec![ValueType::I32, ValueType::I32], None),
                ),
                ImportType::Function(
                    "ethereum",
                    "storageLoad",
                    FunctionType::new(vec![ValueType::I32, ValueType::I32], None),
                ),
                ImportType::Function(
                    "ethereum",
                    "log",
                    FunctionType::new(
                        vec![
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                        ],
                        None,
                    ),
                ),
                ImportType::Function(
                    "ethereum",
                    "getReturnDataSize",
                    FunctionType::new(vec![], Some(ValueType::I32)),
                ),
                ImportType::Function(
                    "ethereum",
                    "returnDataCopy",
                    FunctionType::new(vec![ValueType::I32, ValueType::I32, ValueType::I32], None),
                ),
                ImportType::Function(
                    "ethereum",
                    "finish",
                    FunctionType::new(vec![ValueType::I32, ValueType::I32], None),
                ),
                ImportType::Function(
                    "ethereum",
                    "revert",
                    FunctionType::new(vec![ValueType::I32, ValueType::I32], None),
                ),
                ImportType::Function(
                    "ethereum",
                    "selfDestruct",
                    FunctionType::new(vec![ValueType::I32], None),
                ),
            ])),
            "debug" => Ok(ImportList(vec![
                ImportType::Function(
                    "debug",
                    "print32",
                    FunctionType::new(vec![ValueType::I32], None),
                ),
                ImportType::Function(
                    "debug",
                    "print64",
                    FunctionType::new(vec![ValueType::I64], None),
                ),
                ImportType::Function(
                    "debug",
                    "printMem",
                    FunctionType::new(vec![ValueType::I32, ValueType::I32], None),
                ),
                ImportType::Function(
                    "debug",
                    "printMemHex",
                    FunctionType::new(vec![ValueType::I32, ValueType::I32], None),
                ),
                ImportType::Function(
                    "debug",
                    "printStorage",
                    FunctionType::new(vec![ValueType::I32], None),
                ),
                ImportType::Function(
                    "debug",
                    "printStorageHex",
                    FunctionType::new(vec![ValueType::I32], None),
                ),
            ])),
            "bignum" => Ok(ImportList(vec![
                ImportType::Function(
                    "bignum",
                    "mul256",
                    FunctionType::new(vec![ValueType::I32, ValueType::I32, ValueType::I32], None),
                ),
                ImportType::Function(
                    "bignum",
                    "umulmod256",
                    FunctionType::new(
                        vec![
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                            ValueType::I32,
                        ],
                        None,
                    ),
                ),
            ])),
            _ => Err(()),
        }
    }
}
