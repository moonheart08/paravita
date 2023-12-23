use core::fmt::Debug;


use bytemuck::Pod;
use bytemuck_derive::{Pod, Zeroable};

use super::{PVObject, PrimOpKind};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(PrimOpKind, Aligned), // Due to both BE and LE platforms, and non-64bit using this type, just upcasting to u64 isn't acceptable as it'd imply the need to extend or sign extend all numeric values.
    Object(PVObject),
    Null
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C, align(8))]
#[derive(Pod, Zeroable)]
pub struct Aligned(pub [u8; 8]);

impl Value {
    pub fn reinterpret<T: Pod>(&self) -> T {
        match self {
            Value::Int(_, v) => {
                let s = bytemuck::cast_ref::<Aligned, [u8; 8]>(v).as_slice();
                bytemuck::cast_slice(s)[0]
            },
            Value::Object(_) => unimplemented!(), // hbdxjwhsgcvyexwhuxwh no
            Value::Null => T::zeroed(),
        }
    }

    pub fn from<T: Pod>(v: T, k: PrimOpKind) -> Value {
        let b = bytemuck::bytes_of(&v);
        let mut dest = [0u8; 8];
        dest.copy_from_slice(b);
        return Value::Int(k, Aligned(dest));
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Value::from(value, PrimOpKind::U64)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::from(value, PrimOpKind::I64)
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Value::from(value, PrimOpKind::U32)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::from(value, PrimOpKind::I32)
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Value::from(value, PrimOpKind::U16)
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Value::from(value, PrimOpKind::I16)
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Value::from(value, PrimOpKind::U8)
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Value::from(value, PrimOpKind::I8)
    }
}