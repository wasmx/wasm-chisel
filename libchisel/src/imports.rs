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

impl<'a> ImportType<'a> {
    pub fn module(&self) -> &'a str {
        // FIXME: Is there a way to shorten this expression?
        match self {
            ImportType::Function(module, _, _) => module,
            ImportType::Global(module, _)
            | ImportType::Memory(module, _)
            | ImportType::Table(module, _) => module,
        }
    }

    pub fn field(&self) -> &'a str {
        // FIXME: Is there a way to shorten this expression?
        match self {
            ImportType::Function(_, field, _) => field,
            ImportType::Global(_, field)
            | ImportType::Memory(_, field)
            | ImportType::Table(_, field) => field,
        }
    }

    pub fn signature(&self) -> Result<&FunctionType, ()> {
        match self {
            ImportType::Function(_, _, sig) => Ok(&sig),
            _ => Err(()),
        }
    }
}

impl<'a> ImportList<'a> {
    pub fn new() -> Self {
        ImportList(Vec::new())
    }

    pub fn entries(&'a self) -> &'a Vec<ImportType<'a>> {
        &self.0
    }

    pub fn entries_mut(&'a mut self) -> &'a mut Vec<ImportType<'a>> {
        &mut self.0
    }

    pub fn into_inner(self) -> Vec<ImportType<'a>> {
        self.0
    }

    pub fn concatenate(&mut self, other: ImportList<'a>) {
        let mut to_append = other.into_inner();
        self.0.append(&mut to_append);
    }

    pub fn with_entries(entries: Vec<ImportType<'a>>) -> Self {
        ImportList(entries)
    }

    pub fn lookup_by_field(&self, name: &str) -> Option<&ImportType> {
        let entries = self.entries();

        for import in entries {
            if import.field() == name {
                return Some(&import);
            }
        }
        None
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
            "eth2" => Ok(ImportList(vec![
                ImportType::Function(
                    "eth2",
                    "loadPreStateRoot",
                    FunctionType::new(vec![ValueType::I32], None),
                ),
                ImportType::Function(
                    "eth2",
                    "blockDataSize",
                    FunctionType::new(vec![], Some(ValueType::I32)),
                ),
                ImportType::Function(
                    "eth2",
                    "blockDataCopy",
                    FunctionType::new(vec![ValueType::I32, ValueType::I32, ValueType::I32], None),
                ),
                ImportType::Function(
                    "eth2",
                    "savePostStateRoot",
                    FunctionType::new(vec![ValueType::I32], None),
                ),
                ImportType::Function(
                    "eth2",
                    "pushNewDeposit",
                    FunctionType::new(vec![ValueType::I32, ValueType::I32], None),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_by_field_ewasm_good() {
        let list = ImportList::with_preset("ewasm").unwrap();
        assert!(list.lookup_by_field("useGas").is_some());
    }

    #[test]
    fn lookup_by_field_ewasm_not_found() {
        let list = ImportList::with_preset("ewasm").unwrap();
        assert!(list.lookup_by_field("foo").is_none());
    }
}
