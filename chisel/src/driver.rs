//! Chisel driver implementation.
//! ChiselDriver is a state machine-like structure implementing the core logic for chisel
//! execution.
//! It consumes a ChiselConfig generated by its caller and executes the specified rulesets.
//! If it enters an error state, it is up to the caller to handle it. If called again, the ruleset
//! in which the error occurred is dropped.
//! Upon completed execution, the driver returns a ChiselResult structure.

use std::error::Error;
use std::fmt::{self, Display};
use std::fs::{canonicalize, read};
use std::path::PathBuf;

#[cfg(feature = "binaryen")]
use libchisel::binaryenopt::BinaryenOptimiser;
use libchisel::{
    checkfloat::CheckFloat, checkstartfunc::CheckStartFunc, deployer::Deployer,
    dropsection::DropSection, remapimports::RemapImports, remapstart::RemapStart, repack::Repack,
    snip::Snip, trimexports::TrimExports, trimstartfunc::TrimStartFunc,
    verifyexports::VerifyExports, verifyimports::VerifyImports, WasmModule, ModulePreset,
    ModuleTranslator, ModuleValidator,
};

use crate::config::{ChiselConfig, ModuleConfig};
use crate::result::{ChiselResult, ModuleResult, RulesetResult};

/// State machine implementing the main chisel execution loop. Consumes ChiselConfig and returns
/// ChiselResult, with an intermediate state returned to allow error handling.
pub struct ChiselDriver {
    config: ChiselConfig,
    state: DriverState,
}

/// The state of the chisel driver.
pub enum DriverState {
    Ready,
    Error(DriverError, ChiselResult),
    Done(ChiselResult),
}

pub enum DriverError {
    /// A module or ruleset is missing a required field. Left-hand is the config object name,
    /// right-hand is the missing field.
    MissingRequiredField(String, String),
    /// The contained module name was not successfully resolved to an existing chisel module.
    ModuleNotFound(String),
    /// A configuration value is of incorrect type or invalid value. Left-hand is the config object
    /// name, right-hand is the name of the invalid field.
    InvalidField(String, String),
    /// A canonicalized path was generated unsuccessfully. Left-hand is the config object name,
    /// right-hand is the invalid path.
    PathResolution(String, String),
    /// An internal error occurred. Field 0 is the config object, during the execution of which the error occurred.
    /// Field 1 is an additional informational message. Field 2 is the error generated.
    Internal(String, String, Box<dyn Error>),
}

impl ChiselDriver {
    pub fn new(config: ChiselConfig) -> Self {
        ChiselDriver {
            config,
            state: DriverState::Ready,
        }
    }

    pub fn take_result(self) -> ChiselResult {
        match self.state {
            DriverState::Ready => {
                panic!("take_result should never be called on a driver in 'ready' state")
            }
            DriverState::Error(_, result) => result,
            DriverState::Done(result) => result,
        }
    }

