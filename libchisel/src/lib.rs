#[cfg(feature = "binaryen")]
extern crate binaryen;
extern crate parity_wasm;
extern crate rustc_hex;
#[cfg(feature = "wabt")]
pub extern crate wabt;

pub use parity_wasm::elements::Module;

use std::{error, fmt};

pub mod imports;

#[cfg(feature = "binaryen")]
pub mod binaryenopt;
pub mod checkfloat;
pub mod checkstartfunc;
pub mod deployer;
pub mod dropsection;
#[cfg(feature = "wabt")]
pub mod fromwat;
pub mod remapimports;
pub mod remapstart;
pub mod repack;
pub mod snip;
pub mod trimexports;
pub mod trimstartfunc;
pub mod verifyexports;
pub mod verifyimports;

mod depgraph;

#[derive(Eq, PartialEq, Debug)]
pub enum ModuleKind {
    Creator,
    Translator,
    Validator,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ModuleError {
    NotSupported,
    NotFound,
    Custom(String),
}

/// Utility interface for chisel modules.
pub trait ChiselModule<'a> {
    type ObjectReference: ?Sized;
    /// Returns the name of the chisel module.
    fn id(&'a self) -> String;

    fn kind(&'a self) -> ModuleKind;

    /// Borrows the instance as a trait object.
    fn as_abstract(&'a self) -> Self::ObjectReference;
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
    fn with_preset(preset: &str) -> Result<Self, ModuleError>
    where
        Self: std::marker::Sized;
}

impl From<String> for ModuleError {
    fn from(error: String) -> Self {
        ModuleError::Custom(error)
    }
}

impl From<std::io::Error> for ModuleError {
    fn from(error: std::io::Error) -> Self {
        use std::error::Error;
        ModuleError::Custom(error.description().to_string())
    }
}

// Also aliased as parity_wasm::SerializationError
impl From<parity_wasm::elements::Error> for ModuleError {
    fn from(a: parity_wasm::elements::Error) -> Self {
        use std::error::Error;
        ModuleError::Custom(a.description().to_string())
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

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    struct SampleModule {}

    impl ModuleCreator for SampleModule {
        fn create(&self) -> Result<Module, ModuleError> {
            Ok(Module::default())
        }
    }

    impl ModuleTranslator for SampleModule {
        fn translate(&self, _module: &Module) -> Result<Option<Module>, ModuleError> {
            Ok(Some(Module::default()))
        }
        fn translate_inplace(&self, _module: &mut Module) -> Result<bool, ModuleError> {
            Ok(true)
        }
    }

    impl ModuleValidator for SampleModule {
        fn validate(&self, _module: &Module) -> Result<bool, ModuleError> {
            Ok(true)
        }
    }

    impl<'a> ChiselModule<'a> for SampleModule {
        // Yes, it implements all the traits, but we will treat it as a validator when used as a
        // trait object for testing purposes.
        type ObjectReference = &'a dyn ModuleValidator;

        fn id(&'a self) -> String {
            "Sample".to_string()
        }

        fn kind(&'a self) -> ModuleKind {
            ModuleKind::Validator
        }

        fn as_abstract(&'a self) -> Self::ObjectReference {
            self as Self::ObjectReference
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

    #[test]
    fn opaque_module() {
        let validator = SampleModule {};
        assert_eq!(validator.id(), "Sample");

        let opaque: &dyn ChiselModule<ObjectReference = &dyn ModuleValidator> =
            &validator as &dyn ChiselModule<ObjectReference = &dyn ModuleValidator>;

        assert_eq!(opaque.kind(), ModuleKind::Validator);

        let as_trait: &dyn ModuleValidator = opaque.as_abstract();

        let result = as_trait.validate(&Module::default());
        assert!(result.is_ok());
    }
}
