use super::{ModuleCreator, ModuleError};
use parity_wasm::builder;
use parity_wasm::elements::{CustomSection, Module};

use byteorder::{LittleEndian, WriteBytesExt};

/// Enum on which ModuleCreator is implemented.
pub enum Deployer<'a> {
    Memory(&'a [u8]),
    CustomSection(&'a [u8]),
}

// FIXME: Bring ModulePreset API in line with the other with_preset methods so a ModulePreset impl
// can be written
impl<'a> Deployer<'a> {
    pub fn with_preset(preset: &str, payload: &'a [u8]) -> Result<Self, ()> {
        match preset {
            "memory" => Ok(Deployer::Memory(payload)),
            "customsection" => Ok(Deployer::CustomSection(payload)),
            _ => Err(()),
        }
    }
}

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
    vec![
        0, 97, 115, 109, 1, 0, 0, 0, 1, 19, 4, 96, 0, 1, 127, 96, 3, 127, 127, 127, 0, 96, 2, 127,
        127, 0, 96, 0, 0, 2, 62, 3, 8, 101, 116, 104, 101, 114, 101, 117, 109, 11, 103, 101, 116,
        67, 111, 100, 101, 83, 105, 122, 101, 0, 0, 8, 101, 116, 104, 101, 114, 101, 117, 109, 8,
        99, 111, 100, 101, 67, 111, 112, 121, 0, 1, 8, 101, 116, 104, 101, 114, 101, 117, 109, 6,
        102, 105, 110, 105, 115, 104, 0, 2, 3, 2, 1, 3, 5, 3, 1, 0, 1, 7, 17, 2, 6, 109, 101, 109,
        111, 114, 121, 2, 0, 4, 109, 97, 105, 110, 0, 3, 10, 44, 1, 42, 1, 3, 127, 16, 0, 33, 0,
        65, 0, 65, 0, 32, 0, 16, 1, 32, 0, 65, 4, 107, 40, 2, 0, 33, 2, 32, 0, 65, 4, 107, 32, 2,
        107, 33, 1, 32, 1, 32, 2, 16, 2, 11,
    ]
}

/// Returns a module which contains the deployable bytecode as a custom section.
fn create_custom_deployer(payload: &[u8]) -> Module {
    // The standard deployer code, which expects a 32 bit little endian as the trailing content
    // immediately following the payload, placed in a custom section.
    let code = deployer_code();

    // This is the pre-written deployer code.
    let mut module: Module = parity_wasm::deserialize_buffer(&code).expect("Failed to load module");

    // Re-write memory to pre-allocate enough for code size
    let memory_initial = (payload.len() as u32 / 65536) + 1;
    let mem_type = parity_wasm::elements::MemoryType::new(memory_initial, None, false);
    module.memory_section_mut().unwrap().entries_mut()[0] = mem_type;

    // Prepare payload (append length).
    let mut custom_payload = payload.to_vec();
    custom_payload
        .write_i32::<LittleEndian>(payload.len() as i32)
        .unwrap();

    // Prepare and append custom section.
    let custom = CustomSection::new("deployer".to_string(), custom_payload);

    module
        .sections_mut()
        .push(parity_wasm::elements::Section::Custom(custom));

    module
}

