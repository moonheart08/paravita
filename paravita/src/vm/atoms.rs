use alloc::{boxed::Box, string::ToString};
use async_lock::*;
use bytemuck::Contiguous;
use core::hash::Hash;
use core::{
    num::NonZeroU32,
    ops::{Deref, DerefMut},
};
use indexmap::IndexSet;
use once_cell::race::OnceBox;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Atom {
    handle: NonZeroU32,
}

static ATOM_STORE: OnceBox<AtomStore> = OnceBox::new();

impl From<Atom> for &'static str {
    fn from(value: Atom) -> Self {
        AtomStore::read(value)
    }
}

impl From<&str> for Atom {
    fn from(value: &str) -> Self {
        AtomStore::insert(value)
    }
}

struct AtomStore {
    // SAFETY: DO NOT REMOVE ATOMS FROM THE SET. Shit explodes!
    atoms: RwLock<IndexSet<&'static str, fnv::FnvBuildHasher>>,
}

unsafe impl Sync for AtomStore {}

impl AtomStore {
    fn map(&self) -> RwLockReadGuard<'_, IndexSet<&'static str, fnv::FnvBuildHasher>> {
        let idx = loop {
            if let Some(x) = self.atoms.try_read() {
                break x;
            }
        };

        return idx;
    }

    fn mut_map(&self) -> RwLockWriteGuard<'_, IndexSet<&'static str, fnv::FnvBuildHasher>> {
        let idx = loop {
            if let Some(x) = self.atoms.try_write() {
                break x;
            }
        };

        return idx;
    }

    pub fn insert(s: &str) -> Atom {
        let this = Self::get();
        // Necessary block to prevent deadlock over handles.
        {
            let idx = this.map();
            // rust analyzer is all kinds of sad about RwLock for some reason so this is here so it chokes less.
            let idx: &IndexSet<&'static str, fnv::FnvBuildHasher> = idx.deref();
            if let Some(i) = idx.get_index_of(s) {
                return Atom {
                    handle: unsafe { NonZeroU32::new_unchecked(i as u32 + 1) },
                };
            }

            // MAP LOCK DROPPED HERE
        }

        let mut idx = this.mut_map();
        let idx: &mut IndexSet<&'static str, fnv::FnvBuildHasher> = idx.deref_mut();

        // MEMSAFETY: This /will/ explode on OOM. FIXME!
        let (i, _) = idx.insert_full(s.to_string().leak());

        return Atom {
            handle: unsafe { NonZeroU32::new_unchecked(i as u32 + 1) },
        };
    }

    fn get<'a>() -> &'a Self {
        ATOM_STORE.get_or_init(Self::init)
    }

    fn init() -> Box<AtomStore> {
        Box::new({
            AtomStore {
                atoms: RwLock::new(IndexSet::with_hasher(fnv::FnvBuildHasher::default())),
            }
        })
    }

    pub fn read(h: Atom) -> &'static str {
        let idx = Self::get().map();
        let idx: &IndexSet<&'static str, fnv::FnvBuildHasher> = idx.deref();
        idx[h.handle.into_integer() as usize - 1]
    }
}

// Returns the number of atoms the VM has in cache.
pub fn atoms_count() -> usize {
    AtomStore::get().map().len()
}

impl Hash for Atom {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.handle.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::{atoms_count, Atom};

    #[test]
    pub fn insert_get() {
        let foo_atom = Atom::from("foo");
        let bar_atom = Atom::from("bar");

        assert_eq!("foo", <Atom as Into<&str>>::into(foo_atom));
        let baz_atom = Atom::from("baz");
        assert_eq!("bar", <Atom as Into<&str>>::into(bar_atom));
        assert_eq!("baz", <Atom as Into<&str>>::into(baz_atom));
        assert_eq!(atoms_count(), 3);
    }
}
