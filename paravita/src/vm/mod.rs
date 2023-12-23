mod atoms;
mod error;
mod object;
mod opcodes;
mod value;
use core::any::TypeId;
use core::cell::{Ref, RefMut};
use core::net::Ipv6Addr;
use core::ops::DerefMut;
use core::{alloc::AllocError, ops::Deref};
#[cfg(std)]
use std::println;

use alloc::vec::Vec;
pub use atoms::*;
use bytemuck::Pod;
use num::{
    traits::{WrappingAdd, WrappingMul, WrappingSub},
    FromPrimitive, Integer,
};
pub use object::*;
pub use opcodes::*;
use portable_atomic::AtomicU64;
use tinyvec::TinyVec;
pub use value::*;

use self::error::{VmError, VmResult};

static PROC_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct Process {
    pid: Ipv6Addr,
    stack: Vec<Value>,
}

impl Process {
    #[must_use]
    pub fn new(prefix: Ipv6Addr) -> Result<Process, AllocError> {
        let count = PROC_COUNTER.fetch_add(1, portable_atomic::Ordering::Relaxed);
        let mut segs = prefix.segments();
        segs[7] = (count & 0xffff) as u16;
        segs[6] = ((count >> 16) & 0xffff) as u16;
        segs[5] = ((count >> 32) & 0xffff) as u16;
        segs[4] = ((count >> 48) & 0xffff) as u16;
        Ok(Process {
            pid: segs.into(),
            stack: Vec::new(),
        })
    }

    #[inline(never)]
    #[must_use]
    pub(super) fn pop(&mut self) -> VmResult<Value> {
        self.stack.pop().ok_or(VmError::StackUnderflow())
    }

    
    #[inline(never)]
    #[must_use]
    pub(super) fn pop_into(&mut self, into: &mut Value) -> VmResult<()> {
        *into = self.stack.pop().ok_or(VmError::StackUnderflow())?;
        Ok(())
    }

    #[inline(never)]
    pub(super) unsafe fn pop_unchecked(&mut self) -> Value {
        let len = self.stack.len()-1;
        let v = core::mem::take(self.stack.get_unchecked_mut(len));
        self.stack.set_len(len);
        return v;
    }

    #[must_use]
    pub(super) fn pop2(&mut self) -> VmResult<(Value, Value)> {
        if self.stack.len() < 2 {
            return Err(VmError::StackUnderflow());
        }

        unsafe { Ok((self.pop_unchecked(), self.pop_unchecked())) }
    }

    #[must_use]
    pub(super) fn pop2_into(&mut self, x: &mut Value, y: &mut Value) -> VmResult<()> {
        if self.stack.len() < 2 {
            return Err(VmError::StackUnderflow());
        }

        unsafe {
            *x = self.pop_unchecked();
            *y = self.pop_unchecked();
        }
        Ok(())
    }


    #[must_use]
    pub(super) fn pop3(&mut self) -> VmResult<(Value, Value, Value)> {
        Ok((self.pop()?, self.pop()?, self.pop()?))
    }

    pub(super) fn push(&mut self, v: Value) {
        self.stack.push(v)
    }

