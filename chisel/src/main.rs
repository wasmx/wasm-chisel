#[macro_use]
mod logger;
mod config;
mod driver;
mod options;
mod result;

use clap::{crate_description, crate_name, crate_version};
use std::fs::{read, read_to_string, write};
use std::process;

use clap::{App, Arg, ArgMatches, SubCommand};
use serde_yaml::Value;

use libchisel::{
    checkstartfunc::*, deployer::*, dropsection::*, remapimports::*, remapstart::*, repack::*,
    snip::*, trimexports::*, trimstartfunc::*, verifyexports::*, verifyimports::*,
};

#[cfg(feature = "binaryen")]
use libchisel::binaryenopt::*;

use libchisel::*;

use config::*;

// Error messages
static ERR_NO_SUBCOMMAND: &'static str = "No subcommand provided.";
static ERR_FAILED_OPEN_BINARY: &'static str = "Failed to open wasm binary.";
static ERR_DESERIALIZE_MODULE: &'static str = "Failed to deserialize the wasm binary.";

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

impl ChiselContext {
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
    fn fields(&self) -> (&String, &String) {
        (&self.module_name, &self.preset)
    }
}

fn err_exit(msg: &str) -> ! {
    eprintln!("{}: {}", crate_name!(), msg);
    process::exit(-1);
}

/// Helper that tries both translation methods in the case that a module cannot implement one of them.
fn translate_module<T>(module: &mut Module, translator: &T) -> Result<bool, &'static str>
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
    chisel_debug!(1, "Running module '{}'", name);
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
                translate_module(module, &chisel)
            } else {
                Err("trimexports: Invalid preset")
            }
        }
        "trimstartfunc" => {
            is_translator = true;
            if let Ok(chisel) = TrimStartFunc::with_preset(&preset) {
                translate_module(module, &chisel)
            } else {
                Err("trimstartfunc: Invalid preset")
            }
        }
        "remapimports" => {
            is_translator = true;
            if let Ok(chisel) = RemapImports::with_preset(&preset) {
                translate_module(module, &chisel)
            } else {
                Err("remapimports: Invalid preset")
            }
        }
        "remapstart" => {
            is_translator = true;
            if let Ok(chisel) = RemapStart::with_preset(&preset) {
                translate_module(module, &chisel)
            } else {
                Err("remapimports: Invalid preset")
            }
        }
        "deployer" => {
            is_translator = true;
            if let Ok(chisel) = Deployer::with_preset(&preset) {
                translate_module(module, &chisel)
            } else {
                Err("deployer: Invalid preset")
            }
        }
        "repack" => {
            is_translator = true;
            translate_module(module, &Repack::new())
        }
        "snip" => {
            is_translator = true;
            translate_module(module, &Snip::new())
        }
        "dropnames" => {
            is_translator = true;
            translate_module(module, &DropSection::NamesSection)
        }
        #[cfg(feature = "binaryen")]
        "binaryenopt" => {
            is_translator = true;
            if let Ok(chisel) = BinaryenOptimiser::with_preset(&preset) {
                translate_module(module, &chisel)
            } else {
                Err("binaryenopt: Invalid preset")
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
    eprintln!("\t{}: {}", name, module_status_msg);

    if let Ok(result) = ret {
        if !result && is_translator {
            true
        } else {
            result
        }
    } else {
        false
    }
}

fn chisel_execute(context: &ChiselContext) -> Result<bool, &'static str> {
    if let Ok(buffer) = read(context.file()) {
        if let Ok(module) = Module::from_bytes(&buffer) {
            // If we do not parse the NamesSection here, parity-wasm will drop it at serialisation
            // It is useful to have this for a number of optimisation passes, including binaryenopt and snip
            // TODO: better error handling
            let mut module = module.parse_names().expect("Failed to parse NamesSection");

            let original = module.clone();
            eprintln!("Ruleset {}:", context.name());
            let chisel_results = context
                .get_modules()
                .iter()
                .map(|ctx| execute_module(ctx, &mut module))
                .fold(true, |b, e| e & b);

            // If the module was mutated, serialize to file.
            if original != module {
                let serialized = module.to_bytes().expect("Failed to serialize Module");
                if let Some(path) = context.outfile() {
                    chisel_debug!(1, "Writing to file: {}", path);
                    write(path, serialized).unwrap();
                } else {
                    chisel_debug!(1, "No output file specified; writing in place");
                    write(context.file(), serialized).unwrap();
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

fn yaml_configure(yaml: &str) -> ChiselConfig {
    if let Ok(rulesets) = serde_yaml::from_str::<Value>(yaml) {
        let config = ChiselConfig::from_yaml(&rulesets);
        if config.is_err() {
            err_exit(&config.unwrap_err());
        } else {
            config.unwrap()
        }
    } else {
        err_exit("Failed to parse YAML configuration")
    }
}

impl From<&(String, config::Ruleset)> for ChiselContext {
    fn from(input: &(String, config::Ruleset)) -> ChiselContext {
        let modules: Vec<ModuleContext> =
            input
                .1
                .modules()
                .iter()
                .fold(Vec::new(), |mut acc, module| {
                    acc.push(ModuleContext {
                        module_name: module.0.clone(),
                        preset: module
                            .1
                            .options()
                            .get("preset")
                            .unwrap_or_else(|| err_exit("Module missing required field: 'preset'"))
                            .clone(),
                    });
                    acc
                });

        ChiselContext {
            ruleset_name: input.0.clone(),
            file: input
                .1
                .options()
                .get("file")
                .unwrap_or_else(|| err_exit("Ruleset missing required field: 'file'"))
                .clone(),
            outfile: input.1.options().get("output").cloned(),
            modules: modules,
        }
    }
}

fn chisel_subcommand_run(args: &ArgMatches) -> i32 {
    let config_path = args.value_of("CONFIG").unwrap_or(DEFAULT_CONFIG_PATH);

    if args.is_present("VERBOSE") {
        logger::set_global_log_level(1);
    }

    if let Ok(conf) = read_to_string(config_path) {
        let ctxs = yaml_configure(&conf);

        let ctxs = ctxs.rulesets();
        chisel_debug!(1, "Loaded {} rulesets", ctxs.len());

        let result_final = ctxs.iter().fold(0, |acc, ctx| {
            chisel_debug!(1, "Executing ruleset {}", ctx.0);
            let ctx: ChiselContext = ctx.into();
            match chisel_execute(&ctx) {
                Ok(result) => acc + if result { 0 } else { 1 }, // Add the number of module failures to exit code.
                Err(msg) => err_exit(msg),
            }
        });

        result_final
    } else {
        err_exit(&format!(
            "Could not load configuration file: {}",
            config_path
        ))
    }
}

pub fn main() {
    let cli_matches = App::new("chisel")
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::with_name("VERBOSE")
                .short("v")
                .long("verbose")
                .help("Enables verbose debug logging")
                .global(true),
        )
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
        ("run", Some(subcmd_matches)) => {
            chisel_debug!(1, "Running chisel");
            process::exit(chisel_subcommand_run(subcmd_matches))
        }
        _ => err_exit(ERR_NO_SUBCOMMAND),
    };
}
