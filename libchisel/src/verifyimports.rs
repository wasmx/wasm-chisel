use super::{
    imports::{ImportList, ImportType},
    ModuleError, ModulePreset, ModuleValidator,
};
use parity_wasm::elements::{External, FunctionType, ImportSection, Module, Type, ValueType};

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
    list: ImportList<'a>,
    /// Option to require the presence of all listed imports in the module. When false, only the
    /// validity of existing imports on the list is checked.
    require_all: bool,
    /// Option to allow imports that are not listed in `entries`.
    allow_unlisted: bool,
}

impl<'a> ModulePreset for VerifyImports<'a> {
    fn with_preset(preset: &str) -> Result<Self, ()> {
        match preset {
            "ewasm" => Ok(VerifyImports {
                list: ImportList::with_preset("ewasm").unwrap(),
                require_all: false,
                allow_unlisted: false,
            }),
            "pwasm" => Ok(VerifyImports {
                // From https://github.com/paritytech/parity-ethereum/blob/5ed25276635f66450925cba3081028a36de5150d/ethcore/wasm/src/env.rs
                entries: vec![
                    ImportType::Function(
                        "env",
                        "storage_read",
                        FunctionType::new(vec![ValueType::I32, ValueType:I32], None),
                    ),
                    ImportType::Function(
                        "env",
                        "storage_write",
                        FunctionType::new(vec![ValueType::I32, ValueType:I32], None),
                    ),
                    ImportType::Function(
                        "env",
                        "ret",
                        FunctionType::new(vec![ValueType::I32, ValueType:I32], None),
                    ),
                    ImportType::Function(
                        "env",
                        "gas",
                        FunctionType::new(vec![ValueType::I32], None),
                    ),
    "balance",
    "sender",
    "origin",
    "fetch_input",
    "input_length",
    "ccall",
    "dcall",
    "scall",
    "create",
    "balance",
    "blockhash",
    "blocknumber",
    "coinbase",
    "timestamp",
    "difficulty",
    "gaslimit",
    "address",
    "value",
    "suicide",
    "panic",
    "elog",
"abort"

                //FIXME: It is messy to inline all the function signatures in the constructor.
                entries: vec![
                    ImportType::Function(
                        "ethereum",
                        "useGas",
                        FunctionType::new(vec![ValueType::I64], None),
                    ),
            _ => Err(()),
        }
    }
}

// Utility functions used in tests to get more coverage
#[cfg(test)]
impl<'a> VerifyImports<'a> {
    fn set_require_all(&mut self, arg: bool) {
        self.require_all = arg;
    }

    fn set_allow_unlisted(&mut self, arg: bool) {
        self.allow_unlisted = arg;
    }
}

impl<'a> ModuleValidator for VerifyImports<'a> {
    fn validate(&self, module: &Module) -> Result<bool, ModuleError> {
        let import_section_len = if let Some(section) = module.import_section() {
            section.entries().len()
        } else {
            0
        };

        Ok(match (self.require_all, self.allow_unlisted) {
            // Check that all listed imports exist and are correct.
            (true, true) => self
                .list
                .entries()
                .iter()
                .map(|e| e.is_imported(module))
                .find(|e| *e == false)
                .is_none(),
            // Check that all listed imports exist, are correct, and are the only imports in the
            // module.
            (true, false) => {
                self.list
                    .entries()
                    .iter()
                    .map(|e| e.is_imported(module))
                    .find(|e| *e == false)
                    .is_none()
                    && (self.list.entries().len() == import_section_len)
            }
            // Check that the imports which are both listed and imported are of correct type.
            (false, true) => self
                .list
                .entries()
                .iter()
                .map(|e| e.check(module))
                .find(|e| *e == ImportStatus::Malformed)
                .is_none(),
            (false, false) => {
                // Check that all existent imports are listed and correct.
                let mut checklist: Vec<ImportStatus> = self
                    .list
                    .entries()
                    .iter()
                    .map(|e| e.check(module))
                    .collect();
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
                    External::Function(idx) => {
                        let sig = func_sig.expect("Function entry missing signature!");
                        if *sig == imported_func_sig_by_index(module, *idx as usize) {
                            ImportStatus::Good
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

    match type_section.types()[index] {
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
            list: ImportList::with_entries(vec![ImportType::Function(
                "ethereum",
                "storageStore",
                FunctionType::new(vec![ValueType::I32, ValueType::I32], None),
            )]),
            require_all: true,
            allow_unlisted: false,
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
            list: ImportList::with_entries(vec![ImportType::Function(
                "ethereum",
                "storageStore",
                FunctionType::new(vec![ValueType::I32, ValueType::I32], None),
            )]),
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
            list: ImportList::with_preset("ewasm").unwrap(),
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
            list: ImportList::with_preset("ewasm").unwrap(),
            allow_unlisted: false,
            require_all: true,
        };
        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn verify_with_dynamic_dispatch_before_imports_good() {
        // NOTE: This is important for binaries utilizing dynamic dispatch.
        // For example, rustc will place a type for vtable lookups before the imports in the type
        // section, causing OOB.
        // wast:
        // (module
        //   (type (;0;) (func (param i32 i32 i32) (result i32))) ; this is the problem
        //   (type (;1;) (func (param i64)))
        //
        //   (import "ethereum" "useGas" (func $useGas (type 1)))
        //
        //   (memory 1)
        //   (export "memory" (memory 0))
        //   (export "main" (func $main))
        //
        //   (func $main)
        // )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x0f, 0x03, 0x60, 0x03, 0x7f,
            0x7f, 0x7f, 0x01, 0x7f, 0x60, 0x01, 0x7e, 0x00, 0x60, 0x00, 0x00, 0x02, 0x13, 0x01,
            0x08, 0x65, 0x74, 0x68, 0x65, 0x72, 0x65, 0x75, 0x6d, 0x06, 0x75, 0x73, 0x65, 0x47,
            0x61, 0x73, 0x00, 0x01, 0x03, 0x02, 0x01, 0x02, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07,
            0x11, 0x02, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x04, 0x6d, 0x61,
            0x69, 0x6e, 0x00, 0x01, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();
        let checker = VerifyImports::with_preset("ewasm").unwrap();

        let result = checker.validate(&module).unwrap();

        assert_eq!(true, result);
    }
}
