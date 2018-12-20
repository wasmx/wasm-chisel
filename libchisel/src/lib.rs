extern crate byteorder;
extern crate parity_wasm;
extern crate rustc_hex;

use parity_wasm::elements::Module;

pub mod checkstartfunc;
pub mod deployer;
pub mod remapimports;
pub mod trimexports;
pub mod verifyexports;
pub mod verifyimports;

#[derive(Eq, PartialEq, Debug)]
pub enum ModuleError {
    NotSupported,
    Custom(String),
}

pub trait ModuleCreator {
    /// Returns new module.
    fn create(&self) -> Result<Module, ModuleError>;
}

pub trait ModuleTranslator {
    /// Translates module. Returns new module. Can fail with ModuleError::NotSupported.
    fn translate(&self, module: &Module) -> Result<Module, ModuleError>;

    /// Translates module in-place. Returns true if the module was modified. Can fail with ModuleError::NotSupported.
    fn translate_inplace(&self, module: &mut Module) -> Result<bool, ModuleError>;
}

pub trait ModuleValidator {
    /// Validates module. Returns true if it is valid or false if invalid.
    fn validate(&self, module: &Module) -> Result<bool, ModuleError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SampleModule {}

    impl ModuleCreator for SampleModule {
        fn create(&self) -> Result<Module, ModuleError> {
            Ok(Module::default())
        }
    }

    impl ModuleTranslator for SampleModule {
        fn translate(&self, module: &Module) -> Result<Module, ModuleError> {
            Ok(Module::default())
        }
        fn translate_inplace(&self, module: &mut Module) -> Result<bool, ModuleError> {
            Ok((true))
        }
    }

    impl ModuleValidator for SampleModule {
        fn validate(&self, module: &Module) -> Result<bool, ModuleError> {
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