    pub fn fire(&mut self) -> &DriverState {
        let mut results = match &mut self.state {
            DriverState::Ready => ChiselResult::new(),
            DriverState::Error(_, previous_result) => previous_result.clone(),
            DriverState::Done(_) => panic!("fire() called on a completed driver"),
        };

        // Consume the rulesets in the configuration and execute each one.
        while let Some((name, mut ruleset)) = self.config.rulesets_mut().pop_front() {
            let mut ruleset_result = RulesetResult::new(name.clone());

            // Load binary.
            chisel_debug!(1, "Running ruleset {}", name);
            chisel_debug!(1, "Looking for binary path...");
            let binary_path = if let Some(binary_path) = ruleset.options().get(&"file".to_string())
            {
                chisel_debug!(1, "Found binary path: {}", &binary_path);
                chisel_debug!(1, "Attempting to resolve path...");

                match canonicalize(binary_path) {
                    Ok(path_resolved) => {
                        chisel_debug!(1, "Successfully resolved binary path");
                        path_resolved
                    }
                    Err(_) => {
                        chisel_debug!(1, "Failed to resolve binary path");
                        self.state = DriverState::Error(
                            DriverError::PathResolution(name.clone(), binary_path.clone()),
                            results,
                        );
                        return &self.state;
                    }
                }
            } else {
                self.state = DriverState::Error(
                    DriverError::MissingRequiredField(name.clone(), "file".to_string()),
                    results,
                );
                return &self.state;
            };

            // Look for output path and set.
            let output_path =
                if let Some(output_path) = ruleset.options().get(&"output".to_string()) {
                    chisel_debug!(1, "Found output path: {}", &output_path);
                    PathBuf::from(output_path)
                } else {
                    chisel_debug!(1, "No output path found.");
                    binary_path.clone()
                };
            ruleset_result.set_output_path(output_path);

            // Load the wasm binary into a buffer before deserialization.
            chisel_debug!(1, "Deserializing module from file");
            let wasm_raw = match read(binary_path) {
                Ok(ret) => ret,
                Err(e) => {
                    chisel_debug!(1, "Failed to load Wasm binary");
                    self.state = DriverState::Error(
                        DriverError::Internal(
                            name.clone(),
                            "Failed to load file".to_string(),
                            e.into(),
                        ),
                        results,
                    );
                    return &self.state;
                }
            };

            // Try parsing as Wasm text (Wat) first. Note: this function passes through binaries.
            let wasm_raw = match wat::parse_bytes(&wasm_raw) {
                Ok(ret) => ret,
                Err(e) => {
                    chisel_debug!(1, "Failed to parse input as text");
                    self.state = DriverState::Error(
                        DriverError::Internal(
                            name.clone(),
                            "Failed to parse input as text".to_string(),
                            e.into(),
                        ),
                        results,
                    );
                    return &self.state;
                }
            };

            // Deserialize the Wasm binary and parse its names section.
            let mut wasm = match WasmModule::from_bytes(wasm_raw) {
                Ok(wasm) => {
                    chisel_debug!(1, "Successfully deserialized Wasm module");
                    // TODO: Make this error recoverable
                    wasm.parse_names().expect("names parsing failed")
                }
                Err(e) => {
                    chisel_debug!(1, "Failed to deserialize Wasm module");
                    self.state = DriverState::Error(
                        DriverError::Internal(
                            name.clone(),
                            "Deserialization failure".to_string(),
                            e.into(),
                        ),
                        results,
                    );
                    return &self.state;
                }
            };

            // Consume modules in ruleset and execute.
            while let Some((name, module)) = ruleset.modules_mut().pop_front() {
                chisel_debug!(1, "Executing module {}", &name);

                let module_result = match self.execute_module(name, module, &mut wasm) {
                    Ok(result) => result,
                    Err(error_state) => {
                        self.state = DriverState::Error(error_state, results);
                        return &self.state;
                    }
                };

                // If the module was a translator or creator, we set the output in the result.
                match module_result {
                    ModuleResult::Creator(_, ref result)
                    | ModuleResult::Translator(_, ref result) => {
                        if let Ok(true) = result {
                            chisel_debug!(1, "Module mutated or created.");
                            ruleset_result.set_output_module(wasm.clone()); //TODO: Refactor to only set this at the end and save some expensive copies
                        }
                    }
                    ModuleResult::Validator(_, _) => (),
                }
                ruleset_result.results_mut().push(module_result);
            }
            results.rulesets_mut().push(ruleset_result);
        }
        self.state = DriverState::Done(results);
        &self.state
    }

