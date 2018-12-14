use super::ModuleValidator;

use parity_wasm::elements::{External, FunctionType, ImportSection, Module, Type, ValueType};

/// Enum representing a type of import and any extra data to check.
#[derive(Clone)]
pub enum ImportType<'a> {
    Function(&'a str, &'a str, FunctionType),
    Global(&'a str, &'a str),
    Memory(&'a str, &'a str),
    Table(&'a str, &'a str),
}

/// Enum representing the state of an import in a module.
#[derive(PartialEq)]
pub enum ImportStatus {
    Good,
    NotFound,
    Malformed,
}

/// Trait over ImportType that lets a caller check if it is imported in a given module, and
/// verifies its type signature is correct.
trait IsImported {
    fn is_imported(&self, module: &Module) -> bool;
}

/// Trait over ImportType that checks an import's type signature in the case that it is imported.
trait ImportCheck {
    fn check(&self, module: &Module) -> ImportStatus;
}

/// Struct on which ModuleValidator is implemented.
pub struct VerifyImports<'a> {
    /// List of function signatures to check.
    entries: Vec<ImportType<'a>>,
    /// Option to require the presence of all listed imports in the module. When false, only the
    /// validity of existing imports on the list is checked.
    require_all: bool,
    /// Option to allow imports that are not listed in `entries`.
    allow_unlisted: bool,
}

impl<'a> VerifyImports<'a> {
    pub fn with_preset(preset: &str) -> Result<Self, ()> {
        match preset {
            "ewasm" => Ok(VerifyImports {
                //FIXME: It is messy to inline all the function signatures in the constructor.
                entries: vec![
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
                        FunctionType::new(
                            vec![ValueType::I64, ValueType::I32],
                            Some(ValueType::I32),
                        ),
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
                        FunctionType::new(
                            vec![ValueType::I32, ValueType::I32, ValueType::I32],
                            None,
                        ),
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
                        FunctionType::new(
                            vec![ValueType::I32, ValueType::I32, ValueType::I32],
                            None,
                        ),
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
                ]
                .iter()
                .cloned()
                .collect(),
                require_all: false,
                allow_unlisted: false,
            }),
            _ => Err(()),
        }
    }

    // Utility functions used in tests to get more coverage
    #[cfg(test)]
    fn set_require_all(&mut self, arg: bool) {
        self.require_all = arg;
    }

    #[cfg(test)]
    fn set_allow_unlisted(&mut self, arg: bool) {
        self.allow_unlisted = arg;
    }
}

impl<'a> ModuleValidator for VerifyImports<'a> {
    fn validate(self, module: &Module) -> Result<bool, String> {
        let import_section_len = if let Some(section) = module.import_section() {
            section.entries().len()
        } else {
            0
        };

        Ok(match (self.require_all, self.allow_unlisted) {
            // Check that all listed imports exist and are correct.
            (true, true) => self
                .entries
                .iter()
                .map(|e| e.is_imported(module))
                .find(|e| *e == false)
                .is_none(),
            // Check that all listed imports exist, are correct, and are the only imports in the
            // module.
            (true, false) => {
                self.entries
                    .iter()
                    .map(|e| e.is_imported(module))
                    .find(|e| *e == false)
                    .is_none()
                    && (self.entries.len() == import_section_len)
            }
            // Check that the imports which are both listed and imported are of correct type.
            (false, true) => self
                .entries
                .iter()
                .map(|e| e.check(module))
                .find(|e| *e == ImportStatus::Malformed)
                .is_none(),
            (false, false) => {
                // Check that all existent imports are listed and correct.
                let mut checklist: Vec<ImportStatus> =
                    self.entries.iter().map(|e| e.check(module)).collect();
                let valid_entries_count = checklist
                    .iter()
                    .filter(|e| **e == ImportStatus::Good)
                    .count();

                // Proof: If the number of valid entries is equal to the number of existing entries, all
                // entries are valid.
                //
                // If an import entry is valid, it exists; If the number of existent imports is
                // equal to the number of valid imports, all existent imports are valid; If all
                // existent imports are valid, there are no existent invalid imports; qed
                valid_entries_count == import_section_len
            }
        })
    }
}

impl<'a> IsImported for ImportType<'a> {
    fn is_imported(&self, module: &Module) -> bool {
        if let Some(section) = module.import_section() {
            match self {
                ImportType::Function(namespace, field, sig) => {
                    has_func_import(module, namespace, field, sig)
                }
                ImportType::Global(namespace, field) => {
                    has_global_import(section, namespace, field)
                }
                ImportType::Memory(namespace, field) => {
                    has_memory_import(section, namespace, field)
                }
                ImportType::Table(namespace, field) => has_table_import(section, namespace, field),
            }
        } else {
            false
        }
    }
}

