use std::collections::HashMap;
use super::bin_reader::BinReader;
use super::sections_parser::Section;
use super::super::constants::opcodes::*;

// WebAssembly utilizes types to generalize parameters and returns for funcs.
#[derive(Clone, Default)]
pub struct TypeDef {
    pub params: usize,
    pub returns: usize,
}

// WasmManager stores general WebAssembly binary data that multiplty utilities may need to access.
pub struct WasmManager {
    pub types: Vec<TypeDef>,
    pub func_type_indices: Vec<usize>,
    pub import_funcs_count: usize,

    pub import_globals: Vec<(String, String, u8, bool)>,
    pub import_tables: Vec<(String, String, u32, Option<u32>)>,
    pub import_memories: Vec<(String, String, u32, Option<u32>)>,

    pub globals: Vec<(u8, bool, String)>,
    pub tables: Vec<(u8, u32, Option<u32>)>,
    pub elements: Vec<(bool, u32, String, Vec<u32>)>,

    pub func_names: HashMap<usize, String>,
    pub export_names: HashMap<usize, String>,
    pub export_memories: HashMap<usize, String>,
    pub export_tables: HashMap<usize, String>,
    pub export_globals: HashMap<usize, String>,
    pub import_names: HashMap<usize, (String, String)>,

    pub initial_memory_pages: u32,
    pub data_segments: Vec<(bool, String, Vec<u8>)>,
    pub start_func_id: Option<usize>,
}

impl WasmManager {
    pub fn new(sections: &[Section]) -> Self {
        let mut manager = Self {
            types: Vec::new(),
            func_type_indices: Vec::new(),
            import_funcs_count: 0,
            import_globals: Vec::new(),
            import_tables: Vec::new(),
            import_memories: Vec::new(),
            globals: Vec::new(),
            tables: Vec::new(),
            elements: Vec::new(),
            func_names: HashMap::new(),
            export_names: HashMap::new(),
            export_memories: HashMap::new(),
            export_tables: HashMap::new(),
            export_globals: HashMap::new(),
            import_names: HashMap::new(),
            initial_memory_pages: 0,
            data_segments: Vec::new(),
            start_func_id: None,
        };

        manager.parse_types(sections);
        manager.parse_imports(sections);
        manager.parse_func_types(sections);
        manager.parse_tables(sections);
        manager.parse_memory(sections);
        manager.parse_globals(sections);
        manager.parse_exports(sections);
        manager.parse_start(sections);
        manager.parse_elements(sections);
        manager.parse_data(sections);
        manager.parse_names(sections);

        manager
    }

    fn read_limits(reader: &mut BinReader) -> (u32, Option<u32>) {
        let flags = reader.read_u32().unwrap_or(0);
        let min = reader.read_u32().unwrap_or(0);
        let max = if flags & 0x01 != 0 {
            Some(reader.read_u32().unwrap_or(0))
        } else {
            None
        };
        (min, max)
    }

    // Parse function names into wasm_export_[name], import_[name] if it is an import, or wasm_func_[name]
    pub fn parse_func_name(&self, func_id: usize) -> String {
        if func_id < self.import_funcs_count {
            format!("import_{}", func_id)
        } else if let Some(name) = self.export_names.get(&func_id) {
            format!("wasm_export_{}", name)
        } else if let Some(name) = self.func_names.get(&func_id) {
            name.clone()
        } else {
            format!("wasm_func_{}", func_id)
        }
    }

    fn evaluate_const_expr(&self, reader: &mut BinReader) -> String {
        let mut last = "0".to_string();

        loop {
            let opcode = match reader.read_byte() {
                Ok(v) => v,
                Err(_) => break,
            };

            if opcode == END {
                break;
            }

            match opcode {
                I32_CONST => last = reader.read_i32().unwrap_or(0).to_string(),
                I64_CONST => last = format!("{}n", reader.read_i64().unwrap_or(0)),
                F32_CONST => last = reader.read_f32().unwrap_or(0.0).to_string(),
                F64_CONST => last = reader.read_f64().unwrap_or(0.0).to_string(),
                GLOBAL_GET => {
                    let id = reader.read_u32().unwrap_or(0) as usize;
                    if id < self.import_globals.len() {
                        last = format!("(imported_global_{} !== null && typeof imported_global_{} === 'object' && 'value' in imported_global_{} ? imported_global_{}.value : global_{})", id, id, id, id, id);
                    } else {
                        last = format!("global_{}", id);
                    }
                }
                _ => {}
            }
        }

        last
    }

    // Parse function types section (section 1 of WASM binary).
    fn parse_types(&mut self, sections: &[Section]) {
        if let Some(section) = sections.iter().find(|s| s.id == 1) {
            let mut reader = BinReader::new(section.data.clone());
            let count = reader.read_u32().unwrap_or(0);

            for _ in 0..count {
                reader.read_byte().ok(); 
                let params = reader.read_u32().unwrap_or(0) as usize;
                reader.addr += params;
                let returns = reader.read_u32().unwrap_or(0) as usize;
                reader.addr += returns;

                self.types.push(TypeDef { params, returns });
            }
        }
    }