    pub fn execute_module(
        &mut self,
        name: String,
        module: ModuleConfig,
        wasm: &mut WasmModule,
    ) -> Result<ModuleResult, DriverError> {
        let result = match name.as_str() {
            "checkfloat" => {
                let checkfloat = CheckFloat::new();
                let module_result = checkfloat.validate(wasm);
                ModuleResult::Validator(name, module_result)
            }
            "checkstartfunc" => {
                if let Some(require_start) = module.options().get("require_start") {
                    let require_start = match require_start.as_str() {
                        "true" => true,
                        "false" => false,
                        _ => {
                            return Err(DriverError::InvalidField(
                                name,
                                "require_start".to_string(),
                            ));
                        }
                    };
                    let checkstartfunc = CheckStartFunc::new(require_start);
                    let module_result = checkstartfunc.validate(wasm);
                    ModuleResult::Validator(name, module_result)
                } else {
                    chisel_debug!(1, "checkstartfunc missing field 'require_start'");
                    return Err(DriverError::MissingRequiredField(
                        name,
                        "require_start".to_string(),
                    ));
                }
            }
            "deployer" => {
                if let Some(preset) = module.options().get("preset") {
                    match Deployer::with_preset(preset.as_str()) {
                        Ok(deployer) => match deployer.translate(wasm) {
                            Ok(new_wasm) => {
                                let did_mutate = if let Some(new_wasm) = new_wasm {
                                    *wasm = new_wasm;
                                    true
                                } else {
                                    false
                                };

                                ModuleResult::Translator(name, Ok(did_mutate))
                            }
                            Err(e) => ModuleResult::Translator(name, Err(e)),
                        },
                        Err(_) => {
                            chisel_debug!(1, "deployer given invalid preset");
                            return Err(DriverError::InvalidField(name, "preset".to_string()));
                        }
                    }
                } else {
                    chisel_debug!(1, "deployer missing field 'preset'");
                    return Err(DriverError::MissingRequiredField(
                        name,
                        "preset".to_string(),
                    ));
                }
            }
            "dropnames" => {
                let dropsection = DropSection::NamesSection;
                ModuleResult::Translator(name, dropsection.translate_inplace(wasm))
            }
            "remapimports" => {
                if let Some(preset) = module.options().get("preset") {
                    let remapimports = RemapImports::with_preset(preset.as_str());
                    if let Ok(remapimports) = remapimports {
                        let module_result = remapimports.translate_inplace(wasm);
                        ModuleResult::Translator(name, module_result)
                    } else {
                        chisel_debug!(1, "remapimports given invalid preset");
                        return Err(DriverError::InvalidField(name, "preset".to_string()));
                    }
                } else {
                    chisel_debug!(1, "remapimports missing field 'preset'");
                    return Err(DriverError::MissingRequiredField(
                        name,
                        "preset".to_string(),
                    ));
                }
            }
            "remapstart" => {
                // NOTE: preset "ewasm" maps to the default and only mode. Fixing
                // later.
                let remapstart = RemapStart::with_preset("ewasm").expect("Should not fail");
                let module_result = remapstart.translate_inplace(wasm);
                ModuleResult::Translator(name, module_result)
            }
            "repack" => {
                let repack = Repack::new();
                let module_result = repack.translate(wasm).expect("No failure cases");

                let did_mutate = if let Some(new_wasm) = module_result {
                    *wasm = new_wasm;
                    true
                } else {
                    false
                };

                ModuleResult::Translator(name, Ok(did_mutate))
            }
            "snip" => {
                let snip = Snip::new();
                let module_result = match snip.translate(wasm) {
                    Ok(result) => result,
                    Err(e) => {
                        return Err(DriverError::Internal(
                            "snip".to_string(),
                            "Chisel module failed".to_string(),
                            e.into(),
                        ))
                    }
                };

                let did_mutate = if let Some(new_wasm) = module_result {
                    *wasm = new_wasm;
                    true
                } else {
                    false
                };

                ModuleResult::Translator(name, Ok(did_mutate))
            }
            "trimexports" => {
                if let Some(preset) = module.options().get("preset") {
                    let trimexports = TrimExports::with_preset(preset.as_str());
                    if let Ok(trimexports) = trimexports {
                        let module_result = trimexports.translate_inplace(wasm);
                        ModuleResult::Translator(name, module_result)
                    } else {
                        chisel_debug!(1, "trimexports given invalid preset");
                        return Err(DriverError::InvalidField(name, "preset".to_string()));
                    }
                } else {
                    chisel_debug!(1, "remapimports missing field 'preset'");
                    return Err(DriverError::MissingRequiredField(
                        name,
                        "preset".to_string(),
                    ));
                }
            }
            "trimstartfunc" => {
                // NOTE: preset "ewasm" maps to the default and only mode. Fixing
                // later.
                let trimstartfunc = TrimStartFunc::with_preset("ewasm").expect("Should not fail");
                let module_result = trimstartfunc.translate_inplace(wasm);
                ModuleResult::Translator(name, module_result)
            }
            "verifyexports" => {
                if let Some(preset) = module.options().get("preset") {
                    let verifyexports = VerifyExports::with_preset(preset.as_str());
                    if let Ok(verifyexports) = verifyexports {
                        let module_result = verifyexports.validate(wasm);
                        ModuleResult::Validator(name, module_result)
                    } else {
                        chisel_debug!(1, "verifyexports given invalid preset");
                        return Err(DriverError::InvalidField(name, "preset".to_string()));
                    }
                } else {
                    chisel_debug!(1, "verifyexports missing field 'preset'");
                    return Err(DriverError::MissingRequiredField(
                        name,
                        "preset".to_string(),
                    ));
                }
            }
            "verifyimports" => {
                if let Some(preset) = module.options().get("preset") {
                    let verifyimports = VerifyImports::with_preset(preset.as_str());
                    if let Ok(verifyimports) = verifyimports {
                        let module_result = verifyimports.validate(&wasm);
                        ModuleResult::Validator(name, module_result)
                    } else {
                        chisel_debug!(1, "verifyimports given invalid preset");
                        return Err(DriverError::InvalidField(name, "preset".to_string()));
                    }
                } else {
                    chisel_debug!(1, "verifyimports missing field 'preset'");
                    return Err(DriverError::MissingRequiredField(
                        name,
                        "preset".to_string(),
                    ));
                }
            }
            #[cfg(feature = "binaryen")]
            "binaryenopt" => {
                if let Some(preset) = module.options().get("preset") {
                    let binaryenopt = BinaryenOptimiser::with_preset(preset.as_str());
                    if let Ok(binaryenopt) = binaryenopt {
                        match binaryenopt.translate(wasm) {
                            Ok(new_wasm) => {
                                let did_mutate = if let Some(new_wasm) = new_wasm {
                                    *wasm = new_wasm;
                                    true
                                } else {
                                    false
                                };

                                ModuleResult::Translator(name, Ok(did_mutate))
                            }
                            Err(e) => ModuleResult::Translator(name, Err(e)),
                        }
                    } else {
                        chisel_debug!(1, "binaryenopt given invalid preset");
                        return Err(DriverError::InvalidField(name, "preset".to_string()));
                    }
                } else {
                    chisel_debug!(1, "binaryenopt missing field 'preset'");
                    return Err(DriverError::MissingRequiredField(
                        name,
                        "preset".to_string(),
                    ));
                }
            }
            _ => {
                return Err(DriverError::ModuleNotFound(name.clone()));
            }
        };
        Ok(result)
    }
}

