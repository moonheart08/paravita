use core::{
    any::TypeId,
    error::Error,
    fmt::{Debug, Display},
};

use alloc::collections::TryReserveError;

pub type VmResult<T> = core::result::Result<T, VmError>;

pub enum VmError {
    StackUnderflow(),
    PopExpectedType(TypeId),
    PopExpectedObject(),
    PopExpectedArray(),
    PopExpectedUserData(TypeId),
    MemoryReserveFailed(TryReserveError),
}

impl Error for VmError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            VmError::MemoryReserveFailed(e) => Some(e),
            _ => None,
        }
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }

    //fn provide<'a>(&'a self, request: &mut core::error::Request<'a>) {}
}

impl Display for VmError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Debug for VmError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            VmError::StackUnderflow() => write!(f, "Process stack underflow."),
            VmError::PopExpectedType(ty) => write!(f, "VM expected type {ty:?} on pop."),
            VmError::PopExpectedObject() => write!(f, "VM expected object on pop."),
            VmError::PopExpectedArray() => write!(f, "VM expected array on pop."),
            VmError::PopExpectedUserData(ty) => write!(f, "VM expected userdata {ty:?} on pop."),
            VmError::MemoryReserveFailed(e) => write!(f, "{e:?}"),
        }
    }
}

impl From<TryReserveError> for VmError {
    fn from(value: TryReserveError) -> Self {
        VmError::MemoryReserveFailed(value)
    }
}
