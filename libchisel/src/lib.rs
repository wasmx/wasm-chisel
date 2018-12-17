extern crate byteorder;
extern crate parity_wasm;
extern crate rustc_hex;

pub mod checkstartfunc;
pub mod deployer;
pub mod remapimports;
pub mod trimexports;
pub mod verifyexports;
pub mod verifyimports;

use parity_wasm::elements::*;

pub trait ModuleCreator {
    fn create(&self) -> Result<Module, String>;
}

pub trait ModuleTranslator {
    fn translate(&self, module: &Module) -> Result<Module, String>;
    fn translate_inplace(&self, module: &mut Module) -> Result<bool, String>;
}

pub trait ModuleValidator {
    fn validate(&self, module: &Module) -> Result<bool, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SampleModule {}

    impl ModuleCreator for SampleModule {
        fn create(&self) -> Result<Module, String> {
            Ok(Module::default())
        }
    }

    impl ModuleTranslator for SampleModule {
        fn translate(&self, module: &Module) -> Result<Module, String> {
            Ok(Module::default())
        }
        fn translate_inplace(&self, module: &mut Module) -> Result<bool, String> {
            Ok((true))
        }
    }

    impl ModuleValidator for SampleModule {
        fn validate(&self, module: &Module) -> Result<bool, String> {
            Ok(true)
        }
    }

    #[test]
    fn creator_succeeds() {
        let creator = SampleModule {};
        let result = creator.create();
        assert!(result.is_ok());
    }

    #[test]
    fn translator_succeeds() {
        let translator = SampleModule {};
        let result = translator.translate(&Module::default());
        assert!(result.is_ok());
    }

    #[test]
    fn translator_inplace_succeeds() {
        let translator = SampleModule {};
        let result = translator.translate_inplace(&mut Module::default());
        assert!(result.is_ok());
    }

    #[test]
    fn validator_succeeds() {
        let validator = SampleModule {};
        let result = validator.validate(&Module::default());
        assert!(result.is_ok());
    }
}
