//! Chisel execution results structures.
//! This module implements ChiselResult, RulesetResult, and ModuleResult.
//! These structures represent execution manifests for chisel modules, and are produced
//! by ChiselDriver upon completed execution.
//! ChiselConfig is a nested structure which contains RulesetResults, which in turn contain
//! ModuleResults. This makes it structurally identical to the ChiselConfig for the driver
//! execution which produced it.
//! RulesetResult also implements utilities for writing the resulting Wasm module to file, if the
//! driver performed any transformations.

use std::error::Error;
use std::fmt::{self, Display};
use std::fs::write;
use std::path::{Path, PathBuf};

use ansi_term::Colour::{Green, Red, Yellow};

#[cfg(feature = "wabt")]
use libchisel::wabt;
use libchisel::{Module, ModuleError};

#[derive(Clone)]
/// Main result structure returned by ChiselDriver, containing a manifest of modules executed and
/// exposing methods to write output from translators and creators.
pub struct ChiselResult(Vec<RulesetResult>);

#[derive(Clone)]
/// Ruleset result structure.
pub struct RulesetResult {
    ruleset_name: String,
    results: Vec<ModuleResult>,
    output_path: PathBuf,
    output_module: Option<Module>,
}

/// Output writer struct for Chisel module output.
pub enum ChiselOutputWriter {
    Bin(Module, PathBuf),
    Hex(Module, PathBuf),
    #[cfg(feature = "wabt")]
    Wat(Module, PathBuf),
}

#[derive(Clone)]
/// Individual module execution result. Left-hand field is the module name, and left-hand is the
/// return value.
pub enum ModuleResult {
    Creator(String, Result<bool, ModuleError>),
    Translator(String, Result<bool, ModuleError>),
    Validator(String, Result<bool, ModuleError>),
}

impl ChiselResult {
    pub fn new() -> Self {
        ChiselResult(Vec::new())
    }

    pub fn rulesets_mut(&mut self) -> &mut Vec<RulesetResult> {
        &mut self.0
    }

    pub fn rulesets(&self) -> &Vec<RulesetResult> {
        &self.0
    }
}

impl RulesetResult {
    pub fn new(name: String) -> Self {
        RulesetResult {
            ruleset_name: name,
            results: Vec::new(),
            output_path: PathBuf::new(),
            output_module: None,
        }
    }

    pub fn name(&self) -> &str {
        self.ruleset_name.as_str()
    }

    pub fn results_mut(&mut self) -> &mut Vec<ModuleResult> {
        &mut self.results
    }

    pub fn set_output_path(&mut self, path: PathBuf) {
        self.output_path = path;
    }

    pub fn set_output_module(&mut self, module: Module) {
        self.output_module = Some(module);
    }

    /// Write output module to specified file if the module was mutated.
    /// Returns Ok(false) if there is no mutation.
    /// Returns error on writer error or invalid mode.
    pub fn write(&mut self, mode: &str) -> Result<bool, Box<dyn Error>> {
        if let Some(module) = self.output_module.take() {
            let writer = match mode {
                "bin" => Some(ChiselOutputWriter::Bin(
                    module,
                    PathBuf::from(&self.output_path),
                )),
                "hex" => Some(ChiselOutputWriter::Hex(
                    module,
                    PathBuf::from(&self.output_path),
                )),
                #[cfg(feature = "wabt")]
                "wat" => Some(ChiselOutputWriter::Wat(
                    module,
                    PathBuf::from(&self.output_path),
                )),
                _ => None,
            };
            if let Some(writer) = writer {
                match writer.write() {
                    Ok(()) => Ok(true),
                    Err(e) => Err(e),
                }
            } else {
                Err("invalid mode".into())
            }
        } else {
            Ok(false)
        }
    }
}

impl ChiselOutputWriter {
    /// Serializes and writes the contained module.
    pub fn write(self) -> Result<(), Box<dyn Error>> {
        let ret = match self {
            ChiselOutputWriter::Bin(module, path) => {
                if *path == PathBuf::from("/dev/stdout") || *path == PathBuf::from("/dev/stderr") {
                    return Err("cannot write raw binary to a standard stream".into());
                } else {
                    let module = module.to_bytes()?;
                    write(path, module)
                }
            }
            ChiselOutputWriter::Hex(module, path) => {
                let module = module.to_bytes()?;
                let hex = hex::encode(&module);
                write(path, hex)
            }
            #[cfg(feature = "wabt")]
            ChiselOutputWriter::Wat(module, path) => {
                let module = module.to_bytes()?;
                let wat = wabt::wasm2wat(module)?;
                write(path, wat)
            }
        };
        match ret {
            Ok(()) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

impl Display for ChiselResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0
            .iter()
            .map(|ruleset_result| write!(f, "{}", ruleset_result))
            .fold(Ok(()), |acc, r| if r.is_err() { r } else { acc })
    }
}

impl Display for RulesetResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result = write!(f, "\nRuleset {}:", &self.name());
        if let Err(e) = self
            .results
            .iter()
            .map(|module_result| write!(f, "\n\t{}", module_result))
            .fold(Ok(()), |acc, r| if r.is_err() { r } else { acc })
        {
            Err(e)
        } else {
            result
        }
    }
}

impl Display for ModuleResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ModuleResult::Creator(name, result) => writeln!(
                f,
                "Creator {}: {}",
                name,
                match result {
                    Ok(r) => {
                        if *r {
                            Green.paint("OK")
                        } else {
                            Red.paint("FAILED")
                        }
                    }
                    Err(e) => Red.bold().paint(format!("ERROR; {}", e.description())),
                }
            ),
            ModuleResult::Translator(name, result) => write!(
                f,
                "Translator {}: {}",
                name,
                match result {
                    Ok(r) => {
                        if *r {
                            Yellow.paint("MUTATED")
                        } else {
                            Green.paint("NO CHANGE")
                        }
                    }
                    Err(e) => Red.bold().paint(format!("ERROR; {}", e.description())),
                }
            ),
            ModuleResult::Validator(name, result) => write!(
                f,
                "Validator {}: {}",
                name,
                match result {
                    Ok(r) => {
                        if *r {
                            Green.paint("VALID")
                        } else {
                            Red.paint("INVALID")
                        }
                    }
                    Err(e) => Red.bold().paint(format!("ERROR; {}", e.description())),
                }
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writer_deny_raw_binary_to_stdout() {
        let mut ruleset_result = {
            let mut result = RulesetResult::new("Test".to_string());
            let module = Module::default();
            result.set_output_module(module);
            result.set_output_path(PathBuf::from("/dev/stdout"));
            result
        };

        let result = ruleset_result.write("bin");
        assert!(result.is_err());
    }

    #[test]
    fn writer_invalid_mode() {
        let mut ruleset_result = {
            let mut result = RulesetResult::new("Test".to_string());
            let module = Module::default();
            result.set_output_module(module);
            result.set_output_path(PathBuf::from("/dev/stdout"));
            result
        };

        let result = ruleset_result.write("foo");
        assert!(result.is_err());
    }

    #[test]
    fn writer_no_module() {
        let mut ruleset_result = RulesetResult::new("Test".to_string());

        let result = ruleset_result.write("hex");
        assert_eq!(result.expect("Should be Ok"), false);
    }
}
