use parity_wasm::builder;
use parity_wasm::elements::Module;

use super::{ChiselModule, ModuleError, ModuleKind, ModuleTranslator};

pub struct Repack;

impl Repack {
    pub fn new() -> Self {
        Repack {}
    }
}

impl<'a> ChiselModule<'a> for Repack {
    type ObjectReference = &'a dyn ModuleTranslator;

    fn id(&'a self) -> String {
        "repack".to_string()
    }

    fn kind(&'a self) -> ModuleKind {
        ModuleKind::Translator
    }

    fn as_abstract(&'a self) -> Self::ObjectReference {
        self as Self::ObjectReference
    }
}

impl ModuleTranslator for Repack {
    fn translate_inplace(&self, _module: &mut Module) -> Result<bool, ModuleError> {
        Err(ModuleError::NotSupported)
    }

    fn translate(&self, module: &Module) -> Result<Option<Module>, ModuleError> {
        // TODO: check in names section is carried over.
        let module = module.clone();
        let module = builder::from_module(module).build();
        Ok(Some(module))
    }
}

#[cfg(test)]
mod tests {
    use parity_wasm::builder;
    use parity_wasm::elements::CustomSection;
    use rustc_hex::FromHex;

    use super::*;

    #[test]
    fn smoke_test() {
        let module = Module::default();

        let repack = Repack::new();
        assert_eq!(module, repack.translate(&module).unwrap().unwrap());
    }

    #[test]
    fn basic_sections_only() {
        let module = builder::module()
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

        let repack = Repack::new();
        assert_eq!(module, repack.translate(&module).unwrap().unwrap());
    }

    #[test]
    fn custom_section() {
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

        let custom = CustomSection::new("test".to_string(), vec![42u8; 16]);
        module
            .sections_mut()
            .push(parity_wasm::elements::Section::Custom(custom));

        let repack = Repack::new();
        assert_ne!(module, repack.translate(&module).unwrap().unwrap());
    }

    #[test]
    fn names_section() {
        let input = FromHex::from_hex(
            "0061736d010000000104016000000303020000070801046d61696e00010a
0a020300010b040010000b0014046e616d65010d0200047465737401046d
61696e",
        )
        .unwrap();
        let module = Module::from_bytes(&input).unwrap();
        // Forcefully parse names section here.
        let module = module
            .parse_names()
            .expect("parsing the names section failed");
        assert_eq!(module.names_section().is_some(), true);
        let repack = Repack::new();
        // Repack drops names section too.
        let output = repack.translate(&module).unwrap().unwrap();
        assert_eq!(output.has_names_section(), false);
    }
}
