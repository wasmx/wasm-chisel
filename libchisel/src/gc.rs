extern crate parity_wasm;
extern crate wasm_gc;

use super::ModuleTranslator;
use parity_wasm::elements::*;

pub struct WasmGC(wasm_gc::Config);

impl Default for WasmGC {
    fn default() -> WasmGC {
        WasmGC(wasm_gc::Config::new())
    }
}

impl WasmGC {
    pub fn with_config(cfg: wasm_gc::Config) -> Self {
        WasmGC(cfg)
    }
}

macro_rules! update_section {
    ($muts:expr, $cons:expr, $empty:expr) => {
        match ($muts, $cons) {
            (Some(m), Some(c)) => *m = c.clone(),
            (Some(m), None) => *m = $empty,
            _ => {}
        };
    };
}

impl ModuleTranslator for WasmGC {
    fn translate(mut self, module: &mut Module) -> Result<bool, String> {
        let serialized = parity_wasm::elements::serialize::<Module>(module.clone())
            .expect("Could not serialize module");
        match self.0.gc(&serialized[..]) {
            Ok(gced_bytes) => {
                let gced = parity_wasm::elements::deserialize_buffer::<Module>(&gced_bytes[..])
                    .expect("Could not deserialize gc'ed module");

                // Presumably, the custom section will not be modified

                update_section!(
                    module.type_section_mut(),
                    gced.type_section(),
                    TypeSection::with_types(vec![])
                );
                update_section!(
                    module.import_section_mut(),
                    gced.import_section(),
                    ImportSection::with_entries(vec![])
                );
                update_section!(
                    module.function_section_mut(),
                    gced.function_section(),
                    FunctionSection::with_entries(vec![])
                );
                update_section!(
                    module.table_section_mut(),
                    gced.table_section(),
                    TableSection::with_entries(vec![])
                );
                update_section!(
                    module.memory_section_mut(),
                    gced.memory_section(),
                    MemorySection::with_entries(vec![])
                );
                update_section!(
                    module.global_section_mut(),
                    gced.global_section(),
                    GlobalSection::with_entries(vec![])
                );
                update_section!(
                    module.export_section_mut(),
                    gced.export_section(),
                    ExportSection::with_entries(vec![])
                );
                update_section!(
                    module.elements_section_mut(),
                    gced.elements_section(),
                    ElementSection::with_entries(vec![])
                );
                update_section!(
                    module.code_section_mut(),
                    gced.code_section(),
                    CodeSection::with_bodies(vec![])
                );
                update_section!(
                    module.data_section_mut(),
                    gced.data_section(),
                    DataSection::with_entries(vec![])
                );

                if module.start_section() != gced.start_section() {
                    if let Some(f) = gced.start_section() {
                        module.set_start_section(f);
                    } else {
                        module.clear_start_section();
                    }
                }

                Ok(gced_bytes.len() != serialized.len())
            }
            Err(e) => Err(format!("GC failure: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_wasm::deserialize_buffer;
    use rustc_hex::FromHex;

    #[test]
    fn do_not_touch_simple_module() {
        let wasm: Vec<u8> = FromHex::from_hex("0061736d01000000").unwrap();

        let mut module = deserialize_buffer::<Module>(&wasm).unwrap();
        let result = WasmGC::default().translate(&mut module).unwrap();
        assert_eq!(false, result);
    }

    #[test]
    fn gc_does_something() {
        let wasm: Vec<u8> = FromHex::from_hex(
            "
            0061736d0100000001120460017f017f6000017f60000060017f017f024d05
            03656e760a6d656d6f727942617365037f0003656e76066d656d6f72790200
            800203656e76057461626c650170000003656e76097461626c654261736503
            7f0003656e76055f7075747300030304030102020610037f0141000b7f0141
            000b7f0041000b073304125f5f706f73745f696e7374616e74696174650003
            055f6d61696e00010b72756e506f7374536574730002045f73747203040901
            000a25030900230010001a41000b0300010b1500230041106a240223024180
            80c0026a240310020b0b13010023000b0d68656c6c6f2c20776f726c6421
            ",
        ).unwrap();

        let mut module = deserialize_buffer::<Module>(&wasm).unwrap();
        let result = WasmGC::default().translate(&mut module).unwrap();
        assert_eq!(true, result);
    }

    #[test]
    fn remove_unneeded_types() {
        let wasm: Vec<u8> = FromHex::from_hex(
            "
            0061736d0100000001120460017f017f6000017f60000060017f017f024d05
            03656e760a6d656d6f727942617365037f0003656e76066d656d6f72790200
            800203656e76057461626c650170000003656e76097461626c654261736503
            7f0003656e76055f7075747300030304030102020610037f0141000b7f0141
            000b7f0041000b073304125f5f706f73745f696e7374616e74696174650003
            055f6d61696e00010b72756e506f7374536574730002045f73747203040901
            000a25030900230010001a41000b0300010b1500230041106a240223024180
            80c0026a240310020b0b13010023000b0d68656c6c6f2c20776f726c6421
            ",
        ).unwrap();

        let mut module = deserialize_buffer::<Module>(&wasm).unwrap();
        assert_eq!(4, module.type_section().unwrap().types().len());
        WasmGC::default().translate(&mut module).unwrap();
        assert_eq!(3, module.type_section().unwrap().types().len());
    }

    #[test]
    fn remove_unneeded_imports() {
        let wasm: Vec<u8> = FromHex::from_hex(
            "
            0061736d0100000001120460017f017f6000017f60000060017f017f024d05
            03656e760a6d656d6f727942617365037f0003656e76066d656d6f72790200
            800203656e76057461626c650170000003656e76097461626c654261736503
            7f0003656e76055f7075747300030304030102020610037f0141000b7f0141
            000b7f0041000b073304125f5f706f73745f696e7374616e74696174650003
            055f6d61696e00010b72756e506f7374536574730002045f73747203040901
            000a25030900230010001a41000b0300010b1500230041106a240223024180
            80c0026a240310020b0b13010023000b0d68656c6c6f2c20776f726c6421
            ",
        ).unwrap();

        let mut module = deserialize_buffer::<Module>(&wasm).unwrap();
        assert_eq!(5, module.import_section().unwrap().entries().len());
        WasmGC::default().translate(&mut module).unwrap();
        assert_eq!(3, module.import_section().unwrap().entries().len());
    }

    #[test]
    fn remove_unneeded_functions() {
        let wasm: Vec<u8> = FromHex::from_hex(
            "
            0061736d0100000001120460017f017f6000017f60000060017f017f024d05
            03656e760a6d656d6f727942617365037f0003656e76066d656d6f72790200
            800203656e76057461626c650170000003656e76097461626c654261736503
            7f0003656e76055f707574730003030504010202020610037f0141000b7f01
            41000b7f0041000b073304125f5f706f73745f696e7374616e746961746500
            03055f6d61696e00010b72756e506f7374536574730002045f737472030409
            01000a29040900230010001a41000b0300010b1500230041106a2402230241
            8080c0026a240310020b0300010b0b13010023000b0d68656c6c6f2c20776f
            726c6421
            ",
        ).unwrap();

        let mut module = deserialize_buffer::<Module>(&wasm).unwrap();
        assert_eq!(4, module.function_section().unwrap().entries().len());
        WasmGC::default().translate(&mut module).unwrap();
        assert_eq!(3, module.function_section().unwrap().entries().len());
    }

    #[test]
    fn update_start() {
        let wasm: Vec<u8> = FromHex::from_hex(
            "
            0061736d0100000001120460017f017f6000017f60000060017f017f024d05
            03656e760a6d656d6f727942617365037f0003656e76066d656d6f72790200
            800203656e76057461626c650170000003656e76097461626c654261736503
            7f0003656e76055f707574730003030504010102020610037f0141000b7f01
            41000b7f0041000b073304125f5f706f73745f696e7374616e746961746500
            04055f6d61696e00020b72756e506f7374536574730003045f737472030408
            01030901000a2a04040041000b0900230010001a41000b0300010b15002300
            41106a24022302418080c0026a240310030b0b13010023000b0d68656c6c6f
            2c20776f726c6421
            ",
        ).unwrap();

        let mut module = deserialize_buffer::<Module>(&wasm).unwrap();
        assert_eq!(3, module.start_section().unwrap());
        WasmGC::default().translate(&mut module).unwrap();
        assert_eq!(2, module.start_section().unwrap());
    }
}