impl<'a> ImportCheck for ImportType<'a> {
    fn check(&self, module: &Module) -> ImportStatus {
        // Destructure self here so that it is easier to manipulate individual fields later.
        let (module_str, field_str, func_sig) = match self {
            ImportType::Function(namespace, field, sig) => (namespace, field, Some(sig)),
            ImportType::Global(namespace, field) => (namespace, field, None),
            ImportType::Memory(namespace, field) => (namespace, field, None),
            ImportType::Table(namespace, field) => (namespace, field, None),
        };

        if let Some(section) = module.import_section() {
            // Find an entry that matches self. If the name matches, check the namespace and/or
            // signature.
            if let Some(entry) = section
                .entries()
                .iter()
                .find(|e| e.field() == *field_str && *module_str == e.module())
            {
                match entry.external() {
                    // TODO: Wrap this in a helper.
                    External::Function(idx) => {
                        if let Some(sig) = func_sig {
                            if *sig == imported_func_sig_by_index(module, *idx as usize) {
                                ImportStatus::Good
                            } else {
                                ImportStatus::Malformed
                            }
                        } else {
                            ImportStatus::Malformed
                        }
                    }
                    // NOTE: There may be a better way to do mappings between enum variants.
                    // Just check import variant here.
                    External::Global(_idx) => {
                        if let ImportType::Global(_n, _f) = self {
                            ImportStatus::Good
                        } else {
                            ImportStatus::Malformed
                        }
                    }
                    External::Memory(_idx) => {
                        if let ImportType::Memory(_n, _f) = self {
                            ImportStatus::Good
                        } else {
                            ImportStatus::Malformed
                        }
                    }
                    External::Table(_idx) => {
                        if let ImportType::Table(_n, _f) = self {
                            ImportStatus::Good
                        } else {
                            ImportStatus::Malformed
                        }
                    }
                }
            } else {
                ImportStatus::NotFound
            }
        } else {
            ImportStatus::NotFound
        }
    }
}

fn has_global_import(section: &ImportSection, namespace: &str, field: &str) -> bool {
    if let Some(import) = section
        .entries()
        .iter()
        .find(|e| e.module() == namespace && e.field() == field)
    {
        match import.external() {
            External::Global(_globaltype) => true,
            _ => false,
        }
    } else {
        false
    }
}

fn has_memory_import(section: &ImportSection, namespace: &str, field: &str) -> bool {
    if let Some(import) = section
        .entries()
        .iter()
        .find(|e| e.module() == namespace && e.field() == field)
    {
        match import.external() {
            External::Memory(_memorytype) => true,
            _ => false,
        }
    } else {
        false
    }
}

fn has_table_import(section: &ImportSection, namespace: &str, field: &str) -> bool {
    if let Some(import) = section
        .entries()
        .iter()
        .find(|e| e.module() == namespace && e.field() == field)
    {
        match import.external() {
            External::Table(_tabletype) => true,
            _ => false,
        }
    } else {
        false
    }
}

fn has_func_import(module: &Module, namespace: &str, field: &str, sig: &FunctionType) -> bool {
    if let Some(section) = module.import_section() {
        if let Some(import) = section
            .entries()
            .iter()
            .find(|e| e.module() == namespace && e.field() == field)
        {
            match import.external() {
                External::Function(index) => {
                    imported_func_sig_by_index(module, *index as usize) == *sig
                }
                _ => false,
            }
        } else {
            false
        }
    } else {
        false
    }
}

