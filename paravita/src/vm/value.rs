use core::{fmt::Debug, mem::{size_of, align_of, MaybeUninit, discriminant}, any::TypeId};

use bytemuck::Pod;
use bytemuck_derive::{Pod, Zeroable};

use super::{PVObject, PrimOpKind};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Int(PrimOpKind, Aligned), // Due to both BE and LE platforms, and non-64bit using this type, just upcasting to u64 isn't acceptable as it'd imply the need to extend or sign extend all numeric values.
    Object(PVObject),
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C, align(8))]
#[derive(Pod, Zeroable)]
pub struct Aligned(pub u64);

impl Value {
    pub fn reinterpret<T: Pod>(&self) -> T {
        if (size_of::<T>() > size_of::<Aligned>()) || (align_of::<T>() > align_of::<Aligned>()) { panic!("Tried to read a value too large or improperly aligned to be contained in Value! {:?}", TypeId::of::<T>())}
        match self {
            Value::Int(_, v) => {
                let ptr = v as *const Aligned as *const T;
                return unsafe { ptr.read() }
            }
            Value::Object(_) => T::zeroed(), // hbdxjwhsgcvyexwhuxwh no
            Value::Null => T::zeroed(),
        }
    }

    pub fn from<T: Pod>(v: T, k: PrimOpKind) -> Value {
        if (size_of::<T>() > size_of::<Aligned>()) || (align_of::<T>() > align_of::<Aligned>()) { panic!("Tried to create a value too large or improperly aligned to be contained in Value! {:?}", TypeId::of::<T>())}

        let mut a = MaybeUninit::uninit();
        let ptr = a.as_mut_ptr() as *mut T;
        //SAFETY: Guaranteed to fit.
        unsafe {
            ptr.write(v);
        }
        //SAFETY: Minor UB, may leak parts of stack. But safe as far as I'm concerned :^)
        return Value::Int(k, unsafe { a.assume_init() });
    }

    pub fn is_null(&self) -> bool {
        discriminant(self) == discriminant(&Value::Null)
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