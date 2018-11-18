extern crate libchisel;
extern crate parity_wasm;

use std::env;

use libchisel::*;

pub fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 3 {
        println!("Usage: {} in.wasm out.wasm", args[0]);
        return;
    }

    let code = std::fs::read(&args[1]).expect("Failed to open and read file");

    let mut module = parity_wasm::deserialize_buffer(&code).expect("Failed to load module");

    let trimexports = trimexports::TrimExports::with_preset("ewasm");
    trimexports
        .translate(&mut module)
        .expect("Failed to trim exports");

    let remapimports = remapimports::RemapImports::ewasm();
    remapimports
        .translate(&mut module)
        .expect("Failed to remap imports");

    parity_wasm::serialize_to_file(&args[2], module).expect("Failed to write module");
}
