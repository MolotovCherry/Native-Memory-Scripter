#[allow(non_snake_case)]
#[repr(C)]
pub union Ret {
    pub void: (),

    pub f32: f32,
    pub f64: f64,

    pub u8: u8,
    pub u16: u16,
    pub u32: u32,
    pub u64: u64,
    pub u128: u128,

    pub i8: i8,
    pub i16: i16,
    pub i32: i32,
    pub i64: i64,
    pub i128: i128,

    pub ptr: i64,
}
