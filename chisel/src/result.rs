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
use std::path::PathBuf;

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
            let path = PathBuf::from(&self.output_path);
            let ret = match mode {
                "bin" => {
                    if *path == PathBuf::from("/dev/stdout")
                        || *path == PathBuf::from("/dev/stderr")
                    {
                        return Err("cannot write raw binary to a standard stream".into());
                    } else {
                        let module = module.to_bytes()?;
                        write(path, module)
                    }
                }
                "hex" => {
                    let module = module.to_bytes()?;
                    let hex = hex::encode(&module);
                    write(path, hex)
                }
                #[cfg(feature = "wabt")]
                "wat" => {
                    let module = module.to_bytes()?;
                    let wat = wabt::wasm2wat(module)?;
                    write(path, wat)
                }
                _ => return Err("invalid mode".into()),
            };
            match ret {
                Ok(()) => Ok(true),
                Err(e) => Err(e.into()),
            }
        } else {
            Ok(false)
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
    fn writer_success_to_stdout() {
        let mut ruleset_result = {
            let mut result = RulesetResult::new("Test".to_string());
            let module = Module::default();
            result.set_output_module(module);
            result.set_output_path(PathBuf::from("/dev/stdout"));
            result
        };

        // First run
        let result = ruleset_result.write("hex");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);

        // Second run
        let result = ruleset_result.write("hex");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

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

        // First run
        let result = ruleset_result.write("hex");
        assert!(result.is_ok());
        assert_eq!(result.expect("Should be Ok"), false);

        // Second run
        let result = ruleset_result.write("hex");
        assert!(result.is_ok());
        assert_eq!(result.expect("Should be Ok"), false);
    }
}
