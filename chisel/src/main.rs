extern crate libchisel;
extern crate parity_wasm;
#[macro_use]
extern crate clap;
extern crate serde;
extern crate serde_derive;
extern crate serde_yaml;

use std::fs::{read, read_to_string};
use std::process;

use libchisel::{
    checkstartfunc::*, deployer::*, remapimports::*, trimexports::*, verifyexports::*,
    verifyimports::*,
};

use clap::{App, Arg, ArgMatches, SubCommand};
use libchisel::*;
use parity_wasm::elements::{deserialize_buffer, serialize_to_file, Module, Serialize};
use serde_yaml::Value;

// Error messages
static ERR_NO_SUBCOMMAND: &'static str = "No subcommand provided.";
static ERR_FAILED_OPEN_CONFIG: &'static str = "Failed to open configuration file.";
static ERR_FAILED_OPEN_BINARY: &'static str = "Failed to open wasm binary.";
static ERR_FAILED_PARSE_CONFIG: &'static str = "Failed to parse configuration file.";
static ERR_CONFIG_INVALID: &'static str = "Config is invalid.";
static ERR_CONFIG_MISSING_FILE: &'static str = "Config missing file path to chisel.";
static ERR_INPUT_FILE_TYPE_MISMATCH: &'static str = "Entry 'file' does not map to a string.";
static ERR_MODULE_TYPE_MISMATCH: &'static str =
    "A module configuration does not point to a key-value map. Perhaps an option field is missing?";
static ERR_PRESET_TYPE_MISMATCH: &'static str =
    "A field 'preset' belonging to a module is not a string";
static ERR_DESERIALIZE_MODULE: &'static str = "Failed to deserialize the wasm binary.";
static ERR_MISSING_PRESET: &'static str = "Module configuration missing preset.";

// Other constants
static DEFAULT_CONFIG_PATH: &'static str = "chisel.yml";

/// Chisel configuration structure. Contains a file to chisel and a list of modules configurations.
struct ChiselContext {
    ruleset_name: String,
    file: String,
    // Output file. If a ModuleTranslator or ModuleCreator is invoked, resorts to a default.
    outfile: Option<String>,
    modules: Vec<ModuleContext>,
}

struct ModuleContext {
    module_name: String,
    preset: String,
}

/// Helper to get a field from a config mapping. Assumes that the Value is a Mapping.
fn get_field(yaml: &Value, key: &str) -> Result<String, &'static str> {
    if let Some(path) = yaml
        .as_mapping()
        .unwrap()
        .get(&Value::String(String::from(key)))
    {
        if path.is_string() {
            Ok(String::from(path.as_str().unwrap()))
        } else {
            Err(ERR_INPUT_FILE_TYPE_MISMATCH)
        }
    } else {
        Err(ERR_CONFIG_MISSING_FILE)
    }
}

impl ChiselContext {
    fn from_ruleset(ruleset: &Value) -> Result<Vec<Self>, &'static str> {
        if let Value::Mapping(rules) = ruleset {
            let mut ret: Vec<ChiselContext> = vec![];

            for (name, mut config) in rules.iter().filter(|(left, right)| match (left, right) {
                (Value::String(_s), Value::Mapping(_m)) => true,
                _ => false,
            }) {
                let filepath = get_field(config, "file")?;

                let outfilepath = if let Ok(out) = get_field(config, "output") {
                    Some(out)
                } else {
                    None
                };

                // Parse all valid module entries. Unwrap is ok here because we
                // established earlier that config is a Mapping.
                let mut config_clone = config.as_mapping().unwrap().clone();
                // Remove "file" and "output" so we don't interpret it as a module.
                // TODO: use mappings to avoid the need for this
                config_clone.remove(&Value::String(String::from("file")));
                config_clone.remove(&Value::String(String::from("output")));

                let mut module_confs: Vec<ModuleContext> = vec![];
                let mut config_itr = config_clone.iter();
                // Read modules while there are still modules left.
                while let Some(module) = config_itr.next() {
                    module_confs.push(ModuleContext::from_yaml(module)?);
                }

                ret.push(ChiselContext {
                    ruleset_name: name.as_str().unwrap().into(),
                    file: filepath,
                    outfile: outfilepath,
                    modules: module_confs,
                });
            }

            Ok(ret)
        } else {
            Err(ERR_CONFIG_INVALID)
        }
    }

    fn name(&self) -> &String {
        &self.ruleset_name
    }

    fn file(&self) -> &String {
        &self.file
    }

    fn outfile(&self) -> &Option<String> {
        &self.outfile
    }

    fn get_modules(&self) -> &Vec<ModuleContext> {
        &self.modules
    }
}

impl ModuleContext {
    fn from_yaml(yaml: (&Value, &Value)) -> Result<Self, &'static str> {
        match yaml {
            (Value::String(name), Value::Mapping(flags)) => Ok(ModuleContext {
                module_name: name.clone(),
                preset: if let Some(pset) = flags.get(&Value::String(String::from("preset"))) {
                    // Check that the value to which "preset" resolves is a String. If not, return an error.
                    if pset.is_string() {
                        String::from(pset.as_str().unwrap())
                    } else {
                        return Err(ERR_PRESET_TYPE_MISMATCH);
                    }
                } else {
                    return Err(ERR_MISSING_PRESET);
                },
            }),
            _ => Err(ERR_MODULE_TYPE_MISMATCH),
        }
    }

    fn fields(&self) -> (&String, &String) {
        (&self.module_name, &self.preset)
    }
}

fn err_exit(msg: &str) -> ! {
    println!("{}: {}", crate_name!(), msg);
    process::exit(-1);
}

