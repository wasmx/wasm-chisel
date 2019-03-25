extern crate byteorder;
extern crate parity_wasm;
extern crate rustc_hex;

use parity_wasm::elements::Module;

pub mod imports;

pub mod checkstartfunc;
pub mod deployer;
pub mod remapimports;
pub mod trimexports;
pub mod verifyexports;
pub mod verifyimports;

mod depgraph;

use std::{error, fmt};

#[derive(Eq, PartialEq, Debug)]
pub enum ModuleError {
    NotSupported,
    NotFound,
    Custom(String),
}

pub trait ModuleCreator {
    /// Returns new module.
    fn create(&self) -> Result<Module, ModuleError>;
}

pub trait ModuleTranslator {
    /// Translates module. Returns new module or none if nothing was modified. Can fail with ModuleError::NotSupported.
    fn translate(&self, module: &Module) -> Result<Option<Module>, ModuleError>;

    /// Translates module in-place. Returns true if the module was modified. Can fail with ModuleError::NotSupported.
    fn translate_inplace(&self, module: &mut Module) -> Result<bool, ModuleError>;
}

pub trait ModuleValidator {
    /// Validates module. Returns true if it is valid or false if invalid.
    fn validate(&self, module: &Module) -> Result<bool, ModuleError>;
}

pub trait ModulePreset {
    fn with_preset(preset: &str) -> Result<Self, ()>
    where
        Self: std::marker::Sized;
}

impl From<String> for ModuleError {
    fn from(error: String) -> Self {
        ModuleError::Custom(error)
    }
}

impl fmt::Display for ModuleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ModuleError::NotSupported => "Method unsupported",
                ModuleError::NotFound => "Not found",
                ModuleError::Custom(msg) => msg,
            }
        )
    }
}

impl error::Error for ModuleError {
    fn description(&self) -> &str {
        match self {
            ModuleError::NotSupported => "Method unsupported",
            ModuleError::NotFound => "Not found",
            ModuleError::Custom(msg) => msg,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::error::Error;

    struct SampleModule {}

    impl ModuleCreator for SampleModule {
        fn create(&self) -> Result<Module, ModuleError> {
            Ok(Module::default())
        }
    }

    impl ModuleTranslator for SampleModule {
        fn translate(&self, module: &Module) -> Result<Option<Module>, ModuleError> {
            Ok(Some(Module::default()))
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

    #[test]
    fn from_error() {
        let err: ModuleError = "custom message".to_string().into();
        assert_eq!(err, ModuleError::Custom("custom message".to_string()));
    }

    #[test]
    fn fmt_good() {
        // Add new tests for each enum variant here as they are implemented.
        let fmt_result_unsupported = format!("{}", ModuleError::NotSupported);
        assert_eq!("Method unsupported", fmt_result_unsupported);

        let fmt_result_custom = format!("{}", ModuleError::Custom("foo".to_string()));
        assert_eq!("foo", fmt_result_custom);
    }

    #[test]
    fn error_good() {
        // Add new tests for each enum variant here as they are implemented.
        let err_unsupported = ModuleError::NotSupported;
        let err_description_unsupported = err_unsupported.description();
        assert_eq!("Method unsupported", err_description_unsupported);

        let err_custom = ModuleError::Custom("bar".to_string());
        let err_description_custom = err_custom.description();
        assert_eq!("bar", err_description_custom);
    }
}