    pub fn run_op(&mut self, o: Operation) -> VmResult<()> {
        match o {
            Operation::Trap => todo!(),
            Operation::Add(k) => {
                fn add<T: WrappingAdd>(x: T, y: T) -> T {
                    x.wrapping_add(&y)
                }

                let mut x = Value::Null;
                let mut y = Value::Null;
                self.pop2_into(&mut x, &mut y)?;
                let v = match k {
                    PrimOpKind::U8 => add::<u8>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::I8 => add::<i8>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::U16 => add::<u16>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::I16 => add::<i16>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::U32 => add::<u32>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::I32 => add::<i32>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::U64 => add::<u64>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::I64 => add::<i64>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                };

                self.push(v);
            }
            Operation::AddImm(k, imm) => {
                fn add<T: WrappingAdd>(x: T, y: T) -> T {
                    x.wrapping_add(&y)
                }

                let mut x = Value::Null;
                self.pop_into(&mut x)?;
                let v = match k {
                    PrimOpKind::U8 => add::<u8>(Self::as_num(&x)?, imm.read_u8(k)).into(),
                    PrimOpKind::I8 => add::<i8>(Self::as_num(&x)?, imm.read_i8(k)).into(),
                    PrimOpKind::U16 => add::<u16>(Self::as_num(&x)?, imm.read_u16(k)).into(),
                    PrimOpKind::I16 => add::<i16>(Self::as_num(&x)?, imm.read_i16(k)).into(),
                    PrimOpKind::U32 => add::<u32>(Self::as_num(&x)?, imm.read_u32(k)).into(),
                    PrimOpKind::I32 => add::<i32>(Self::as_num(&x)?, imm.read_i32(k)).into(),
                    PrimOpKind::U64 => add::<u64>(Self::as_num(&x)?, imm.read_u64(k)).into(),
                    PrimOpKind::I64 => add::<i64>(Self::as_num(&x)?, imm.read_i64(k)).into(),
                };

                self.push(v);
            }
            Operation::Sub(k) => {
                fn sub<T: WrappingSub>(x: T, y: T) -> T {
                    x.wrapping_sub(&y)
                }

                let mut x = Value::Null;
                let mut y = Value::Null;
                self.pop2_into(&mut x, &mut y)?;
                let v = match k {
                    PrimOpKind::U8 => sub::<u8>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::I8 => sub::<i8>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::U16 => sub::<u16>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::I16 => sub::<i16>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::U32 => sub::<u32>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::I32 => sub::<i32>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::U64 => sub::<u64>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::I64 => sub::<i64>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                };

                self.push(v);
            }
            Operation::SubImm(k, imm) => {
                fn sub<T: WrappingSub>(x: T, y: T) -> T {
                    x.wrapping_sub(&y)
                }

                let mut x = Value::Null;
                self.pop_into(&mut x)?;
                let v = match k {
                    PrimOpKind::U8 => sub::<u8>(Self::as_num(&x)?, imm.read_u8(k)).into(),
                    PrimOpKind::I8 => sub::<i8>(Self::as_num(&x)?, imm.read_i8(k)).into(),
                    PrimOpKind::U16 => sub::<u16>(Self::as_num(&x)?, imm.read_u16(k)).into(),
                    PrimOpKind::I16 => sub::<i16>(Self::as_num(&x)?, imm.read_i16(k)).into(),
                    PrimOpKind::U32 => sub::<u32>(Self::as_num(&x)?, imm.read_u32(k)).into(),
                    PrimOpKind::I32 => sub::<i32>(Self::as_num(&x)?, imm.read_i32(k)).into(),
                    PrimOpKind::U64 => sub::<u64>(Self::as_num(&x)?, imm.read_u64(k)).into(),
                    PrimOpKind::I64 => sub::<i64>(Self::as_num(&x)?, imm.read_i64(k)).into(),
                };

                self.push(v);
            }
            Operation::Mul(k) => {
                fn mul<T: WrappingMul>(x: T, y: T) -> T {
                    x.wrapping_mul(&y)
                }

                let mut x = Value::Null;
                let mut y = Value::Null;
                self.pop2_into(&mut x, &mut y)?;
                let v = match k {
                    PrimOpKind::U8 => mul::<u8>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::I8 => mul::<i8>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::U16 => mul::<u16>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::I16 => mul::<i16>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::U32 => mul::<u32>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::I32 => mul::<i32>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::U64 => mul::<u64>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                    PrimOpKind::I64 => mul::<i64>(Self::as_num(&x)?, Self::as_num(&y)?).into(),
                };

                self.push(v);
            }
            Operation::MulImm(k, imm) => {
                fn mul<T: WrappingMul>(x: T, y: T) -> T {
                    x.wrapping_mul(&y)
                }

                let mut x = Value::Null;
                self.pop_into(&mut x)?;
                let v = match k {
                    PrimOpKind::U8 => mul::<u8>(Self::as_num(&x)?, imm.read_u8(k)).into(),
                    PrimOpKind::I8 => mul::<i8>(Self::as_num(&x)?, imm.read_i8(k)).into(),
                    PrimOpKind::U16 => mul::<u16>(Self::as_num(&x)?, imm.read_u16(k)).into(),
                    PrimOpKind::I16 => mul::<i16>(Self::as_num(&x)?, imm.read_i16(k)).into(),
                    PrimOpKind::U32 => mul::<u32>(Self::as_num(&x)?, imm.read_u32(k)).into(),
                    PrimOpKind::I32 => mul::<i32>(Self::as_num(&x)?, imm.read_i32(k)).into(),
                    PrimOpKind::U64 => mul::<u64>(Self::as_num(&x)?, imm.read_u64(k)).into(),
                    PrimOpKind::I64 => mul::<i64>(Self::as_num(&x)?, imm.read_i64(k)).into(),
                };

                self.push(v);
            }
            Operation::Div(_k) => todo!(),
            Operation::DivImm(_k, _imm) => todo!(),
            Operation::PushImm(k, v) => self.push(Value::Int(k, v.as_aligned())),
            Operation::PushAtom(a) => self.push(Value::Object(PVObject::from(a))),
            Operation::MakeObject(_) => self.push(Value::Object(PVObject::make_map()?)),
            Operation::MakeArray => self.push(Value::Object(PVObject::make_array()?)),
            Operation::IndexArray => {
                let mut arr = Value::Null;
                let mut idx = Value::Null;
                self.pop2_into(&mut arr, &mut idx)?;
                let idx = Self::as_num::<usize>(&idx)?;
                let arr = Self::as_list(&arr)?;
                self.push(arr.deref().load(idx).unwrap_or(Value::Null));
            }
            Operation::SetArray => {
                let (arr, idx, value) = self.pop3()?;
                let idx = Self::as_num::<usize>(&idx)?;
                let mut arr = Self::as_list_mut(&arr)?;
                arr.deref_mut().store(idx, value)?;
            }
            Operation::Drop => {
                core::mem::drop(self.pop());
            }
            Operation::Dup => {
                let mut x = Value::Null;
                self.pop_into(&mut x)?;
                self.push(x.clone());
                self.push(x)
            }
            Operation::Swap => {
                let (x, y) = self.pop2()?;
                self.push(y);
                self.push(x);
            }
            Operation::DebugOut => {
                #[cfg(std)]
                {
                    println!("{:?}", self.pop());
                }
            }
            Operation::__Final => todo!(),
        }
        Ok(())
    }

