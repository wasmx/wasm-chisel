use std::collections::HashMap;

use parity_wasm::elements::Module;

use super::{ChiselModule, ModuleError, ModuleKind, ModuleTranslator};

// TODO: consider making this a generic helper?
fn check_bool_option(config: &HashMap<String, String>, option: &str, default: bool) -> bool {
    if let Some(value) = config.get(option) {
        value == "true"
    } else {
        default
    }
}

#[derive(Clone)]
pub struct Snip(wasm_snip::Options);

impl<'a> ChiselModule<'a> for Snip {
    type ObjectReference = &'a dyn ModuleTranslator;

    fn id(&'a self) -> String {
        "snip".to_string()
    }

    fn kind(&'a self) -> ModuleKind {
        ModuleKind::Translator
    }

    fn as_abstract(&'a self) -> Self::ObjectReference {
        self as Self::ObjectReference
    }

    fn with_defaults() -> Result<Self, ModuleError> {
        let mut options = wasm_snip::Options::default();
        // TODO: expose these as options
        options.snip_rust_fmt_code = true;
        options.snip_rust_panicking_code = true;
        options.skip_producers_section = true;
        Ok(Snip { 0: options })
    }

    fn with_config(config: &HashMap<String, String>) -> Result<Self, ModuleError> {
        let mut options = wasm_snip::Options::default();
        options.snip_rust_fmt_code = check_bool_option(&config, "snip_rust_fmt_code", true);
        options.snip_rust_panicking_code =
            check_bool_option(&config, "snip_rust_panicking_code", true);
        options.skip_producers_section = check_bool_option(&config, "skip_producers_section", true);
        Ok(Snip { 0: options })
    }
}

impl From<failure::Error> for ModuleError {
    fn from(error: failure::Error) -> Self {
        ModuleError::Custom(error.to_string())
    }
}

impl ModuleTranslator for Snip {
    fn translate_inplace(&self, _module: &mut Module) -> Result<bool, ModuleError> {
        Err(ModuleError::NotSupported)
    }

    fn translate(&self, module: &Module) -> Result<Option<Module>, ModuleError> {
        let serialized = module.clone().to_bytes()?;

        let mut input = walrus::Module::from_buffer(&serialized)?;
        wasm_snip::snip(&mut input, self.0.clone())?;
        let output = input.emit_wasm();

        let output = Module::from_bytes(&output[..])?;
        Ok(Some(output))
    }
}

#[cfg(test)]
mod tests {
    use rustc_hex::FromHex;

    use super::*;

    #[test]
    fn smoke_test() {
        // (module
        // (import "env" "ethereum_useGas" (func (param i64)))
        // (memory 1)
        // (export "main" (func $main))
        // (export "memory" (memory 0))
        // (func $main
        //     (call $std::panicking::rust_panic_with_hook::h12b7239ed4348eae)
        //     (call $core::fmt::write::h9f284ae8e8e9b94a)
        // )
        // (func $std::panicking::rust_panic_with_hook::h12b7239ed4348eae)
        // (func $core::fmt::write::h9f284ae8e8e9b94a)
        // )
        let wasm: Vec<u8> = FromHex::from_hex(
            "0061736d0100000001080260017e0060000002170103656e760f65746865
7265756d5f75736547617300000304030101010503010001071102046d61
696e0001066d656d6f727902000a10030600100210030b0300010b030001
0b007f046e616d650178040011696d706f72742466756e6374696f6e2430
01046d61696e02377374643a3a70616e69636b696e673a3a727573745f70
616e69635f776974685f686f6f6b3a3a6831326237323339656434333438
6561650323636f72653a3a666d743a3a77726974653a3a68396632383461
65386538653962393461",
        )
        .unwrap();

        let module = Module::from_bytes(&wasm).unwrap();
        let module = Snip::with_defaults().unwrap().translate(&module);
        let module = module
            .expect("translation to be succesful")
            .expect("new module to be returned");
        assert!(module.to_bytes().unwrap().len() < wasm.len());
    }
}
