use parity_wasm::elements::Instruction;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum InstructionError {
	GlobalNotFound,
	LocalNotFound,
	UnmatchedInstruction,
	InvalidOperation(Instruction),
}

impl fmt::Display for InstructionError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			InstructionError::GlobalNotFound =>
				write!(f, "Global not found"),
			InstructionError::LocalNotFound =>
				write!(f, "Local not found"),
			InstructionError::UnmatchedInstruction =>
				write!(f, "Unmatched instruction"),
			InstructionError::InvalidOperation(i) =>
				write!(f, "{}", format!("Invalid operation: {:?}", i).as_str()),
		}
	}
}

impl error::Error for InstructionError {
	fn description(&self) -> &str {
		match self {
			InstructionError::GlobalNotFound =>
				"Global not found",
			InstructionError::LocalNotFound =>
				"Local not found",
			InstructionError::UnmatchedInstruction =>
				"Unmatched instruction",
			InstructionError::InvalidOperation(_) =>
				"Invalid operation"
		}
	}

	fn cause(&self) -> Option<&error::Error> {
		None
	}
}
