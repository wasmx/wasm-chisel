use super::{ModuleCreator, ModuleError};
use crate::utils::*;
use parity_wasm::elements::{deserialize_buffer, Module};
use wabt::Wat2Wasm;

/// Struct on which ModuleCreator is implemented.
pub struct FromWat<'a> {
    filename: &'a str,
}

impl<'a> FromWat<'a> {
    pub fn new(filename: &'a str) -> Result<Self, ()> {
        Ok(FromWat { filename: filename })
    }
}

fn load_file(filename: &str) -> Result<Vec<u8>, ModuleError> {
    use std::fs::File;
    use std::io::prelude::*;
    let mut file = File::open(filename)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

impl From<wabt::Error> for ModuleError {
    fn from(error: wabt::Error) -> Self {
        use std::error::Error;
        ModuleError::Custom(error.description().to_string())
    }
}

fn convert_wat(input: &[u8]) -> Result<Module, ModuleError> {
    // TODO: turn on relocatable(true)?
    let module = Wat2Wasm::new()
        .canonicalize_lebs(true)
        .write_debug_names(true)
        .convert(&input)?;

    let module = deserialize_buffer::<Module>(module.as_ref())?;
    Ok(module)
}

impl<'a> ModuleCreator for FromWat<'a> {
    fn create(&self) -> Result<Module, ModuleError> {
        let input = load_file(&self.filename)?;
        convert_wat(&input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_wasm::elements::*;

    #[test]
    fn smoke() {
        let expectation: Vec<u8> = vec![
            0, 97, 115, 109, 1, 0, 0, 0, 0, 8, 4, 110, 97, 109, 101, 2, 1, 0,
        ];

        let module = convert_wat(&r#"(module)"#.as_bytes()).unwrap();

        let result = serialize::<Module>(module).unwrap();
        assert_eq!(result, expectation);
    }

    #[test]
    fn simple_function() {
        let expectation: Vec<u8> = vec![
            0, 97, 115, 109, 1, 0, 0, 0, 1, 9, 2, 96, 2, 127, 127, 0, 96, 0, 0, 2, 19, 1, 8, 101,
            116, 104, 101, 114, 101, 117, 109, 6, 102, 105, 110, 105, 115, 104, 0, 0, 3, 2, 1, 1,
            7, 8, 1, 4, 109, 97, 105, 110, 0, 1, 10, 10, 1, 8, 0, 65, 0, 65, 0, 16, 0, 11, 0, 27,
            4, 110, 97, 109, 101, 1, 9, 1, 0, 6, 102, 105, 110, 105, 115, 104, 2, 9, 2, 0, 2, 0, 0,
            1, 0, 1, 0,
        ];

        let module = convert_wat(
            &r#"
      (module
        (import "ethereum" "finish" (func $finish (param i32 i32)))
        (func (export "main")
          (call $finish (i32.const 0) (i32.const 0))
        )
      )
      "#
            .as_bytes(),
        )
        .unwrap();

        let result = serialize::<Module>(module).unwrap();
        assert_eq!(result, expectation);
    }
}