    // Parse function imports section (section 2 of WASM binary).
    fn parse_imports(&mut self, sections: &[Section]) {
        if let Some(section) = sections.iter().find(|s| s.id == 2) {
            let mut reader = BinReader::new(section.data.clone());
            let count = reader.read_u32().unwrap_or(0);

            for _ in 0..count {
                let mod_len = reader.read_u32().unwrap_or(0);
                let mod_name = String::from_utf8_lossy(
                    &reader.read_bytes(mod_len as usize).unwrap_or_default()
                ).to_string();

                let name_len = reader.read_u32().unwrap_or(0);
                let name = String::from_utf8_lossy(
                    &reader.read_bytes(name_len as usize).unwrap_or_default()
                ).to_string();

                let kind = reader.read_byte().unwrap_or(0);

                match kind {
                    0x00 => {
                        let type_id = reader.read_u32().unwrap_or(0) as usize;
                        self.import_names.insert(self.import_funcs_count, (mod_name, name));
                        self.func_type_indices.push(type_id);
                        self.import_funcs_count += 1;
                    }
                    0x01 => {
                        reader.read_byte().ok(); 
                        let (min, max) = Self::read_limits(&mut reader);
                        self.import_tables.push((mod_name, name, min, max));
                    }
                    0x02 => {
                        let (min, max) = Self::read_limits(&mut reader);
                        self.import_memories.push((mod_name, name, min, max));
                    }
                    0x03 => {
                        let val_type = reader.read_byte().unwrap_or(0);
                        let is_mut = reader.read_byte().unwrap_or(0) == 1;
                        self.import_globals.push((mod_name, name, val_type, is_mut));
                    }
                    _ => {}
                }
            }
        }
    }

    // Parse function type mapping section (section 3 of WASM binary).
    fn parse_func_types(&mut self, sections: &[Section]) {
        if let Some(section) = sections.iter().find(|s| s.id == 3) {
            let mut reader = BinReader::new(section.data.clone());
            let count = reader.read_u32().unwrap_or(0);

            for _ in 0..count {
                self.func_type_indices.push(reader.read_u32().unwrap_or(0) as usize);
            }
        }
    }

    // Parse tables section (section 4 of WASM binary).
    fn parse_tables(&mut self, sections: &[Section]) {
        if let Some(section) = sections.iter().find(|s| s.id == 4) {
            let mut reader = BinReader::new(section.data.clone());
            let count = reader.read_u32().unwrap_or(0);

            for _ in 0..count {
                let element_type = reader.read_byte().unwrap_or(0);
                let (min, max) = Self::read_limits(&mut reader);
                self.tables.push((element_type, min, max));
            }
        }
    }

    // Parse memory section (section 5 of WASM binary).
    fn parse_memory(&mut self, sections: &[Section]) {
        if let Some(section) = sections.iter().find(|s| s.id == 5) {
            let mut reader = BinReader::new(section.data.clone());
            let count = reader.read_u32().unwrap_or(0);

            if count > 0 {
                let (min, _) = Self::read_limits(&mut reader);
                self.initial_memory_pages = min;
            }
        }
    }

    // Parse globals section (section 6 of WASM binary).
    fn parse_globals(&mut self, sections: &[Section]) {
        if let Some(section) = sections.iter().find(|s| s.id == 6) {
            let mut reader = BinReader::new(section.data.clone());
            let count = reader.read_u32().unwrap_or(0);

            for _ in 0..count {
                let val_type = reader.read_byte().unwrap_or(0);
                let is_mut = reader.read_byte().unwrap_or(0) == 1;
                let init = self.evaluate_const_expr(&mut reader);
                self.globals.push((val_type, is_mut, init));
            }
        }
    }

