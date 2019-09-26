use crate::ChiselModule;
use crate::ModuleError;
use crate::ModuleKind;

use crate::checkfloat::CheckFloat;

// Primitive module server, relies on huge matcher
pub fn get_module(name: &str) -> Result<Box<dyn ChiselModule>, ModuleError> {
    match name {
        "checkfloat" => Ok(Box::new(CheckFloat::new() as dyn ChiselModule)),
        _ => Err(ModuleError::NotFound),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_checkfloat() {
        let module = get_module("checkfloat").expect("Cannot fail");

        assert_eq!(module.kind(), ModuleKind::Validator);
    }
}
