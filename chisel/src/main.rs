#[macro_use]
mod logger;
mod cmd_run;
mod config;
mod driver;
mod options;
mod result;

use clap::{crate_description, crate_name, crate_version};
use clap::{App, Arg, SubCommand};

use std::process;

use cmd_run::chisel_subcommand_run;

static ERR_NO_SUBCOMMAND: &'static str = "No subcommand provided.";

fn err_exit(msg: &str) -> ! {
    eprintln!("{}: {}", crate_name!(), msg);
    process::exit(-1);
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