    fn as_num<T>(v: &Value) -> VmResult<T>
    where
        T: Integer + Pod + FromPrimitive,
    {
        if let Value::Int(_, _) = v {
            Ok(v.reinterpret())
        } else {
            Err(error::VmError::PopExpectedType())
        }
    }

    fn as_list(v: &Value) -> VmResult<Ref<'_, PVObjectType>> {
        if let Value::Object(o) = v {
            let r = o.get();
            if let PVObjectType::Array(_) = r.deref() {
                return Ok(r);
            }
        }

        Err(error::VmError::PopExpectedArray())
    }

    fn as_list_mut(v: &Value) -> VmResult<RefMut<'_, PVObjectType>> {
        if let Value::Object(o) = v {
            let r = o.get_mut();
            if let PVObjectType::Array(_) = r.deref() {
                return Ok(r);
            }
        }

        Err(error::VmError::PopExpectedArray())
    }
}

#[cfg(test)]
mod tests {
    use core::net::Ipv6Addr;

    use alloc::vec;

    use super::{Operation, PrimOpKind, Process, error::VmResult};

    #[test]
    pub fn add() -> VmResult<()> {
        let prog = vec![
            Operation::PushImm(PrimOpKind::I64, 1i64.into()),
            Operation::PushImm(PrimOpKind::I64, 1i64.into()),
            Operation::Add(PrimOpKind::I64),
        ];

        let mut process = Process::new(Ipv6Addr::UNSPECIFIED).unwrap();

        for i in prog {
            process.run_op(i).unwrap();
        }

        assert!(process.stack.len() == 1);
        assert!(Process::as_num::<u64>(&process.pop()?)? == 2u64);
        Ok(())
    }
}