fn yaml_configure(yaml: &String) -> Result<Vec<ChiselContext>, &'static str> {
    if let Ok(rulesets) = serde_yaml::from_str::<Value>(yaml.as_str()) {
        ChiselContext::from_ruleset(&rulesets)
    } else {
        Err(ERR_FAILED_PARSE_CONFIG)
    }
}

/// Helper that tries both translation methods in the case that a module cannot implement one of them.
fn translate_module<T>(module: &mut Module, translator: T) -> Result<bool, &'static str>
where
    T: ModuleTranslator,
{
    // NOTE: The module must return an Err (in the case of failure) without mutating the module or nasty stuff happens.
    if let Ok(ret) = translator.translate_inplace(module) {
        Ok(ret)
    } else if let Ok(new_module) = translator.translate(module) {
        if new_module.is_some() {
            *module = new_module.unwrap();
            Ok(true)
        } else {
            Ok(false)
        }
    } else {
        Err("Module translation failed")
    }
}

fn execute_module(context: &ModuleContext, module: &mut Module) -> bool {
    let (conf_name, conf_preset) = context.fields();
    let preset = conf_preset.clone();

    let mut is_translator = false; // Flag representing if the module is a translator
    let name = conf_name.as_str();
    let ret = match name {
        "verifyexports" => {
            if let Ok(chisel) = VerifyExports::with_preset(&preset) {
                Ok(chisel.validate(module).unwrap_or(false))
            } else {
                Err("verifyexports: Invalid preset")
            }
        }
        "verifyimports" => {
            if let Ok(chisel) = VerifyImports::with_preset(&preset) {
                Ok(chisel.validate(module).unwrap_or(false))
            } else {
                Err("verifyimports: Invalid preset")
            }
        }
        "checkstartfunc" => {
            // NOTE: checkstartfunc takes a bool for configuration. false by default for now.
            let chisel = CheckStartFunc::new(false);
            let ret = chisel.validate(module).unwrap_or(false);
            Ok(ret)
        }
        "trimexports" => {
            is_translator = true;
            if let Ok(chisel) = TrimExports::with_preset(&preset) {
                translate_module(module, chisel)
            } else {
                Err("trimexports: Invalid preset")
            }
        }
        "remapimports" => {
            is_translator = true;
            if let Ok(chisel) = RemapImports::with_preset(&preset) {
                translate_module(module, chisel)
            } else {
                Err("remapimports: Invalid preset")
            }
        }
        "deployer" => {
            is_translator = true;
            let mut payload = Vec::new();
            module.clone().serialize(&mut payload).unwrap(); // This should not fail, but perhaps check anyway?

            if let Ok(chisel) = Deployer::with_preset(&preset, &payload) {
                let new_module = chisel.create().unwrap();
                *module = new_module;
                Ok(true)
            } else {
                Err("deployer: Invalid preset")
            }
        }
        _ => Err("Module Not Found"),
    };

    let module_status_msg = if let Ok(result) = ret {
        match (result, is_translator) {
            (true, true) => "Translated",
            (true, false) => "OK",
            (false, true) => "Already OK; not translated",
            (false, false) => "Malformed",
        }
    } else {
        ret.unwrap_err()
    };
    println!("\t{}: {}", name, module_status_msg);

    if let Ok(result) = ret {
        result
    } else {
        false
    }
}

fn chisel_execute(context: &ChiselContext) -> Result<bool, &'static str> {
    if let Ok(buffer) = read(context.file()) {
        if let Ok(mut module) = deserialize_buffer::<Module>(&buffer) {
            let original = module.clone();
            println!("Ruleset {}:", context.name());
            let chisel_results = context
                .get_modules()
                .iter()
                .map(|ctx| execute_module(ctx, &mut module))
                .fold(true, |b, e| e & b);

            // If the module was mutated, serialize to file.
            if original != module {
                if let Some(path) = context.outfile() {
                    println!("Writing to file: {}", path);
                    serialize_to_file(path, module).unwrap();
                } else {
                    println!("No output file specified; writing in place");
                    serialize_to_file(context.file(), module).unwrap();
                }
            }
            Ok(chisel_results)
        } else {
            Err(ERR_DESERIALIZE_MODULE)
        }
    } else {
        Err(ERR_FAILED_OPEN_BINARY)
    }
}

fn chisel_subcommand_run(args: &ArgMatches) -> i32 {
    let config_path = args.value_of("CONFIG").unwrap_or(DEFAULT_CONFIG_PATH);

    if let Ok(conf) = read_to_string(config_path) {
        match yaml_configure(&conf) {
            Ok(ctxs) => {
                let result_final = ctxs.iter().fold(0, |acc, ctx| match chisel_execute(&ctx) {
                    Ok(result) => acc + if result { 0 } else { 1 }, // Add the number of module failures to exit code.
                    Err(msg) => err_exit(msg),
                });
                return result_final;
            }
            Err(msg) => err_exit(msg),
        };
    } else {
        err_exit(ERR_FAILED_OPEN_CONFIG);
    }
}

pub fn main() {
    let cli_matches = App::new("chisel")
        .version(crate_version!())
        .about(crate_description!())
        .subcommand(
            SubCommand::with_name("run")
                .about("Runs chisel with the closest configuration file.")
                .arg(
                    Arg::with_name("CONFIG")
                        .short("c")
                        .long("config")
                        .help("Sets a custom configuration file")
                        .value_name("CONF_FILE")
                        .takes_value(true),
                ),
        )
        .get_matches();

    match cli_matches.subcommand() {
        ("run", Some(subcmd_matches)) => process::exit(chisel_subcommand_run(subcmd_matches)),
        _ => err_exit(ERR_NO_SUBCOMMAND),
    };
}