/// Returns a module which contains the deployable bytecode as a data segment.
#[cfg_attr(rustfmt, rustfmt_skip)]
fn create_memory_deployer(payload: &[u8]) -> Module {
    // Instructions calling finish(0, payload_len)
    let instructions = vec![
        parity_wasm::elements::Instruction::I32Const(0),
        parity_wasm::elements::Instruction::I32Const(payload.len() as i32),
        parity_wasm::elements::Instruction::Call(0),
        parity_wasm::elements::Instruction::End,
    ];

    let memory_initial = (payload.len() as u32 / 65536) + 1;

    let module = builder::module()
        // Create a func/type for the ethereum::finish
        .function()
            .signature()
              .param().i32()
              .param().i32()
              .build()
            .build()
        .import()
            .module("ethereum")
            .field("finish")
            .external()
              .func(0)
            .build()
        // Create the "main fucntion"
        .function()
            // Empty signature `(func)`
            .signature().build()
            .body()
              .with_instructions(parity_wasm::elements::Instructions::new(instructions))
              .build()
            .build()
        // Export the "main" function.
        .export()
            .field("main")
            .internal()
              .func(2)
            .build()
        // Add default memory section
        .memory()
            .with_min(memory_initial)
            .build()
        // Export memory
        .export()
            .field("memory")
            .internal()
              .memory(0)
            .build()
        // Add data section with payload
        .data()
            .offset(parity_wasm::elements::Instruction::I32Const(0))
            .value(payload.to_vec())
            .build()
        .build();

    module
}

impl<'a> ModuleCreator for Deployer<'a> {
    fn create(&self) -> Result<Module, ModuleError> {
        let output = match self {
            Deployer::Memory(payload) => create_memory_deployer(&payload),
            Deployer::CustomSection(payload) => create_custom_deployer(&payload),
        };

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_wasm;
    use rustc_hex::FromHex;

    #[test]
    fn zero_payload() {
        let payload = vec![];
        let module = Deployer::with_preset("customsection", &payload)
            .unwrap()
            .create()
            .unwrap();
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
        )
        .unwrap();
        let output = parity_wasm::serialize(module).expect("Failed to serialize");
        assert_eq!(output, expected);
    }

    #[test]
    fn nonzero_payload() {
        let payload = FromHex::from_hex("80ff007faa550011").unwrap();
        let module = Deployer::with_preset("customsection", &payload)
            .unwrap()
            .create()
            .unwrap();
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
        )
        .unwrap();
        let output = parity_wasm::serialize(module).expect("Failed to serialize");
        assert_eq!(output, expected);
    }

    #[test]
    fn big_payload() {
        let payload = [0; 632232];
        let module = Deployer::with_preset("customsection", &payload)
            .unwrap()
            .create()
            .unwrap();
        let memory_initial = module.memory_section().unwrap().entries()[0]
            .limits()
            .initial();
        assert_eq!(memory_initial, 10);
    }

    #[test]
    fn memory_zero_payload() {
        let payload = vec![];
        let module = Deployer::with_preset("memory", &payload)
            .unwrap()
            .create()
            .unwrap();
        let expected = FromHex::from_hex(
            "
            0061736d0100000001090260027f7f0060000002130108657468657265756d0666
            696e697368000003030200010503010001071102046d61696e0002066d656d6f72
            7902000a0d0202000b08004100410010000b0b06010041000b00
        ",
        )
        .unwrap();
        let output = parity_wasm::serialize(module).expect("Failed to serialize");
        assert_eq!(output, expected);
    }

    #[test]
    fn memory_nonzero_payload() {
        let payload = FromHex::from_hex("80ff007faa550011").unwrap();
        let module = Deployer::with_preset("memory", &payload)
            .unwrap()
            .create()
            .unwrap();
        let expected = FromHex::from_hex(
            "
            0061736d0100000001090260027f7f0060000002130108657468657265756d0666
            696e697368000003030200010503010001071102046d61696e0002066d656d6f72
            7902000a0d0202000b08004100410810000b0b0e010041000b0880ff007faa5500
            11
        ",
        )
        .unwrap();
        let output = parity_wasm::serialize(module).expect("Failed to serialize");
        assert_eq!(output, expected);
    }

    #[test]
    fn memory_big_payload() {
        let payload = [0; 632232];
        let module = Deployer::with_preset("memory", &payload)
            .unwrap()
            .create()
            .unwrap();
        let memory_initial = module.memory_section().unwrap().entries()[0]
            .limits()
            .initial();
        assert_eq!(memory_initial, 10);
    }
}
