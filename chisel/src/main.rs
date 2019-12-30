#[macro_use]
mod logger;
mod cmd_oneliner;
mod cmd_run;
mod config;
mod driver;
mod options;
mod result;

use std::process;

use clap::{crate_description, crate_name, crate_version, App, Arg, SubCommand};

use cmd_oneliner::chisel_oneliner;
use cmd_run::chisel_run;
use options::ChiselFlags;

fn fail(code: i32, message: &str) -> ! {
    eprintln!("{}: {}", crate_name!(), message);
    process::exit(code);
}

pub fn main() {
    let cli_matches = App::new("chisel")
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::with_name("NO_RECOVER")
                .short("x")
                .long("norecover")
                .help("Exits immediately on all recoverable errors")
                .global(true),
        )
        .arg(
            Arg::with_name("DEBUG_MESSAGES")
                .short("d")
                .long("debug")
                .help("Enables debug messages")
                .global(true),
        )
        .arg(
            Arg::with_name("MODULES")
                .short("m")
                .long("modules")
                .takes_value(true)
                .multiple(true)
                .require_delimiter(true)
                .help("Selects modules to use"),
        )
        .arg(
            Arg::with_name("MODULE_OPTIONS")
                .short("c")
                .long("config")
                .takes_value(true)
                .multiple(true)
                .require_delimiter(true)
                .help("Module configuration in unix mode\nConfiguration items come in the form \"module.field=value\"\n\tExample: verifyimports.preset=ewasm"),
        )
        .arg(
            Arg::with_name("OUTPUT_PATH")
                .short("o")
                .long("output")
                .help("Sets the output file in unix mode")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("OUTPUT_MODE")
                .long("output-mode")
                .takes_value(true)
                .help("Selects the type of output")
                .possible_values(&["bin", "wat", "hex"])
                .global(true)
        )
        .arg(Arg::with_name("FILE").help("File to chisel"))
        .subcommand(
            SubCommand::with_name("run")
                .about("Runs chisel in config-driven mode.")
                .arg(
                    Arg::with_name("CONFIG")
                        .short("c")
                        .long("config")
                        .help("Sets the configuration file in config-driven mode.")
                        .value_name("PATH")
                        .takes_value(true),
                ),
        )
        .after_help("chisel runs in two primary modes: unix-style and config-driven.\n\nunix-style is invoked without a subcommand. \
                    It allows the user to run chisel in a single command and manipulate or redirect its output through standard streams. \
                    \nUsage example: chisel file.wasm --modules remapimports --config remapimports.preset=ewasm \
                    \n\nConfig-driven mode relies entirely on a configuration file written in YAML. It is invoked with 'chisel run'. \
                    For more information on the configuration format, please refer to https://github.com/wasmx/wasm-chisel")
        .get_matches();

    let mut flags = ChiselFlags::default();

    match cli_matches.subcommand() {
        ("run", args) => {
            if let Some(opts) = args {
                flags.apply(opts);
            }

            chisel_run(flags)
        }
        ("", None) => {
            flags.apply(&cli_matches);
            chisel_oneliner(flags)
        }
        (_, _) => fail(1, "invalid subcommand"),
    };
}