// Error.description() is deprecated for displaying errors now.
impl Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DriverError::MissingRequiredField(object, field) => {
                write!(f, "in '{}': missing required field '{}'", object, field)
            }
            DriverError::ModuleNotFound(module) => write!(f, "in '{}': module not found", module),
            DriverError::InvalidField(object, field) => {
                write!(f, "in '{}': invalid field '{}'", object, field)
            }
            DriverError::PathResolution(object, path) => {
                write!(f, "in '{}': failed to resolve path '{}'", object, path)
            }
            DriverError::Internal(object, info, err) => {
                write!(f, "in '{}': {}; {}", object, info, err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::panic::catch_unwind;

    use super::*;
    use crate::config::{ChiselConfig, FromArgs};

    #[test]
    fn take_result_ready() {
        let result = catch_unwind(|| {
            let config = ChiselConfig::from_args("test", "test.foo=bar").expect("Cannot fail");
            let driver = ChiselDriver::new(config);
            driver.take_result()
        });
        assert!(result.is_err());
    }

    #[test]
    fn config_missing_path() {
        let config = ChiselConfig::from_args("verifyimports", "verifyimports.preset=ewasm")
            .expect("Cannot fail");

        let mut driver = ChiselDriver::new(config);

        match driver.fire() {
            DriverState::Error(err, _) => match err {
                DriverError::MissingRequiredField(_, _) => (),
                _ => panic!("Incorrect error"),
            },
            _ => panic!("Must succeed"),
        }
    }

    #[test]
    fn module_not_found() {
        let config = ChiselConfig::from_args("foo", "foo.bar=baz").expect("Cannot fail");

        let mut driver = ChiselDriver::new(config);

        match driver.fire() {
            DriverState::Error(_, _) => (),
            _ => panic!("Must be error state"),
        }
    }

    #[test]
    fn execute_module_smoke() {
        let mut config = ChiselConfig::from_args("verifyimports", "verifyimports.preset=ewasm")
            .expect("Cannot fail");

        config.rulesets_mut()[0]
            .1
            .options_mut()
            .insert("file".to_string(), "./res/test/empty.wasm".to_string());

        let mut driver = ChiselDriver::new(config);

        match driver.fire() {
            DriverState::Done(_) => (),
            _ => panic!("Must succeed"),
        }

        let mut result = driver.take_result();

        assert_eq!(result.rulesets().len(), 1);
        assert_eq!(result.rulesets_mut()[0].results_mut().len(), 1);

        let module_result = &result.rulesets_mut()[0].results_mut()[0];

        let is_correct = match module_result {
            ModuleResult::Validator(name, Ok(true)) => *name == "verifyimports",
            _ => false,
        };

        assert!(is_correct, "Module result incorrect");
    }
}
