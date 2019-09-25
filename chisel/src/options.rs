//! Chisel CLI argument handling.
//! This module implements ChiselFlags, a structure generated from CLI arguments.
//! It stores all options which are independent of any given chisel execution,
//! such as the log level, error handling switches, and output format for Wasm modules produced by
//! the driver's execution.
//! The main exception to this is in unix-mode, where the chisel driver configuration is passed in
//! the command line arguments rather than produced from a config file.
//!
//! Options:
//! NO_RECOVER: Forces panic on recoverable errors.
//! VERBOSE: Enables verbose debug logging.
//! CONFIG: Overrides the configuration file path in config-driven mode.
//! MODULES: A list of modules to invoke in oneliner mode.
//! MODULE_OPTIONS: A list of options set for the modules being invoked in oneliner mode.
//! FILE: Sets the input file path in oneliner mode.
//! OUTPUT_PATH: Sets the path to write any mutated binaries in oneliner mode.
//! OUTPUT_MODE: Sets the format in which to output mutated binaries.
//!      - wasm: default binary mode. disallowed when writing to stdout.
//!      - hex: write the output in hex. recommended if writing to stdout.
//!      - wat: write the output in disassembled (.wat) format.

use std::collections::HashMap;
use std::ops::Deref;

use clap::ArgMatches;

/// Key-value structure for immutable CLI options. Used for storing utility options and
/// configurations in oneliner mode.
pub struct ChiselFlags(HashMap<String, String>);

impl ChiselFlags {
    /// Sets the value of `key`.
    pub fn set(&mut self, key: &str, value: &str) {
        self.0.insert(key.to_string(), value.to_string());
    }

    /// Gets the value of `key`.
    pub fn value_of(&self, key: &str) -> Option<&str> {
        match self.0.get(key) {
            Some(s) => Some(s.as_str()),
            None => None,
        }
    }

    /// Compares the value of `key` to the passed `val`.
    pub fn value_eq(&self, key: &str, val: &str) -> bool {
        match self.value_of(key) {
            Some(s) => s == val,
            None => false,
        }
    }

    /// Apply all flags passed from CLI
    pub fn apply(&mut self, matches: &ArgMatches) {
        if matches.is_present("NO_RECOVER") {
            self.set("util.norecover", "true");
        }
        if matches.is_present("VERBOSE") {
            self.set("util.debugging", "true");
        }
        if let Some(value) = matches.value_of("CONFIG") {
            self.set("run.config.path", value);
        }
        if let Some(values) = matches.values_of("MODULES") {
            let values_collected = values.fold(String::new(), |mut acc, val| {
                acc.push_str(&format!("{},", val));
                acc
            });
            self.set("oneliner.modules", &values_collected);
        }
        if let Some(values) = matches.values_of("MODULE_OPTIONS") {
            let values_collected = values.fold(String::new(), |mut acc, val| {
                acc.push_str(&format!("{},", val));
                acc
            });
            self.set("oneliner.modules.options", &values_collected);
        }
        if let Some(value) = matches.value_of("FILE") {
            self.set("oneliner.file", value);
        }
        if let Some(value) = matches.value_of("OUTPUT_PATH") {
            self.set("oneliner.output", value);
        }
        if let Some(value) = matches.value_of("OUTPUT_MODE") {
            match value {
                val @ "bin" | val @ "wat" | val @ "hex" => {
                    self.set("output.mode", val);
                }
                _ => panic!("CLI parser only accepts 'bin', 'wat', or 'hex'"),
            }
        }
    }
}

impl Deref for ChiselFlags {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for ChiselFlags {
    fn default() -> Self {
        let mut ret = ChiselFlags(HashMap::new());

        ret.set("util.norecover", "false");
        ret.set("util.debugging", "false");
        ret.set("output.mode", "bin");
        ret.set("run.config.path", "./chisel.yml");
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options() {
        let options = ChiselFlags::default();
        assert!(options.value_eq("util.norecover", "false"));
        assert!(options.value_eq("util.debugging", "false"));
        assert!(options.value_eq("output.mode", "bin"));
        assert!(options.value_eq("run.config.path", "./chisel.yml"));
    }
}
