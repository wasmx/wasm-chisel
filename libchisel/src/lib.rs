extern crate parity_wasm;

use parity_wasm::elements::*;

pub trait ModuleCreator {
    fn create(self) -> Result<Module, String>;
}

pub trait ModuleTranslator {
    fn translate(self, module: &mut Module) -> Result<(), String>;
}

pub trait ModuleValidator {
    fn validate(self, module: & Module) -> Result<bool, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SampleCreator {
    }

    impl ModuleCreator for SampleCreator {
        fn create(self) -> Result<Module, String> {
            Ok(Module::default())
        }
    }

    #[test]
    fn creator_succeeds() {
        let creator = SampleCreator{};
        let result = creator.create();
        assert!(result.is_ok());
    }
}
