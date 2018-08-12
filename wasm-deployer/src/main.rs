extern crate byteorder;
extern crate parity_wasm;
extern crate rustc_hex;

use std::env;

use parity_wasm::elements::*;

use byteorder::{LittleEndian, WriteBytesExt};
use rustc_hex::FromHex;

use std::fs::File;
use std::io::Read;

/*
(module
  (import "ethereum" "getCodeSize" (func $getCodeSize (result i32)))
  (import "ethereum" "codeCopy" (func $codeCopy (param i32 i32 i32)))
  (import "ethereum" "finish" (func $finish (param i32 i32)))
  (memory 1)
  (export "memory" (memory 0))
  (export "main" (func $main))
  (func $main
    ;; load total code size
    (local $size i32)
    (local $payload_offset i32)
    (local $payload_size i32)
    (set_local $size (call $getCodeSize))

    ;; copy entire thing into memory at offset 0
    (call $codeCopy (i32.const 0) (i32.const 0) (get_local $size))

    ;; retrieve payload size (the last 4 bytes treated as a little endian 32 bit number)
    (set_local $payload_size (i32.load (i32.sub (get_local $size) (i32.const 4))))

    ;; start offset is calculated as $size - 4 - $payload_size
    (set_local $payload_offset (i32.sub (i32.sub (get_local $size) (i32.const 4)) (get_local $payload_size)))

    ;; return the payload
    (call $finish (get_local $payload_offset) (get_local $payload_size))
  )
)
*/
fn deployer_code() -> Vec<u8> {
    FromHex::from_hex(
        "
        0061736d010000000113046000017f60037f7f7f0060027f7f00600000023e0308
        657468657265756d0b676574436f646553697a65000008657468657265756d0863
        6f6465436f7079000108657468657265756d0666696e6973680002030201030503
        010001071102066d656d6f72790200046d61696e00030a2c012a01037f10002100
        4100410020001001200041046b2802002102200041046b20026b21012001200210
        020b
    ",
    ).unwrap()
}

/// Returns a module which contains the deployable bytecode as a custom section.
pub fn create_custom_deployer(payload: &[u8]) -> Module {
    // The standard deployer code, which expects a 32 bit little endian as the trailing content
    // immediately following the payload, placed in a custom section.
    let code = deployer_code();

    // This is the pre-written deployer code.
    let mut module: Module = parity_wasm::deserialize_buffer(&code).expect("Failed to load module");

    // Prepare payload (append length).
    let mut custom_payload = payload.to_vec();
    custom_payload
        .write_i32::<LittleEndian>(payload.len() as i32)
        .unwrap();

    // Prepare and append custom section.
    let custom = CustomSection {
        name: "deployer".to_string(),
        payload: custom_payload,
    };
    module
        .sections_mut()
        .push(parity_wasm::elements::Section::Custom(custom));

    module
}

/// Returns a module which contains the deployable bytecode as a data segment.
pub fn create_memory_deployer(payload: &[u8]) -> Module {
    panic!()
}

pub fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 3 {
        println!("Usage: {} in.wasm out.wasm", args[0]);
        return;
    }

    let mut f = File::open(&args[1]).expect("Failed to open file");
    let mut payload = Vec::new();
    f.read_to_end(&mut payload).expect("Failed to read file");

    let module = create_custom_deployer(&payload);

    parity_wasm::serialize_to_file(&args[2], module).expect("Failed to write module");
}

#[cfg(test)]
mod tests {
    use super::create_custom_deployer;
    use parity_wasm;
    use rustc_hex::FromHex;

    #[test]
    fn zero_payload() {
        let payload = vec![];
        let module = create_custom_deployer(&payload);
        let expected = FromHex::from_hex(
            "
            0061736d010000000113046000017f60037f7f7f0060027f7f00600000023e0308
            657468657265756d0b676574436f646553697a65000008657468657265756d0863
            6f6465436f7079000108657468657265756d0666696e6973680002030201030503
            010001071102066d656d6f72790200046d61696e00030a2c012a01037f10002100
            4100410020001001200041046b2802002102200041046b20026b21012001200210
            020b

            000d086465706c6f79657200000000
        ",
        ).unwrap();
        let output = parity_wasm::serialize(module).expect("Failed to serialize");
        assert_eq!(output, expected);
    }

    #[test]
    fn nonzero_payload() {
        let payload = FromHex::from_hex("80ff007faa550011").unwrap();
        let module = create_custom_deployer(&payload);
        let expected = FromHex::from_hex(
            "
            0061736d010000000113046000017f60037f7f7f0060027f7f00600000023e0308
            657468657265756d0b676574436f646553697a65000008657468657265756d0863
            6f6465436f7079000108657468657265756d0666696e6973680002030201030503
            010001071102066d656d6f72790200046d61696e00030a2c012a01037f10002100
            4100410020001001200041046b2802002102200041046b20026b21012001200210
            020b

            0015086465706c6f79657280ff007faa55001108000000
        ",
        ).unwrap();
        let output = parity_wasm::serialize(module).expect("Failed to serialize");
        assert_eq!(output, expected);
    }
}
