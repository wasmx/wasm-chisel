use super::InstructionValidator;

use parity_wasm::elements::Instruction;
use parity_wasm::elements::Instruction::*;
use parity_wasm::elements::*;

use crate::instructionerrors::*;

pub const GET_INST: [Instruction; 2] = [GetGlobal(0), GetLocal(0)];

pub const I32_BINOP: [Instruction; 15] = [
    I32Add, I32Sub, I32Mul, I32DivS, I32DivU, I32RemS, I32RemU, I32And, I32Or, I32Xor, I32Shl,
    I32ShrS, I32ShrU, I32Rotl, I32Rotr,
];

pub const I64_BINOP: [Instruction; 15] = [
    I64Add, I64Sub, I64Mul, I64DivS, I64DivU, I64RemS, I64RemU, I64And, I64Or, I64Xor, I64Shl,
    I64ShrS, I64ShrU, I64Rotl, I64Rotr,
];

pub const F32_BINOP: [Instruction; 7] =
    [F32Add, F32Sub, F32Mul, F32Div, F32Min, F32Max, F32Copysign];

pub const F64_BINOP: [Instruction; 7] =
    [F64Add, F64Sub, F64Mul, F64Div, F64Min, F64Max, F64Copysign];

use self::Filter::*;
use std::mem::discriminant;

type Pop = Vec<ValueType>;
type Push = Vec<ValueType>;

struct Signature {
    pop: Pop,
    push: Push,
}

pub enum Filter {
    NumericInstructions,
    NoFilter,
}

/// Basic struct for validating modules
pub struct VerifyInstructions {
    filter: Filter,
    stack: Vec<ValueType>,
}

impl VerifyInstructions {
    pub fn new(filter: Filter) -> Self {
        VerifyInstructions {
            filter,
            stack: vec![],
        }
    }

    fn check_instructions(
        &mut self,
        module: &Module,
        body: &FuncBody,
        index: usize,
    ) -> Result<bool, InstructionError> {
        for instruction in body.code().elements() {
            if contains(instruction, &GET_INST)
                && !self.push_global_or_local(module, instruction, body, index)?
            {
                return Ok(false);
            }
            match self.filter {
                NumericInstructions => {
                    let signature = get_instruction_signature(instruction);
                    // if the instruction does not have a signature we are interested in, we continue
                    if signature.is_some()
                        && !self.validate_instruction(&signature.unwrap(), instruction)?
                    {
                        return Ok(false);
                    }
                }
                NoFilter => (), // TODO: do this
            };
        }
        Ok(true)
    }

    fn validate_instruction(
        &mut self,
        signature: &Signature,
        instruction: &Instruction,
    ) -> Result<bool, InstructionError> {
        for signature_value in &signature.pop {
            let value = self.stack.pop();
            match value {
                Some(stack_value) => {
                    if stack_value != *signature_value {
                        return Err(InstructionError::InvalidOperation(instruction.clone()));
                    }
                }
                None => return Err(InstructionError::InvalidOperation(instruction.clone())), // Instructions are small, so clone
            }
        }
        self.stack.extend(&signature.push);

        Ok(true)
    }

    fn push_global_or_local(
        &mut self,
        module: &Module,
        instruction: &Instruction,
        body: &FuncBody,
        index: usize,
    ) -> Result<bool, InstructionError> {
        // These next couple lines are just to get the parameters of the function we're dealing with.
        // We need the parameters because they can be loaded like local variables but they're not in the locals vec

        // type_ref is the index of the FunctionType in types_section
        let type_ref = &module.function_section().unwrap().entries()[index].type_ref();
        let type_variant = &module.type_section().unwrap().types()[*type_ref as usize];

        let mut locals = body.locals().to_vec();
        match type_variant {
            Type::Function(ftype) => {
                locals.extend(ftype.params().iter().map(|f| Local::new(0, *f)));
            }
        }

        match instruction {
            Instruction::GetGlobal(local) => match locals.get(*local as usize) {
                Some(variable) => {
                    self.stack.push(variable.value_type());
                    Ok(true)
                }
                None => Err(InstructionError::GlobalNotFound),
            },
            Instruction::GetLocal(local) => match locals.get(*local as usize) {
                Some(variable) => {
                    self.stack.push(variable.value_type());
                    Ok(true)
                }
                None => Err(InstructionError::LocalNotFound),
            },
            _ => Err(InstructionError::UnmatchedInstruction),
        }
    }
}