    // Parse elements section (section 9 of WASM binary).
    fn parse_elements(&mut self, sections: &[Section]) {
        if let Some(section) = sections.iter().find(|s| s.id == 9) {
            let mut reader = BinReader::new(section.data.clone());
            let count = reader.read_u32().unwrap_or(0);

            for _ in 0..count {
                let flag = reader.read_u32().unwrap_or(0);

                let (is_active, table_id, offset_expr) = match flag {
                    // Active segments (0, 2, 4, 6) have an offset expression.
                    0x00 => {
                        let offset = self.evaluate_const_expr(&mut reader);
                        (true, 0, offset)
                    }
                    0x02 => {
                        let table_id = reader.read_u32().unwrap_or(0);
                        let offset = self.evaluate_const_expr(&mut reader);
                        (true, table_id, offset)
                    }
                    0x04 => {
                        let offset = self.evaluate_const_expr(&mut reader);
                        reader.read_byte().unwrap(); 
                        (true, 0, offset)
                    }
                    0x06 => {
                        let table_id = reader.read_u32().unwrap_or(0);
                        let offset = self.evaluate_const_expr(&mut reader);
                        reader.read_byte().unwrap(); 
                        (true, table_id, offset)
                    }
                    _ => (false, 0, "0".to_string()),
                };

                match flag {
                    // Passive/Declarative segments might declare an elemkind or elemtype.
                    0x01 | 0x02 | 0x03 => { reader.read_byte().ok(); }
                    0x05 | 0x06 | 0x07 => { reader.read_byte().ok(); }
                    _ => {}
                }

                let elem_count = reader.read_u32().unwrap_or(0);
                let mut funcs = Vec::new();

                for _ in 0..elem_count {
                    if flag <= 3 {
                        // Standard MVP vec(funcid).
                        funcs.push(reader.read_u32().unwrap_or(0));
                    } else {
                        // Flags 4, 5, 6, 7 utilize vec(expr) instead of vec(funcid).
                        let mut func_id = u32::MAX;

                        loop {
                            let op = reader.read_byte().unwrap_or(0);
                            match op {
                                END => break,
                                REF_FUNC => func_id = reader.read_u32().unwrap_or(0),
                                REF_NULL => { reader.read_byte().ok(); }
                                _ => {}
                            }
                        }

                        funcs.push(func_id);
                    }
                }

                // Only active segments populate the table memory array at instantiation.
                self.elements.push((is_active, table_id, offset_expr, funcs));
            }
        }
    }

    // Parse data section (section 11 of WASM binary).
    fn parse_data(&mut self, sections: &[Section]) {
        if let Some(section) = sections.iter().find(|s| s.id == 11) {
            let mut reader = BinReader::new(section.data.clone());
            let count = reader.read_u32().unwrap_or(0);

            for _ in 0..count {
                let flag = reader.read_u32().unwrap_or(0);
                
                // Active segments.
                let (is_active, offset) = match flag {
                    // Safely consume the offset expression.
                    0x00 => (true, self.evaluate_const_expr(&mut reader)),
                    0x01 => (false, "0".to_string()),
                    0x02 => {
                        let _memid = reader.read_u32().unwrap_or(0);
                        (true, self.evaluate_const_expr(&mut reader))
                    },
                    _ => (false, "0".to_string()),
                };

                let size = reader.read_u32().unwrap_or(0);
                let data = reader.read_bytes(size as usize).unwrap_or_default();

                // Push active segments to our memory map.
                self.data_segments.push((is_active, offset, data));
            }
        }
    }

    // Parse function exports section (section 7 of WASM binary).
    fn parse_exports(&mut self, sections: &[Section]) {
        if let Some(section) = sections.iter().find(|s| s.id == 7) {
            let mut reader = BinReader::new(section.data.clone());
            let count = reader.read_u32().unwrap_or(0);

            for _ in 0..count {
                let len = reader.read_u32().unwrap_or(0);
                let name = String::from_utf8_lossy(
                    &reader.read_bytes(len as usize).unwrap_or_default()
                ).to_string();

                let kind = reader.read_byte().unwrap_or(0);
                let id = reader.read_u32().unwrap_or(0) as usize;

                match kind {
                    0x00 => { self.export_names.insert(id, name); }     // Funcs
                    0x01 => { self.export_tables.insert(id, name); }    // Tables
                    0x02 => { self.export_memories.insert(id, name); }  // Memories
                    0x03 => { self.export_globals.insert(id, name); }   // Globals
                    _ => {}
                }
            }
        }
    }

    // Parse start section (section 8 of WASM binary).
    fn parse_start(&mut self, sections: &[Section]) {
        if let Some(section) = sections.iter().find(|s| s.id == 8) {
            let mut reader = BinReader::new(section.data.clone());
            self.start_func_id = Some(reader.read_u32().unwrap_or(0) as usize);
        }
    }

    // Parse customs section (section 0 of WASM binary).
    fn parse_names(&mut self, sections: &[Section]) {
        if let Some(section) = sections.iter().find(|s| s.id == 0) {
            let mut reader = BinReader::new(section.data.clone());

            let len = reader.read_u32().unwrap_or(0);
            let name_bytes = &reader.read_bytes(len as usize).unwrap_or_default();
            let name = String::from_utf8_lossy(name_bytes);

            // Only analyze the name section. This is where our custom func names are contained.
            if name == "name" {
                while reader.addr < reader.data.len() {
                    let subsection_id = reader.read_byte().unwrap_or(0);
                    let size = reader.read_u32().unwrap_or(0);

                    if subsection_id == 1 {
                        let count = reader.read_u32().unwrap_or(0);

                        for _ in 0..count {
                            let id = reader.read_u32().unwrap_or(0);
                            let len = reader.read_u32().unwrap_or(0);

                            let fname = String::from_utf8_lossy(
                                &reader.read_bytes(len as usize).unwrap_or_default()
                            ).to_string();

                            self.func_names.insert(id as usize, fname);
                        }
                    } else {
                        reader.addr += size as usize;
                    }
                }
            }
        }
    }
}
