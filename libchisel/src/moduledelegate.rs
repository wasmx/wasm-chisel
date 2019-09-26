use std::collections::HashMap;

use crate::checkfloat::CheckFloat;
use crate::Module;
use crate::ModuleError;
use crate::ModuleKind;
use crate::ModuleValidator;

type ChiselDelegate = dyn Fn(&mut Module, &HashMap<String, String>) -> Result<bool, ModuleError>;

// Primitive module server, relies on huge matcher
pub fn get_module_delegate(name: &str) -> Result<&'static ChiselDelegate, ModuleError> {
    match name {
        "checkfloat" => Ok(&|wasm, _config| {
            let module = CheckFloat::new();
            module.validate(&wasm)
        }),
        _ => Err(ModuleError::NotFound),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_checkfloat() {
        let delegate = get_module_delegate("checkfloat").expect("Cannot fail");

        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x07, 0x01, 0x60, 0x02, 0x7c,
            0x7c, 0x01, 0x7c, 0x03, 0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64,
            0x00, 0x00, 0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0xa0, 0x0b,
        ];
        let mut module = Module::from_bytes(&wasm).unwrap();

        let result = delegate(&mut module, &HashMap::new());

        assert_eq!(result.expect("Cannot fail"), false);
    }
}
