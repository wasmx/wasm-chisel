use std::collections::HashMap;

use parity_wasm::elements::{Instruction, Module};

use super::{ChiselModule, ModuleError, ModuleKind, ModuleValidator};

/// Struct on which ModuleValidator is implemented.
pub struct CheckFloat {}

impl<'a> ChiselModule<'a> for CheckFloat {
    type ObjectReference = &'a dyn ModuleValidator;

    fn id(&'a self) -> String {
        "checkfloat".to_string()
    }

    fn kind(&'a self) -> ModuleKind {
        ModuleKind::Validator
    }

    fn as_abstract(&'a self) -> Self::ObjectReference {
        self as Self::ObjectReference
    }

    fn with_defaults() -> Result<Self, ModuleError> {
        Ok(CheckFloat {})
    }

    fn with_config(_config: &HashMap<String, String>) -> Result<Self, ModuleError> {
        Err(ModuleError::NotSupported)
    }
}

impl ModuleValidator for CheckFloat {
    // NOTE: this will not check for SIMD instructions.
    fn validate(&self, module: &Module) -> Result<bool, ModuleError> {
        let code_section = module.code_section();
        if code_section.is_none() {
            return Err(ModuleError::NotFound);
        }
        for function in code_section.unwrap().bodies() {
            for instruction in function.code().elements() {
                match instruction {
                    Instruction::F32Eq
                    | Instruction::F32Ne
                    | Instruction::F32Lt
                    | Instruction::F32Gt
                    | Instruction::F32Le
                    | Instruction::F32Ge
                    | Instruction::F32Abs
                    | Instruction::F32Neg
                    | Instruction::F32Ceil
                    | Instruction::F32Floor
                    | Instruction::F32Trunc
                    | Instruction::F32Nearest
                    | Instruction::F32Sqrt
                    | Instruction::F32Add
                    | Instruction::F32Sub
                    | Instruction::F32Mul
                    | Instruction::F32Div
                    | Instruction::F32Min
                    | Instruction::F32Max
                    | Instruction::F32Copysign
                    | Instruction::I32TruncSF32
                    | Instruction::I32TruncUF32
                    | Instruction::I64TruncSF32
                    | Instruction::I64TruncUF32
                    | Instruction::F32ConvertSI32
                    | Instruction::F32ConvertUI32
                    | Instruction::F32ConvertSI64
                    | Instruction::F32ConvertUI64
                    | Instruction::F32DemoteF64
                    | Instruction::F64PromoteF32
                    | Instruction::I32ReinterpretF32
                    | Instruction::F32ReinterpretI32
                    | Instruction::F64Eq
                    | Instruction::F64Ne
                    | Instruction::F64Lt
                    | Instruction::F64Gt
                    | Instruction::F64Le
                    | Instruction::F64Ge
                    | Instruction::F64Abs
                    | Instruction::F64Neg
                    | Instruction::F64Ceil
                    | Instruction::F64Floor
                    | Instruction::F64Trunc
                    | Instruction::F64Nearest
                    | Instruction::F64Sqrt
                    | Instruction::F64Add
                    | Instruction::F64Sub
                    | Instruction::F64Mul
                    | Instruction::F64Div
                    | Instruction::F64Min
                    | Instruction::F64Max
                    | Instruction::F64Copysign
                    | Instruction::I32TruncSF64
                    | Instruction::I32TruncUF64
                    | Instruction::I64TruncSF64
                    | Instruction::I64TruncUF64
                    | Instruction::F64ConvertSI32
                    | Instruction::F64ConvertUI32
                    | Instruction::F64ConvertSI64
                    | Instruction::F64ConvertUI64
                    | Instruction::I64ReinterpretF64
                    | Instruction::F64ReinterpretI64
                    | Instruction::F32Const(_)
                    | Instruction::F32Load(_, _)
                    | Instruction::F32Store(_, _)
                    | Instruction::F64Const(_)
                    | Instruction::F64Load(_, _)
                    | Instruction::F64Store(_, _) => {
                        return Ok(false);
                    }
                    _ => {}
                }
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use parity_wasm::builder;

    use super::*;

    #[test]
    fn add_i32_no_fp() {
        //  (module
        //    (func $add (param $lhs i32) (param $rhs i32) (result i32)
        //      get_local $lhs
        //      get_local $rhs
        //      i32.add)
        //    (export "add" (func $add))
        //  )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x07, 0x01, 0x60, 0x02, 0x7f,
            0x7f, 0x01, 0x7f, 0x03, 0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64,
            0x00, 0x00, 0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b,
        ];
        let module = Module::from_bytes(&wasm).unwrap();
        let checker = CheckFloat::with_defaults().unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(true, result);
    }

    #[test]
    fn add_f32_fp() {
        //  (module
        //    (func $add (param $lhs f32) (param $rhs f32) (result f32)
        //      get_local $lhs
        //      get_local $rhs
        //      f32.add)
        //    (export "add" (func $add))
        //  )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x07, 0x01, 0x60, 0x02, 0x7d,
            0x7d, 0x01, 0x7d, 0x03, 0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64,
            0x00, 0x00, 0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x92, 0x0b,
        ];
        let module = Module::from_bytes(&wasm).unwrap();
        let checker = CheckFloat::with_defaults().unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn add_f64_fp() {
        //  (module
        //    (func $add (param $lhs f64) (param $rhs f64) (result f64)
        //      get_local $lhs
        //      get_local $rhs
        //      f64.add)
        //    (export "add" (func $add))
        //  )
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x07, 0x01, 0x60, 0x02, 0x7c,
            0x7c, 0x01, 0x7c, 0x03, 0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64,
            0x00, 0x00, 0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0xa0, 0x0b,
        ];
        let module = Module::from_bytes(&wasm).unwrap();
        let checker = CheckFloat::with_defaults().unwrap();
        let result = checker.validate(&module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn no_code_section() {
        let module = builder::module().build();
        let checker = CheckFloat::with_defaults().unwrap();
        let result = checker.validate(&module);
        assert_eq!(true, result.is_err());
        assert_eq!(result.err().unwrap(), ModuleError::NotFound)
    }
}
