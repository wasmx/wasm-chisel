use std::collections::HashMap;

use parity_wasm::elements::{ExportEntry, ExportSection, Internal, Module, Section};

use super::{ChiselModule, ModuleError, ModuleKind, ModulePreset, ModuleTranslator};

pub struct RemapStart;

impl ModulePreset for RemapStart {
    fn with_preset(preset: &str) -> Result<Self, ModuleError> {
        match preset {
            // TODO refactor this later
            "ewasm" => Ok(RemapStart {}),
            _ => Err(ModuleError::NotSupported),
        }
    }
}

impl RemapStart {
    pub fn new() -> Self {
        RemapStart {}
    }
}

impl<'a> ChiselModule<'a> for RemapStart {
    type ObjectReference = &'a dyn ModuleTranslator;

    fn id(&'a self) -> String {
        "remapstart".to_string()
    }

    fn kind(&'a self) -> ModuleKind {
        ModuleKind::Translator
    }

    fn as_abstract(&'a self) -> Self::ObjectReference {
        self as Self::ObjectReference
    }

    fn with_defaults() -> Result<Self, ModuleError> {
        Ok(RemapStart {})
    }

    // FIXME: drop this, no need for preset here
    fn with_config(config: &HashMap<String, String>) -> Result<Self, ModuleError> {
        if let Some(preset) = config.get("preset") {
            RemapStart::with_preset(preset)
        } else {
            Err(ModuleError::NotSupported)
        }
    }
}

impl ModuleTranslator for RemapStart {
    fn translate_inplace(&self, module: &mut Module) -> Result<bool, ModuleError> {
        Ok(remap_start(module))
    }

    fn translate(&self, module: &Module) -> Result<Option<Module>, ModuleError> {
        let mut ret = module.clone();
        if remap_start(&mut ret) {
            Ok(Some(ret))
        } else {
            Ok(None)
        }
    }
}

/// Replace an exported function with another function, or export if unexported.
fn remap_or_export_main(module: &mut Module, export_name: &str, func_idx: u32) {
    let new_func_export = ExportEntry::new(export_name.to_string(), Internal::Function(func_idx));

    if let Some(export_section) = module.export_section_mut() {
        let export_section = export_section.entries_mut();
        // If we find an export named `export_name`, replace it. Otherwise, append an entry to the
        // section with the supplied func index.
        if let Some(main_export_loc) = export_section
            .iter_mut()
            .position(|e| e.field() == export_name)
        {
            export_section[main_export_loc] = new_func_export;
        } else {
            export_section.push(new_func_export);
        }
    } else {
        let new_export_section =
            Section::Export(ExportSection::with_entries(vec![new_func_export]));

        // This should not fail, because there is no existing export section.
        module
            .insert_section(new_export_section)
            .expect("insert_section should not fail");
    }
}