/// Resolves an imported function's signature from its callable index.
pub fn imported_func_sig_by_index(module: &Module, index: usize) -> FunctionType {
    let import_section = module.import_section().expect("No function section found");
    let type_section = module.type_section().expect("No type section found");

    let func_type_ref: usize = match import_section.entries()[index].external() {
        &External::Function(idx) => idx as usize,
        _ => usize::max_value(),
    };

    match type_section.types()[func_type_ref] {
        Type::Function(ref func_type) => func_type.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_wasm::elements::deserialize_buffer;

    #[test]
    fn no_imports_ok_ewasm() {
        // wast:
        // (module
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x11, 0x02, 0x04, 0x6d,
            0x61, 0x69, 0x6e, 0x00, 0x00, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00,
            0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyImports::with_preset("ewasm").unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(true, result);
    }

    #[test]
    fn one_import_ok_ewasm() {
        // wast:
        // (module
        //   (import "ethereum" "storageStore" (func $storageStore (param i32 i32)))
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x09, 0x02, 0x60, 0x02, 0x7f,
            0x7f, 0x00, 0x60, 0x00, 0x00, 0x02, 0x19, 0x01, 0x08, 0x65, 0x74, 0x68, 0x65, 0x72,
            0x65, 0x75, 0x6d, 0x0c, 0x73, 0x74, 0x6f, 0x72, 0x61, 0x67, 0x65, 0x53, 0x74, 0x6f,
            0x72, 0x65, 0x00, 0x00, 0x03, 0x02, 0x01, 0x01, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07,
            0x11, 0x02, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x01, 0x06, 0x6d, 0x65, 0x6d, 0x6f,
            0x72, 0x79, 0x02, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyImports::with_preset("ewasm").unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(true, result);
    }

    #[test]
    fn one_import_bad_sig_ewasm() {
        // wast:
        // (module
        //   (import "ethereum" "storageStore" (func $storageStore (param i32)))
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x60, 0x01, 0x7f,
            0x00, 0x60, 0x00, 0x00, 0x02, 0x19, 0x01, 0x08, 0x65, 0x74, 0x68, 0x65, 0x72, 0x65,
            0x75, 0x6d, 0x0c, 0x73, 0x74, 0x6f, 0x72, 0x61, 0x67, 0x65, 0x53, 0x74, 0x6f, 0x72,
            0x65, 0x00, 0x00, 0x03, 0x02, 0x01, 0x01, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x11,
            0x02, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x01, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72,
            0x79, 0x02, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyImports::with_preset("ewasm").unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn one_import_bad_namespace_ewasm() {
        // wast:
        // (module
        //   (import "env" "storageStore" (func $storageStore (param i32 i32)))
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x09, 0x02, 0x60, 0x02, 0x7f,
            0x7f, 0x00, 0x60, 0x00, 0x00, 0x02, 0x14, 0x01, 0x03, 0x65, 0x6e, 0x76, 0x0c, 0x73,
            0x74, 0x6f, 0x72, 0x61, 0x67, 0x65, 0x53, 0x74, 0x6f, 0x72, 0x65, 0x00, 0x00, 0x03,
            0x02, 0x01, 0x01, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x11, 0x02, 0x04, 0x6d, 0x61,
            0x69, 0x6e, 0x00, 0x01, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x0a,
            0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyImports::with_preset("ewasm").unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn one_import_bad_field_ewasm() {
        // wast:
        // (module
        //   (import "ethereum" "stoageStore" (func $storageStore (param i32 i32)))
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x09, 0x02, 0x60, 0x02, 0x7f,
            0x7f, 0x00, 0x60, 0x00, 0x00, 0x02, 0x18, 0x01, 0x08, 0x65, 0x74, 0x68, 0x65, 0x72,
            0x65, 0x75, 0x6d, 0x0b, 0x73, 0x74, 0x6f, 0x61, 0x67, 0x65, 0x53, 0x74, 0x6f, 0x72,
            0x65, 0x00, 0x00, 0x03, 0x02, 0x01, 0x01, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x11,
            0x02, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x01, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72,
            0x79, 0x02, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyImports::with_preset("ewasm").unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn state_test_case_good_ewasm() {
        // wast:
        // (module
        // (import "ethereum" "storageStore" (func $storageStore (param i32 i32)))
        //   (memory 1)
        //   (data (i32.const 0)  "\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00") ;; Path
        //   (data (i32.const 32) "\cd\ab\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00") ;; Value
        //   (export "memory" (memory 0))
        //   (export "main" (func $main))
        //   (func $main
        //     ;; Write to storage
        //     (call $storageStore (i32.const 0) (i32.const 32))
        //   )
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x09, 0x02, 0x60, 0x02, 0x7f,
            0x7f, 0x00, 0x60, 0x00, 0x00, 0x02, 0x19, 0x01, 0x08, 0x65, 0x74, 0x68, 0x65, 0x72,
            0x65, 0x75, 0x6d, 0x0c, 0x73, 0x74, 0x6f, 0x72, 0x61, 0x67, 0x65, 0x53, 0x74, 0x6f,
            0x72, 0x65, 0x00, 0x00, 0x03, 0x02, 0x01, 0x01, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07,
            0x11, 0x02, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x04, 0x6d, 0x61,
            0x69, 0x6e, 0x00, 0x01, 0x0a, 0x0a, 0x01, 0x08, 0x00, 0x41, 0x00, 0x41, 0x20, 0x10,
            0x00, 0x0b, 0x0b, 0x4b, 0x02, 0x00, 0x41, 0x00, 0x0b, 0x20, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x41, 0x20, 0x0b, 0x20, 0xcd, 0xab, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyImports::with_preset("ewasm").unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(true, result);
    }

    #[test]
    fn unlisted_import_eth_namespace_ewasm() {
        // wast:
        // (module
        //   (import "ethereum" "storageStore" (func $storageStore (param i32 i32)))
        //   (import "ethereum" "foo" (func $foo))
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x09, 0x02, 0x60, 0x02, 0x7f,
            0x7f, 0x00, 0x60, 0x00, 0x00, 0x02, 0x2b, 0x02, 0x08, 0x65, 0x74, 0x68, 0x65, 0x72,
            0x65, 0x75, 0x6d, 0x0c, 0x73, 0x74, 0x6f, 0x72, 0x61, 0x67, 0x65, 0x53, 0x74, 0x6f,
            0x72, 0x65, 0x00, 0x00, 0x08, 0x65, 0x74, 0x68, 0x65, 0x72, 0x65, 0x75, 0x6d, 0x06,
            0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72, 0x00, 0x01, 0x03, 0x02, 0x01, 0x01, 0x05, 0x03,
            0x01, 0x00, 0x01, 0x07, 0x11, 0x02, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x02, 0x06,
            0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyImports::with_preset("ewasm").unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn unlisted_import_eth_namespace_good_ewasm() {
        // wast:
        // (module
        //   (import "ethereum" "storageStore" (func $storageStore (param i32 i32)))
        //   (import "ethereum" "foo" (func $foo))
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x09, 0x02, 0x60, 0x02, 0x7f,
            0x7f, 0x00, 0x60, 0x00, 0x00, 0x02, 0x2b, 0x02, 0x08, 0x65, 0x74, 0x68, 0x65, 0x72,
            0x65, 0x75, 0x6d, 0x0c, 0x73, 0x74, 0x6f, 0x72, 0x61, 0x67, 0x65, 0x53, 0x74, 0x6f,
            0x72, 0x65, 0x00, 0x00, 0x08, 0x65, 0x74, 0x68, 0x65, 0x72, 0x65, 0x75, 0x6d, 0x06,
            0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72, 0x00, 0x01, 0x03, 0x02, 0x01, 0x01, 0x05, 0x03,
            0x01, 0x00, 0x01, 0x07, 0x11, 0x02, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x02, 0x06,
            0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let mut checker = VerifyImports::with_preset("ewasm").unwrap();
        // Allow unlisted, just for this test case
        checker.set_allow_unlisted(true);
        let result = checker.validate(&module).unwrap();
        assert_eq!(true, result);
    }

    #[test]
    fn one_import_but_all_required_ewasm() {
        // wast:
        // (module
        //   (import "ethereum" "storageStore" (func $storageStore (param i32 i32)))
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x09, 0x02, 0x60, 0x02, 0x7f,
            0x7f, 0x00, 0x60, 0x00, 0x00, 0x02, 0x19, 0x01, 0x08, 0x65, 0x74, 0x68, 0x65, 0x72,
            0x65, 0x75, 0x6d, 0x0c, 0x73, 0x74, 0x6f, 0x72, 0x61, 0x67, 0x65, 0x53, 0x74, 0x6f,
            0x72, 0x65, 0x00, 0x00, 0x03, 0x02, 0x01, 0x01, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07,
            0x11, 0x02, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x01, 0x06, 0x6d, 0x65, 0x6d, 0x6f,
            0x72, 0x79, 0x02, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let mut checker = VerifyImports::with_preset("ewasm").unwrap();
        // Require all, just for this test case
        checker.set_require_all(true);
        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn all_required_imports() {
        // wast:
        // (module
        //   (import "ethereum" "storageStore" (func $storageStore (param i32 i32)))
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x09, 0x02, 0x60, 0x02, 0x7f,
            0x7f, 0x00, 0x60, 0x00, 0x00, 0x02, 0x19, 0x01, 0x08, 0x65, 0x74, 0x68, 0x65, 0x72,
            0x65, 0x75, 0x6d, 0x0c, 0x73, 0x74, 0x6f, 0x72, 0x61, 0x67, 0x65, 0x53, 0x74, 0x6f,
            0x72, 0x65, 0x00, 0x00, 0x03, 0x02, 0x01, 0x01, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07,
            0x11, 0x02, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x01, 0x06, 0x6d, 0x65, 0x6d, 0x6f,
            0x72, 0x79, 0x02, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyImports {
            entries: vec![ImportType::Function(
                "ethereum",
                "storageStore",
                FunctionType::new(vec![ValueType::I32, ValueType::I32], None),
            )]
            .iter()
            .cloned()
            .collect(),
            allow_unlisted: false,
            require_all: true,
        };
        let result = checker.validate(&module).unwrap();
        assert_eq!(true, result);
    }

    #[test]
    fn all_required_imports_but_one_unlisted_same_namespace_ok() {
        // wast:
        // (module
        //   (import "ethereum" "storageStore" (func $storageStore (param i32 i32)))
        //   (import "ethereum" "foo" (func $foo))
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x09, 0x02, 0x60, 0x02, 0x7f,
            0x7f, 0x00, 0x60, 0x00, 0x00, 0x02, 0x28, 0x02, 0x08, 0x65, 0x74, 0x68, 0x65, 0x72,
            0x65, 0x75, 0x6d, 0x0c, 0x73, 0x74, 0x6f, 0x72, 0x61, 0x67, 0x65, 0x53, 0x74, 0x6f,
            0x72, 0x65, 0x00, 0x00, 0x08, 0x65, 0x74, 0x68, 0x65, 0x72, 0x65, 0x75, 0x6d, 0x03,
            0x66, 0x6f, 0x6f, 0x00, 0x01, 0x03, 0x02, 0x01, 0x01, 0x05, 0x03, 0x01, 0x00, 0x01,
            0x07, 0x11, 0x02, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x02, 0x06, 0x6d, 0x65, 0x6d,
            0x6f, 0x72, 0x79, 0x02, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyImports {
            entries: vec![ImportType::Function(
                "ethereum",
                "storageStore",
                FunctionType::new(vec![ValueType::I32, ValueType::I32], None),
            )]
            .iter()
            .cloned()
            .collect(),
            allow_unlisted: true,
            require_all: true,
        };
        let result = checker.validate(&module).unwrap();
        assert_eq!(true, result);
    }

    #[test]
    fn all_required_imports_but_one_unlisted_same_namespace() {
        // wast:
        // (module
        //   (import "ethereum" "storageStore" (func $storageStore (param i32 i32)))
        //   (import "ethereum" "foo" (func $foo))
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x09, 0x02, 0x60, 0x02, 0x7f,
            0x7f, 0x00, 0x60, 0x00, 0x00, 0x02, 0x28, 0x02, 0x08, 0x65, 0x74, 0x68, 0x65, 0x72,
            0x65, 0x75, 0x6d, 0x0c, 0x73, 0x74, 0x6f, 0x72, 0x61, 0x67, 0x65, 0x53, 0x74, 0x6f,
            0x72, 0x65, 0x00, 0x00, 0x08, 0x65, 0x74, 0x68, 0x65, 0x72, 0x65, 0x75, 0x6d, 0x03,
            0x66, 0x6f, 0x6f, 0x00, 0x01, 0x03, 0x02, 0x01, 0x01, 0x05, 0x03, 0x01, 0x00, 0x01,
            0x07, 0x11, 0x02, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x02, 0x06, 0x6d, 0x65, 0x6d,
            0x6f, 0x72, 0x79, 0x02, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyImports {
            entries: vec![ImportType::Function(
                "ethereum",
                "storageStore",
                FunctionType::new(vec![ValueType::I32, ValueType::I32], None),
            )]
            .iter()
            .cloned()
            .collect(),
            allow_unlisted: false,
            require_all: true,
        };
        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn all_required_imports_but_one_unlisted_diff_namespace() {
        // wast:
        // (module
        //   (import "ethereum" "storageStore" (func $storageStore (param i32 i32)))
        //   (import "env" "foo" (func $foo))
        //   (memory 1)
        //   (export "main" (func $main))
        //   (export "memory" (memory 0))
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x09, 0x02, 0x60, 0x02, 0x7f,
            0x7f, 0x00, 0x60, 0x00, 0x00, 0x02, 0x23, 0x02, 0x08, 0x65, 0x74, 0x68, 0x65, 0x72,
            0x65, 0x75, 0x6d, 0x0c, 0x73, 0x74, 0x6f, 0x72, 0x61, 0x67, 0x65, 0x53, 0x74, 0x6f,
            0x72, 0x65, 0x00, 0x00, 0x03, 0x65, 0x6e, 0x76, 0x03, 0x66, 0x6f, 0x6f, 0x00, 0x01,
            0x03, 0x02, 0x01, 0x01, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x11, 0x02, 0x04, 0x6d,
            0x61, 0x69, 0x6e, 0x00, 0x02, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00,
            0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyImports {
            entries: vec![ImportType::Function(
                "ethereum",
                "storageStore",
                FunctionType::new(vec![ValueType::I32, ValueType::I32], None),
            )]
            .iter()
            .cloned()
            .collect(),
            allow_unlisted: false,
            require_all: true,
        };
        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

}
