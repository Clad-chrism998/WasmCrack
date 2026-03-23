// The standard opcodes cannot cover all operations, so WASM utilizes an extension with the FC prefix.
// It contains extra WebAssembly operation with sub-opcodes as provided here.

pub const I32_TRUNC_SAT_F32_S: u32 = 0x00;
pub const I32_TRUNC_SAT_F32_U: u32 = 0x01;
pub const I32_TRUNC_SAT_F64_S: u32 = 0x02;
pub const I32_TRUNC_SAT_F64_U: u32 = 0x03;
pub const I64_TRUNC_SAT_F32_S: u32 = 0x04;
pub const I64_TRUNC_SAT_F32_U: u32 = 0x05;
pub const I64_TRUNC_SAT_F64_S: u32 = 0x06;
pub const I64_TRUNC_SAT_F64_U: u32 = 0x07;
pub const MEMORY_INIT: u32 = 0x08;
pub const DATA_DROP: u32 = 0x09;
pub const MEMORY_COPY: u32 = 0x0A;
pub const MEMORY_FILL: u32 = 0x0B;
pub const TABLE_INIT: u32 = 0x0C;
pub const ELEM_DROP: u32 = 0x0D;
pub const TABLE_COPY: u32 = 0x0E;
pub const TABLE_GROW: u32 = 0x0F;
pub const TABLE_SIZE: u32 = 0x10;
pub const TABLE_FILL: u32 = 0x11;