fn remap_start(module: &mut Module) -> bool {
    if let Some(start_func_idx) = module.start_section() {
        // Look for an export "main". If found, replace it with an export of the function to
        // which the start section points.
        remap_or_export_main(module, "main", start_func_idx);

        // Remove the start section, leaving the "main" export as the entry point.
        module.clear_start_section();

        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use rustc_hex::FromHex;

    use super::*;
    use crate::{ModulePreset, ModuleTranslator};

    #[test]
    fn remapstart_mutation() {
        //wat:
        //(module
        //    (import "env" "ethereum_useGas" (func (param i64)))
        //    (memory 1)
        //    (export "main" (func $main))
        //    (export "memory" (memory 0))
        //    (func $main2)
        //    (func $main)
        //    (start $main2)
        //)

        let wasm: Vec<u8> = FromHex::from_hex(
            "0061736d0100000001080260017e0060
000002170103656e760f657468657265756d5f75736547617300000303020101050301000107110
2046d61696e0001066d656d6f727902000801020a070202000b02000b0020046e616d65010e0201
046d61696e02056d61696e320209030001000001000200",
        )
        .unwrap();

        let mut module = Module::from_bytes(&wasm).unwrap();
        module = module.parse_names().unwrap();
        assert!(module.names_section().is_some());
        let start_idx = module
            .start_section()
            .expect("Module missing start function");

        let new = RemapStart::with_preset("ewasm")
            .unwrap()
            .translate(&module)
            .expect("Module internal error")
            .expect("new module not returned");

        assert!(
            new.start_section().is_none(),
            "start section wasn't removed"
        );
        assert!(new
            .export_section()
            .expect("Module missing export section")
            .entries()
            .iter()
            .find(|e| e.field() == String::from("main")
                && *e.internal() == Internal::Function(start_idx))
            .is_some());
    }

    #[test]
    fn remapstart_no_mutation() {
        // (module
        //    (import "env" "ethereum_useGas" (func (param i64)))
        //    (memory 1)
        //    (export "main" (func $main))
        //    (export "memory" (memory 0))
        //    (func $main)
        //)

        let wasm: Vec<u8> = FromHex::from_hex(
            "0061736d0100000001080260017e0060
        000002170103656e760f657468657265756d5f757365476173000003020101050301000
        1071102046d61696e0001066d656d6f727902000a040102000b",
        )
        .unwrap();

        let module = Module::from_bytes(&wasm).unwrap();
        let new = RemapStart::with_preset("ewasm")
            .unwrap()
            .translate(&module)
            .expect("Module internal error");

        assert!(new.is_none());
    }

    #[test]
    fn remapstart_inplace_mutation() {
        //wat:
        //(module
        //    (import "env" "ethereum_useGas" (func (param i64)))
        //    (memory 1)
        //    (export "main" (func $main))
        //    (export "memory" (memory 0))
        //    (func $main2)
        //    (func $main)
        //    (start $main2)
        //)

        let wasm: Vec<u8> = FromHex::from_hex(
            "0061736d0100000001080260017e0060
000002170103656e760f657468657265756d5f75736547617300000303020101050301000107110
2046d61696e0001066d656d6f727902000801020a070202000b02000b0020046e616d65010e0201
046d61696e02056d61696e320209030001000001000200",
        )
        .unwrap();

        let mut module = Module::from_bytes(&wasm).unwrap();
        module = module.parse_names().unwrap();
        assert!(module.names_section().is_some());

        let res = RemapStart::with_preset("ewasm")
            .unwrap()
            .translate_inplace(&mut module)
            .unwrap();

        assert!(res, "module was not modified");
        assert!(
            module.start_section().is_none(),
            "start section wasn't removed"
        );
    }

    #[test]
    fn remapstart_inplace_no_mutation() {
        // (module
        //    (import "env" "ethereum_useGas" (func (param i64)))
        //    (memory 1)
        //    (export "main" (func $main))
        //    (export "memory" (memory 0))
        //    (func $main)
        //)

        let wasm: Vec<u8> = FromHex::from_hex(
            "0061736d0100000001080260017e0060
000002170103656e760f657468657265756d5f75736547617300000302010105030100010711020
46d61696e0001066d656d6f727902000a040102000b",
        )
        .unwrap();

        let mut module = Module::from_bytes(&wasm).unwrap();
        let res = RemapStart::with_preset("ewasm")
            .unwrap()
            .translate_inplace(&mut module)
            .unwrap();

        assert!(!res, "module was modified");
    }

    #[test]
    fn remapstart_mutation_no_exports() {
        //wat:
        //(module
        //    (import "env" "ethereum_useGas" (func (param i64)))
        //    (memory 1)
        //    (func $main2)
        //    (func $main)
        //    (start $main2)
        //)

        let wasm: Vec<u8> = FromHex::from_hex(
            "0061736d0100000001080260017e0060000002170103656e760f657468657265756d5f7573654761730000030302010105030100010801010a070202000b02000b",
        )
        .unwrap();

        let mut module = Module::from_bytes(&wasm).unwrap();
        let res = RemapStart::with_preset("ewasm")
            .unwrap()
            .translate_inplace(&mut module)
            .unwrap();

        assert!(res, "module was not modified");
        assert!(
            module.export_section().is_some(),
            "export section does not exist"
        );
    }

    #[test]
    fn export_section_exists_but_no_main() {
        // wat:
        // (module
        //     (import "env" "ethereum_useGas" (func (param i64)))
        //     (memory 1)
        //     (start $main)
        //     (export "memory" (memory 0))
        //     (func $main)
        // )
        let wasm: Vec<u8> = FromHex::from_hex(
            "0061736d0100000001080260017e0060000002170103656e760f657468657265756d5f7573654761730000030201010503010001070a01066d656d6f727902000801010a040102000b"
        ).unwrap();
        let mut module = Module::from_bytes(&wasm).unwrap();
        let remapper = RemapStart::with_preset("ewasm").expect("Can't fail");

        let res = remapper.translate_inplace(&mut module);
        assert!(res.is_ok());
        let mutated = res.unwrap();
        assert_eq!(mutated, true);
        assert!(module.export_section().is_some());
        assert!(module.start_section().is_none());
        assert!(module
            .export_section()
            .unwrap()
            .entries()
            .iter()
            .find(|e| e.field() == "main")
            .is_some());
    }
}
