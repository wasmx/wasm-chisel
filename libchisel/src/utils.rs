//! These are helpers to be used internally.

use crate::ModuleError;

use parity_wasm::elements::{deserialize_buffer, serialize, Module};

pub trait SerializationHelpers {
    /// Deserialize bytecode to a Module.
    fn from_slice(input: &[u8]) -> Result<Module, ModuleError>;

    /// Serialize Module to bytecode. Serialization consumes the input.
    fn to_vec(self) -> Result<Vec<u8>, ModuleError>;
}

impl SerializationHelpers for Module {
    fn from_slice(input: &[u8]) -> Result<Self, ModuleError> {
        Ok(deserialize_buffer::<Module>(&input)?)
    }

    fn to_vec(self) -> Result<Vec<u8>, ModuleError> {
        Ok(serialize::<Module>(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_hex::FromHex;

    #[test]
    fn module_roundtrip() {
        let input = FromHex::from_hex(
            "0061736d01000000010401600000030201000405017001010105030100100619
037f01418080c0000b7f00418080c0000b7f00418080c0000b072503066d656d
6f727902000b5f5f686561705f6261736503010a5f5f646174615f656e640302
0a040102000b",
        )
        .unwrap();
        let module = Module::from_slice(&input).unwrap();
        let output = module.to_vec().unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn bytecode_has_names_section() {
        let input = FromHex::from_hex(
            "0061736d010000000104016000000303020000070801046d61696e00010a
0a020300010b040010000b0014046e616d65010d0200047465737401046d
61696e",
        )
        .unwrap();
        let module = Module::from_slice(&input).unwrap();
        assert_eq!(module.has_names_section(), true);
    }

    #[test]
    fn bytecode_has_no_names_section() {
        let input = FromHex::from_hex(
            "0061736d010000000104016000000303020000070801046d61696e00010a
0a020300010b040010000b",
        )
        .unwrap();
        let module = Module::from_slice(&input).unwrap();
        assert_eq!(module.has_names_section(), false);
    }

    #[test]
    fn test_serialize_error() {
        // The failure case.
        let module = parity_wasm::builder::module()
            .export()
            .field("invalid")
            .internal()
            .func(15)
            .build()
            .build();
        // Shouldn't this one fail due to the invalid function reference?
        assert_eq!(module.to_vec().is_ok(), true);

        // The success case.
        assert_eq!(Module::default().to_vec().is_ok(), true)
    }

    #[test]
    fn test_deserialize_error() {
        // The failure case.
        assert_eq!(Module::from_slice(&[0u8; 0]).is_ok(), false);

        // The success case.
        let input = FromHex::from_hex(
            "0061736d01000000010401600000030201000405017001010105030100100619
037f01418080c0000b7f00418080c0000b7f00418080c0000b072503066d656d
6f727902000b5f5f686561705f6261736503010a5f5f646174615f656e640302
0a040102000b",
        )
        .unwrap();
        assert_eq!(Module::from_slice(&input).is_ok(), true)
    }
}