impl InstructionValidator for VerifyInstructions {
    fn validate(&mut self, module: &Module) -> Result<bool, InstructionError> {
        match module.code_section() {
            Some(functions) => {
                for (index, function) in functions.bodies().iter().enumerate() {
                    let is_function_valid: bool =
                        self.check_instructions(module, function, index)?;
                    if !is_function_valid {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            None => Ok(true),
        }
    }
}

fn contains(instruction: &Instruction, container: &[Instruction]) -> bool {
    container
        .iter()
        .any(|f| discriminant(f) == discriminant(instruction))
}

fn get_instruction_signature(instruction: &Instruction) -> Option<Signature> {
    // returns some signature if there is a type we are interested in
    // returns None otherwise
    if contains(instruction, &I32_BINOP) {
        Some(Signature {
            pop: [ValueType::I32; 2].to_vec(),
            push: [ValueType::I32; 1].to_vec(),
        })
    } else if contains(instruction, &I64_BINOP) {
        Some(Signature {
            pop: [ValueType::I64; 2].to_vec(),
            push: [ValueType::I64; 1].to_vec(),
        })
    } else if contains(instruction, &F32_BINOP) {
        Some(Signature {
            pop: [ValueType::F32; 2].to_vec(),
            push: [ValueType::F32; 1].to_vec(),
        })
    } else if contains(instruction, &F64_BINOP) {
        Some(Signature {
            pop: [ValueType::F64; 2].to_vec(),
            push: [ValueType::F64; 1].to_vec(),
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_wasm::elements::deserialize_buffer;

    #[test]
    fn add_two_simple_binary() {
        // WAST:
        // (module
        //   (type $t0 (func (param i32 i32) (result i32)))
        //   (func $f0 (type $t0) (param $p0 i32) (param $p1 i32) (result i32)
        //     (i32.add
        //       (get_local $p0)
        //       (get_local $p1))))
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x07, 0x01, 0x60, 0x02, 0x7f,
            0x7f, 0x01, 0x7f, 0x03, 0x02, 0x01, 0x00, 0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00,
            0x20, 0x01, 0x6a, 0x0b, 0x00, 0x14, 0x04, 0x6e, 0x61, 0x6d, 0x65, 0x02, 0x0d, 0x01,
            0x00, 0x02, 0x00, 0x03, 0x6c, 0x68, 0x73, 0x01, 0x03, 0x72, 0x68, 0x73,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();

        let mut validator = VerifyInstructions::new(NumericInstructions);
        let is_valid = validator.validate(&module).unwrap();
        assert!(true, is_valid)
    }

    #[test]
    #[should_panic]
    fn unmatched_type_failure_binary() {
        // Binary incorrectly tries to add an f64 with i32 to return an i32
        // This should be failed by the validator
        // WAST:
        // (module
        //   (func $addTwo (param f64 i32) (result i32)
        //     get_local 0
        //     get_local 1
        //     i32.add)
        //   (export "addTwo" (func $addTwo)))
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x07, 0x01, 0x60, 0x02, 0x7c,
            0x7f, 0x01, 0x7f, 0x03, 0x02, 0x01, 0x00, 0x07, 0x0a, 0x01, 0x06, 0x61, 0x64, 0x64,
            0x54, 0x77, 0x6f, 0x00, 0x00, 0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01,
            0x6a, 0x0b, 0x00, 0x19, 0x04, 0x6e, 0x61, 0x6d, 0x65, 0x01, 0x09, 0x01, 0x00, 0x06,
            0x61, 0x64, 0x64, 0x54, 0x77, 0x6f, 0x02, 0x07, 0x01, 0x00, 0x02, 0x00, 0x00, 0x01,
            0x00,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();

        let mut validator = VerifyInstructions::new(NumericInstructions);
        validator.validate(&module).unwrap();
    }

    #[test]
    fn print_instructions_complex_binary() {
        // WAST:
        // (module
        //   (type $t0 (func (param i32 i32) (result i32)))
        //   (func $_Z4multii (export "_Z4multii") (type $t0) (param $p0 i32) (param $p1 i32) (result i32)
        //     (i32.mul
        //       (get_local $p1)
        //       (get_local $p0)))
        //   (func $_Z3addii (export "_Z3addii") (type $t0) (param $p0 i32) (param $p1 i32) (result i32)
        //     (i32.add
        //       (get_local $p1)
        //       (get_local $p0)))
        //   (func $_Z6divideii (export "_Z6divideii") (type $t0) (param $p0 i32) (param $p1 i32) (result i32)
        //     (i32.div_s
        //       (get_local $p0)
        //       (get_local $p1)))
        //   (table $T0 0 anyfunc)
        //   (memory $memory (export "memory") 1))

        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x07, 0x01, 0x60, 0x02, 0x7f,
            0x7f, 0x01, 0x7f, 0x03, 0x04, 0x03, 0x00, 0x00, 0x00, 0x04, 0x04, 0x01, 0x70, 0x00,
            0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x2f, 0x04, 0x09, 0x5f, 0x5a, 0x34, 0x6d,
            0x75, 0x6c, 0x74, 0x69, 0x69, 0x00, 0x00, 0x08, 0x5f, 0x5a, 0x33, 0x61, 0x64, 0x64,
            0x69, 0x69, 0x00, 0x01, 0x0b, 0x5f, 0x5a, 0x36, 0x64, 0x69, 0x76, 0x69, 0x64, 0x65,
            0x69, 0x69, 0x00, 0x02, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x0a,
            0x19, 0x03, 0x07, 0x00, 0x20, 0x01, 0x20, 0x00, 0x6c, 0x0b, 0x07, 0x00, 0x20, 0x01,
            0x20, 0x00, 0x6a, 0x0b, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6d, 0x0b, 0x00, 0x4b,
            0x04, 0x6e, 0x61, 0x6d, 0x65, 0x01, 0x23, 0x03, 0x00, 0x09, 0x5f, 0x5a, 0x34, 0x6d,
            0x75, 0x6c, 0x74, 0x69, 0x69, 0x01, 0x08, 0x5f, 0x5a, 0x33, 0x61, 0x64, 0x64, 0x69,
            0x69, 0x02, 0x0b, 0x5f, 0x5a, 0x36, 0x64, 0x69, 0x76, 0x69, 0x64, 0x65, 0x69, 0x69,
            0x02, 0x1f, 0x03, 0x00, 0x02, 0x00, 0x02, 0x70, 0x30, 0x01, 0x02, 0x70, 0x31, 0x01,
            0x02, 0x00, 0x02, 0x70, 0x30, 0x01, 0x02, 0x70, 0x31, 0x02, 0x02, 0x00, 0x02, 0x70,
            0x30, 0x01, 0x02, 0x70, 0x31,
        ];

        let module = deserialize_buffer::<Module>(&wasm).unwrap();

        let mut validator = VerifyInstructions::new(NumericInstructions);
        let is_valid = validator.validate(&module).unwrap();
        assert!(true, is_valid)
    }
}
