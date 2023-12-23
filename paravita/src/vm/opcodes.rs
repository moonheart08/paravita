use core::mem::variant_count;

use bytemuck::{self, Pod};

use super::{Atom, Aligned};

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
pub struct IntOpImmediate([u8; 8]);

impl IntOpImmediate {
    pub fn read_u8(&self, kind: PrimOpKind) -> u8 {
        assert!(kind == PrimOpKind::U8);
        self.read()
    }

    pub fn read_i8(&self, kind: PrimOpKind) -> i8 {
        assert!(kind == PrimOpKind::I8);
        self.read()
    }

    pub fn read_u16(&self, kind: PrimOpKind) -> u16 {
        assert!(kind == PrimOpKind::U16);
        self.read()
    }

    pub fn read_i16(&self, kind: PrimOpKind) -> i16 {
        assert!(kind == PrimOpKind::I16);
        self.read()
    }

    pub fn read_u32(&self, kind: PrimOpKind) -> u32 {
        assert!(kind == PrimOpKind::U32);
        self.read()
    }

    pub fn read_i32(&self, kind: PrimOpKind) -> i32 {
        assert!(kind == PrimOpKind::I32);
        self.read()
    }

    pub fn read_u64(&self, kind: PrimOpKind) -> u64 {
        assert!(kind == PrimOpKind::U64);
        self.read()
    }

    pub fn read_i64(&self, kind: PrimOpKind) -> i64 {
        assert!(kind == PrimOpKind::I64);
        self.read()
    }

    pub fn read<N>(&self) -> N 
        where N: bytemuck::Pod, [u8; core::mem::size_of::<N>()]: bytemuck::Pod
    {
        bytemuck::cast::<[u8; core::mem::size_of::<N>()], N>(Self::slice_as_array(&self.0[0..core::mem::size_of::<u16>()]))
    }

    pub fn as_aligned(&self) -> Aligned {
        Aligned(self.0)
    }

    fn slice_as_array<const N: usize>(s: &[u8]) -> [u8; N] {
        <&[u8; N]>::try_from(s).unwrap().clone()
    }
}

impl<T> From<T> for IntOpImmediate
    where T: Pod 
{
    fn from(value: T) -> Self {
        let mut dest = [0u8; 8];
        let s = bytemuck::bytes_of(&value);
        dest.copy_from_slice(s);
        return IntOpImmediate(dest);
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
    I64
}
