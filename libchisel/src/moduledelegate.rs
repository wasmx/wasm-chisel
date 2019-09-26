//! Uniform chisel module handling abstraction.
//! This module implements get_module_delegate, an API function which takes a string and returns a closure
//! executing the module with the provided binary and configuration.

use std::collections::HashMap;

use crate::Module;
use crate::ModuleError;
use crate::ModuleKind;

/// The delegate type signature.
type ChiselDelegate = dyn Fn(&mut Module, &HashMap<String, String>) -> Result<bool, ModuleError>;

/// Create a matcher for get_module_delegate with the provided delegates.
/// NOTE: There must be a delegate of the same name declared in the delegates module.
macro_rules! delegate_matcher {
    ($name:ident; $( $module:ident ),*) => {
        match $name {
            $(stringify!($module) => Some(&crate::moduledelegate::delegates::$module)),*,
            _ => None,
        }
    }
}

/// Look up the module name and return the appropriate delegate function.
pub fn get_module_delegate(name: &str) -> Option<&'static ChiselDelegate> {
    delegate_matcher!(name; checkfloat, remapstart)
}

/// Delegate functions
mod delegates {
    #![allow(non_upper_case_globals)]
    use super::*;

    use crate::ModuleCreator;
    use crate::ModuleTranslator;
    use crate::ModuleValidator;

    /// Generate a delegate for a module with no config, which calls $method on the module.
    /// NOTE: currently only works with translate_inplace and validate
    macro_rules! __delegate_noconfig {
        ($delegate_name:ident, $module:ty, $method:ident) => {
            pub const $delegate_name: &'static ChiselDelegate = &|mut wasm, _| {
                let module = <$module>::new();
                module.$method(&mut wasm)
            };
        };
    }

    // TODO: integrate config api
    // TODO: accomodate functional-style validate()

    __delegate_noconfig!(checkfloat, crate::checkfloat::CheckFloat, validate);
    __delegate_noconfig!(remapstart, crate::remapstart::RemapStart, translate_inplace);

}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_hex::FromHex;

    #[test]
    fn get_validator() {
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

    #[test]
    fn get_translator() {
        let delegate = get_module_delegate("remapstart").expect("Cannot fail");

        let wasm: Vec<u8> = FromHex::from_hex(
            "0061736d0100000001080260017e0060
000002170103656e760f657468657265756d5f75736547617300000303020101050301000107110
2046d61696e0001066d656d6f727902000801020a070202000b02000b0020046e616d65010e0201
046d61696e02056d61696e320209030001000001000200",
        )
        .unwrap();

        let mut module = Module::from_bytes(&wasm).unwrap();
        let result = delegate(&mut module, &HashMap::new());
        assert_eq!(result.expect("Cannot fail"), true);
    }
}
