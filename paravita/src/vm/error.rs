use core::{
    any::TypeId,
    error::Error,
    fmt::{Debug, Display}, alloc::AllocError,
};

use alloc::collections::TryReserveError;

pub type VmResult<T> = core::result::Result<T, VmError>;

pub enum VmError {
    StackUnderflow(),
    PopExpectedType(),
    PopExpectedObject(),
    PopExpectedArray(),
    PopExpectedUserData(),
    MemoryReserveFailed(TryReserveError),
    MemoryAllocFailed(AllocError),
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
            VmError::PopExpectedType() => write!(f, "VM expected value on pop."),
            VmError::PopExpectedObject() => write!(f, "VM expected object on pop."),
            VmError::PopExpectedArray() => write!(f, "VM expected array on pop."),
            VmError::PopExpectedUserData() => write!(f, "VM expected userdata on pop."),
            VmError::MemoryReserveFailed(e) => write!(f, "{e:?}"),
            VmError::MemoryAllocFailed(e) => write!(f, "{e:?}"),
        }
    }
}

impl From<TryReserveError> for VmError {
    fn from(value: TryReserveError) -> Self {
        VmError::MemoryReserveFailed(value)
    }
}

impl From<AllocError> for VmError {
    fn from(value: AllocError) -> Self {
        VmError::MemoryAllocFailed(value)
    }
}
