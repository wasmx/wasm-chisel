//! Unix-style chisel mode implementation.
//! The main entry point is chisel_oneliner, which uses FromArgs to produce ChiselConfig
//! from the relevant options passed in the CLI.
//! Like config-driven mode, it then passes the config to the driver, executes, and writes
//! output to the specified file (or stdout, if no file is specified).

use crate::config::ChiselConfig;
use crate::config::FromArgs;
use crate::driver::{ChiselDriver, DriverState};
use crate::fail;
use crate::logger;
use crate::options::ChiselFlags;

pub fn chisel_oneliner(flags: ChiselFlags) -> i32 {
    let log_level = match flags.value_of("util.debugging") {
        Some("true") => 1i32,
        Some("false") => 0i32,
        _ => panic!("util.debugging must be set 'true' or 'false'"),
    };
    logger::set_global_log_level(log_level);

    chisel_debug!(1, "Running chisel in oneliner (unix-style) mode");

    // If no modules were passed, just exit.
    match flags.value_of("oneliner.modules") {
        Some(module_list) => {
            chisel_debug!(1, "Modules passed:\n\t{}", module_list);

            let options_list = if let Some(opts) = flags.value_of("oneliner.modules.options") {
                chisel_debug!(1, "Module options passed:\n\t{}", opts);
                opts
            } else {
                ""
            };

            //input file
            let file = flags
                .value_of("oneliner.file")
                .unwrap_or_else(|| fail(1, "No file specified"));

            //output file
            let output = flags.value_of("oneliner.output");
            let output = match output {
                Some(p) => p.to_string(),
                None => "/dev/stdout".to_string(),
            };

            let chisel_config = match ChiselConfig::from_args(module_list, options_list) {
                Ok(mut config) => {
                    // Inject the input and output file paths here.
                    config.rulesets_mut()[0]
                        .1
                        .options_mut()
                        .insert("file".to_string(), file.to_string());
                    config.rulesets_mut()[0]
                        .1
                        .options_mut()
                        .insert("output".to_string(), output);
                    config
                }
                Err(e) => fail(1, &format!("Failed to load configuration: {}", e)),
            };

            chisel_debug!(1, "{}", chisel_config);

            let mut driver = ChiselDriver::new(chisel_config);

            let mut exit_code = 0;
            loop {
                match driver.fire() {
                    DriverState::Error(err, _) => {
                        fail(1, &format!("runtime error: {}", err));
                    }
                    DriverState::Done(_) => break,
                    _ => panic!("Should never return READY"),
                }
            }

            let results = driver.take_result();
            // wish list: write yaml-encoded results to stdout
            chisel_debug!(1, "Module execution completed successfully");
            eprintln!("{}", &results);

            let mut results = results.unwrap(); // Get ruleset
            let io_result = match flags.value_of("output.mode") {
                Some("bin") => {
                    let result = results.pop().expect("One ruleset was executed");
                    result.write("bin")
                }
                #[cfg(feature = "wabt")]
                Some("wat") => {
                    let result = results.pop().expect("One ruleset was executed");
                    result.write("wat")
                }
                #[cfg(not(feature = "wabt"))]
                Some("wat") => Err("This build does not support wat-encoded output"
                    .to_string()
                    .into()),
                Some("hex") => {
                    let result = results.pop().expect("One ruleset was executed");
                    result.write("hex")
                }
                _ => panic!("CLI parser ensures value can only be one of the above"),
            };

            match io_result {
                Ok(true) => eprintln!("Successfully wrote output to file."),
                Ok(false) => eprintln!("No changes to write."),
                Err(e) => {
                    eprintln!("failed to write output to file: {}", e.description());
                    exit_code = 1;
                }
            }

            exit_code
        }
        None => fail(1, "no modules specified"),
    }
}
