use core::mem::{variant_count, size_of, MaybeUninit};

use bytemuck::{self, Pod};

use super::{Aligned, Atom};

#[repr(u8)]
#[non_exhaustive]
pub enum Operation {
    Trap = 0, // because null is bad.
    /// ( n1 n2 -- sum )
    Add(PrimOpKind),
    /// ( n1 -- sum )
    AddImm(PrimOpKind, IntOpImmediate),
    /// ( n1 n2 -- sum )
    Sub(PrimOpKind),
    /// ( n1 -- sum )
    SubImm(PrimOpKind, IntOpImmediate),
    /// ( n1 n2 -- prod )
    Mul(PrimOpKind),
    /// ( n1 -- prod )
    MulImm(PrimOpKind, IntOpImmediate),
    /// ( n1 n2 -- quot rem )
    Div(PrimOpKind),
    /// ( n1 -- quot rem )
    DivImm(PrimOpKind, IntOpImmediate),
    /// ( -- imm )
    PushImm(PrimOpKind, IntOpImmediate),
    /// ( -- imm )
    PushAtom(Atom),
    /// ( -- obj )
    MakeObject(u32),
    /// ( -- arr)
    MakeArray,
    /// ( arr idx -- val )
    IndexArray,
    /// ( arr idx -- val )
    SetArray,
    /// ( v -- )
    Drop,
    /// ( v -- v v )
    Dup,
    /// ( x y -- y x )
    Swap,
    /// ( val -- )
    /// Output to device debug.
    DebugOut,
    // the final op, used for discriminant
    __Final,
}

impl Operation {
    pub fn discriminant(&self) -> u8 {
        // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        // SAFETY2: I copypasted this from the rust std example for Discriminant<T>
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }

    /// Returns the number of kinds of operations implemented. Useful for en/de coding.
    pub fn kinds(&self) -> u8 {
        return variant_count::<Self>() as u8;
    }
}

#[repr(transparent)]
pub struct IntOpImmediate(u64);

impl IntOpImmediate {
    pub fn read_u8(&self, kind: PrimOpKind) -> u8 {
        debug_assert!(kind == PrimOpKind::U8);
        let ptr = &self.0 as *const u64 as *const u8;
        u8::from_le(unsafe { ptr.read() })
    }

    pub fn read_i8(&self, kind: PrimOpKind) -> i8 {
        debug_assert!(kind == PrimOpKind::I8);
        let ptr = &self.0 as *const u64 as *const i8;
        i8::from_le(unsafe { ptr.read() })
    }

    pub fn read_u16(&self, kind: PrimOpKind) -> u16 {
        debug_assert!(kind == PrimOpKind::U16);
        let ptr = &self.0 as *const u64 as *const u16;
        u16::from_le(unsafe { ptr.read() })
    }

    pub fn read_i16(&self, kind: PrimOpKind) -> i16 {
        debug_assert!(kind == PrimOpKind::I16);
        let ptr = &self.0 as *const u64 as *const i16;
        i16::from_le(unsafe { ptr.read() })
    }

    pub fn read_u32(&self, kind: PrimOpKind) -> u32 {
        debug_assert!(kind == PrimOpKind::U32);
        let ptr = &self.0 as *const u64 as *const u32;
        u32::from_le(unsafe { ptr.read() })
    }

    pub fn read_i32(&self, kind: PrimOpKind) -> i32 {
        debug_assert!(kind == PrimOpKind::I32);
        let ptr = &self.0 as *const u64 as *const i32;
        i32::from_le(unsafe { ptr.read() })
    }

    pub fn read_u64(&self, kind: PrimOpKind) -> u64 {
        debug_assert!(kind == PrimOpKind::U64);
        let ptr = &self.0 as *const u64;
        u64::from_le(unsafe { ptr.read() })
    }

    pub fn read_i64(&self, kind: PrimOpKind) -> i64 {
        debug_assert!(kind == PrimOpKind::I64);
        let ptr = &self.0 as *const u64 as *const i64;
        i64::from_le(unsafe { ptr.read() })
    }

    pub fn as_aligned(&self) -> Aligned {
        Aligned(bytemuck::cast(self.0.clone()))
    }

    fn slice_as_array<const N: usize>(s: &[u8]) -> [u8; N] {
        <&[u8; N]>::try_from(s).unwrap().clone()
    }
}

impl<T> From<T> for IntOpImmediate
where
    T: Pod,
{
    fn from(value: T) -> Self {
        let mut a = MaybeUninit::uninit();
        let ptr = a.as_mut_ptr() as *mut T;
        //SAFETY: Guaranteed to fit.
        unsafe {
            ptr.write(value);
        }
        return IntOpImmediate(unsafe { a.assume_init() });
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrimOpKind {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
}
