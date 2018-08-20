extern crate parity_wasm;
#[macro_use]
extern crate log;

use std::collections::HashMap;
use std::env;

use parity_wasm::elements::*;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct ImportPair {
    pub module: String,
    pub field: String
}

impl ImportPair {
    fn new(module: &str, field: &str) -> ImportPair {
        ImportPair { module: module.to_string(), field: field.to_string() }
    }
}

#[derive(Default)]
pub struct Translations {
   translations: HashMap<ImportPair, ImportPair>
}

impl Translations {
    fn ewasm() -> Translations {
        let trans: HashMap<ImportPair, ImportPair> =
            [
                (ImportPair::new("env", "ethereum_useGas"), ImportPair::new("ethereum", "useGas")),
                (ImportPair::new("env", "ethereum_getGasLeft"), ImportPair::new("ethereum", "getGasLeft")),
                (ImportPair::new("env", "ethereum_getAddress"), ImportPair::new("ethereum", "getAddress")),
                (ImportPair::new("env", "ethereum_getBalance"), ImportPair::new("ethereum", "getBalance")),
                (ImportPair::new("env", "ethereum_getTxGasPrice"), ImportPair::new("ethereum", "getTxGasPrice")),
                (ImportPair::new("env", "ethereum_getTxOrigin"), ImportPair::new("ethereum", "getTxOrigin")),
                (ImportPair::new("env", "ethereum_getCaller"), ImportPair::new("ethereum", "getCaller")),
                (ImportPair::new("env", "ethereum_getCallDataSize"), ImportPair::new("ethereum", "getCallDataSize")),
                (ImportPair::new("env", "ethereum_callDataCopy"), ImportPair::new("ethereum", "callDataCopy")),
                (ImportPair::new("env", "ethereum_getCodeSize"), ImportPair::new("ethereum", "getCodeSize")),
                (ImportPair::new("env", "ethereum_codeCopy"), ImportPair::new("ethereum", "codeCopy")),
                (ImportPair::new("env", "ethereum_getReturnDataSize"), ImportPair::new("ethereum", "getReturnDataSize")),
                (ImportPair::new("env", "ethereum_returnDataCopy"), ImportPair::new("ethereum", "returnDataCopy")),
                (ImportPair::new("env", "ethereum_call"), ImportPair::new("ethereum", "call")),
                (ImportPair::new("env", "ethereum_callCode"), ImportPair::new("ethereum", "callCode")),
                (ImportPair::new("env", "ethereum_callDelegate"), ImportPair::new("ethereum", "callDelegate")),
                (ImportPair::new("env", "ethereum_callStatic"), ImportPair::new("ethereum", "callStatic")),
                (ImportPair::new("env", "ethereum_storageLoad"), ImportPair::new("ethereum", "storageLoad")),
                (ImportPair::new("env", "ethereum_storageStore"), ImportPair::new("ethereum", "storageStore")),
                (ImportPair::new("env", "ethereum_revert"), ImportPair::new("ethereum", "revert")),
                (ImportPair::new("env", "ethereum_finish"), ImportPair::new("ethereum", "finish")),
                (ImportPair::new("env", "ethereum_selfDestruct"), ImportPair::new("ethereum", "selfDestruct")),
            ].iter().cloned().collect();
        Translations { translations: trans }
    }

    fn insert(&mut self, from_module: &str, from_field: &str, to_module: &str, to_field: &str) {
        self.translations.insert(ImportPair::new(from_module, from_field), ImportPair::new(to_module, to_field));
    }

//    fn get_simple(&self, module: &str, field: &str) -> Option<&str, &str> {
//        if let Some(translation) = self.translations.get(&ImportPair::new(module, field)) {
//            Some(translation.module.clone(), translation.field.clone())
//        } else {
//            None
//        }
//    }

    fn get(&self, pair: &ImportPair) -> Option<&ImportPair> {
        self.translations.get(&pair)
    }
}

// FIXME: There is no `Module::import_section_mut()`
fn import_section_mut(module: &mut Module) -> Option<&mut ImportSection> {
    for section in module.sections_mut() {
        if let &mut Section::Import(ref mut import_section) = section { return Some(import_section); }
    }
    None
}

pub fn rename_imports(module: &mut Module, translations: Translations) {
    if let Some(section) = import_section_mut(module) {
        for entry in section.entries_mut().iter_mut() {
            if let Some(replacement) = translations.get(&ImportPair::new(entry.module(), entry.field())) {
                *entry = ImportEntry::new(replacement.module.clone(), replacement.field.clone(), *entry.external())
            }
        }
    }
}

pub fn cleanup() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 3 {
        println!("Usage: {} in.wasm out.wasm", args[0]);
        return;
    }

    let module = parity_wasm::deserialize_file(&args[1]).expect("Failed to load module");

    if let Some(section) = module.function_section() {
        for (i, entry) in section.entries().iter().enumerate() {
            debug!("function {:?}", i);
        }
    }

    if let Some(section) = module.code_section() {
        for (i, entry) in section.bodies().iter().enumerate() {
            for opcode in entry.code().elements() {
              debug!("opcode {:?}", opcode)
              // iterate opcodes..
            }
        }
    }

    parity_wasm::serialize_to_file(&args[2], module).expect("Failed to write module");
}

#[cfg(test)]
mod tests {
    use parity_wasm;
    use std::collections::HashMap;
    use super::{ImportPair,Translations};

    #[test]
    fn smoke_test() {
        let mut module = parity_wasm::deserialize_file("src/test.wasm").expect("failed");
        ::rename_imports(&mut module, Translations::ewasm());
        parity_wasm::serialize_to_file("src/test-out.wasm", module).expect("failed");
    }
}
