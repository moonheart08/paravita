use core::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    fmt::Debug,
    ops::Deref,
};

use alloc::{rc::Rc, string::String, vec::Vec};
use fnv::FnvBuildHasher;
use indexmap::IndexMap;

use super::{
    error::{VmError, VmResult},
    Atom, Value,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PVString {
    Atom(Atom),
    Str(String),
}

/// A reference to a Paravita object. To clone the inner object, call [duplicate()]
#[derive(Debug, PartialEq, Clone)]
pub struct PVObject {
    cell: Rc<RefCell<PVObjectType>>,
}

impl PVObject {
    pub fn get(&self) -> Ref<PVObjectType> {
        self.cell.borrow()
    }

    pub fn get_mut(&self) -> RefMut<PVObjectType> {
        self.cell.borrow_mut()
    }

    fn build_handle(h: PVObjectType) -> VmResult<Self> {
        Ok(Self {
            
            cell: Rc::try_new(RefCell::new(h))?,
        })
    }

    pub fn make_map() -> VmResult<Self> {
        //MEMSAFETY: IndexMap default is infalliable.
        let inner = PVObjectType::Map(IndexMap::default());
        Self::build_handle(inner)
    }

    pub fn make_array() -> VmResult<Self> {
        //MEMSAFETY: Vec::new() is infalliable.
        let inner = PVObjectType::Array(Vec::new());
        Self::build_handle(inner)
    }

    pub fn duplicate(&self) -> VmResult<Self> {
        // Clone the interior object
        //MEMSAFETY: We need a better API for this, clone is falliable by panic.
        let inner = (*self.cell.borrow().deref()).clone();
        // and construct a handle to the clone.
        Self::build_handle(inner)
    }
}

#[derive(Debug, Clone)]
pub enum PVObjectType {
    Map(IndexMap<PVString, Value, FnvBuildHasher>),
    Array(Vec<Value>),
    String(PVString),
    UserData(Rc<dyn PVUserData>),
}
impl PartialEq for PVObjectType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Map(l0), Self::Map(r0)) => l0 == r0,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            /* never equal. */
            (Self::UserData(_), Self::UserData(_)) => false,
            _ => false,
        }
    }
}

impl PVObjectType {
    pub fn load(&self, idx: usize) -> Option<Value> {
        match self {
            PVObjectType::Map(_) => None,
            PVObjectType::String(_) => None,
            PVObjectType::Array(v) => v.get(idx).map(|x| x.clone()),
            PVObjectType::UserData(_) => None,
        }
    }

    pub fn store(&mut self, idx: usize, value: Value) -> VmResult<()> {
        match self {
            PVObjectType::Map(_) => Ok(()),
            PVObjectType::String(_) => Ok(()),
            PVObjectType::Array(v) => {
                if v.len() >= idx {
                    let elems = idx - v.len() + 1;
                    v.try_reserve(elems)?;
                    v.extend([Value::Null; 1].iter().cloned().cycle().take(elems));
                }
                v[idx] = value;
                Ok(())
            }
            PVObjectType::UserData(_) => Ok(()),
        }
    }
}

impl From<Atom> for PVObject {
    fn from(value: Atom) -> Self {
        Self {
            cell: Rc::new(RefCell::new(PVObjectType::String(PVString::Atom(value)))),
        }
    }
}

pub trait PVUserData: Any + Debug {}
