// The BinReader contains the binary data, and an address for where the reader is currently at. Similar to an iterator structure. 
pub struct BinReader {
    pub data: Vec<u8>,
    pub addr: usize,
}

impl BinReader {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, addr: 0 }
    }

    pub fn read_byte(&mut self) -> Result<u8, String> {
        if self.addr > self.data.len() - 1 {
            return Err("Unexpected end of WASM binary".to_string());
        }
        let byte = self.data[self.addr];
        self.addr += 1;
        Ok(byte)
    }

    pub fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>, String> {
        if self.addr + count > self.data.len() {
            return Err("Unexpected end of WASM binary while reading chunk".to_string());
        }
        let chunk = self.data[self.addr..self.addr + count].to_vec();
        self.addr += count;
        Ok(chunk)
    }

    // Important note: In WebAssembly binaries, all integers are encoded with LEB128 for compression reasons. 
    // Because of this we must decode the LEB128, and cannot directly read from a definite byte count.
    
    pub fn read_u32(&mut self) -> Result<u32, String> {
        let mut result: u32 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_byte()?;
            let low_bits = (byte & 0x7F) as u32;
            if let Some(shifted) = low_bits.checked_shl(shift) {
                result |= shifted;
            }
            if (byte & 0x80) == 0 {
                break;
            }
            shift += 7;
        }
        Ok(result)
    }

    pub fn read_u64(&mut self) -> Result<u64, String> {
        let mut result: u64 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_byte()?;
            let low_bits = (byte & 0x7F) as u64;
            if let Some(shifted) = low_bits.checked_shl(shift) {
                result |= shifted;
            }
            if (byte & 0x80) == 0 {
                break;
            }
            shift += 7;
        }
        Ok(result)
    }

    pub fn read_i32(&mut self) -> Result<i32, String> {
        let mut result: i32 = 0;
        let mut shift = 0;
        let mut byte: u8;
        loop {
            byte = self.read_byte()?;
            let low_bits = (byte & 0x7F) as i32;
            if let Some(shifted) = low_bits.checked_shl(shift) {
                result |= shifted;
            }
            shift += 7;
            if (byte & 0x80) == 0 {
                break;
            }
        }
        if shift < 32 && (byte & 0x40) != 0 {
            result |= (!0i32).checked_shl(shift).unwrap_or(0);
        }
        Ok(result)
    }

    pub fn read_i64(&mut self) -> Result<i64, String> {
        let mut result: i64 = 0;
        let mut shift = 0;
        let mut byte: u8;
        loop {
            byte = self.read_byte()?;
            let low_bits = (byte & 0x7F) as i64;
            if let Some(shifted) = low_bits.checked_shl(shift) {
                result |= shifted;
            }
            shift += 7;
            if (byte & 0x80) == 0 {
                break;
            }
        }
        if shift < 64 && (byte & 0x40) != 0 {
            result |= (!0i64).checked_shl(shift).unwrap_or(0);
        }
        Ok(result)
    }

    // Floats are unencoded in WebAssembly binaries, so we may read from a set amount of bytes. 

    pub fn read_f32(&mut self) -> Result<f32, String> {
        let bytes = self.read_bytes(4)?;
        let mut arr = [0u8; 4];
        arr.copy_from_slice(&bytes);
        Ok(f32::from_le_bytes(arr))
    }

    pub fn read_f64(&mut self) -> Result<f64, String> {
        let bytes = self.read_bytes(8)?;
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&bytes);
        Ok(f64::from_le_bytes(arr))
    }
}
