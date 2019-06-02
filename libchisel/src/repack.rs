use super::{ModuleError, ModuleTranslator};
use parity_wasm::builder;
use parity_wasm::elements::*;

pub struct Repack;

impl Repack {
    pub fn new() -> Self {
        Repack {}
    }
}

impl ModuleTranslator for Repack {
    fn translate_inplace(&self, module: &mut Module) -> Result<bool, ModuleError> {
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
    use super::*;
    use parity_wasm::builder;

    #[test]
    fn smoke_test() {
        let mut module = Module::default();

        let repack = Repack::new();
        assert_eq!(module, repack.translate(&module).unwrap().unwrap());
    }

    #[test]
    fn basic_sections_only() {
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
}
