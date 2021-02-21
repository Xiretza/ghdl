//! String interning.
//! Derived from:
//! https://matklad.github.io/2020/03/22/fast-simple-rust-interner.html
//!
//! It provides almost the same API as the old name_table package, but
//! with many differences in the implementation:
//! * the overhead is higher.  Because this implementation uses the standard
//!   HashMap, it needs a corresponding Vec.
//! * the strings are never moved.  In a sense, this is safer.  The reference
//!   to a string returned by lookup is always valid (... until the whole
//!   interner is destroyed/droped).  There is no lie.  In the previous
//!   implementation, they could be moved when a new string was added.
//!
//! The string should be encoded using UTF-8 (although this is not checked).
//!
//! Compared to other interners, it provides these extra features:
//! * The identifier is a u32
//! * The identifiers are allocated sequentially. So the behaviour is
//!   deterministic which is used to reserve identifiers for keywords.
//! * As a consequence, it is easy to iterate over all entries.
//!
//! TODO (in the future) :
//! * Reduce memory requirement (this will certainly need to write our own hash map)
//! * Make it multi-thread ready (an RW lock should be enough).
use std::{collections::HashMap, mem};

mod ffi;

/// Identifiers.
/// They represent a interned string (use [lookup] to get it).
/// Strings are equal iff the identifiers are equal.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct NameId(u32);

pub struct Interner {
    //  Map strings to identifiers.
    map: HashMap<&'static str, NameId>,
    //  Map identifiers to strings + info
    vec: Vec<(&'static str, u32)>,
    //  Current buffer where new strings are put.
    buf: String,
    //  Old (and full) buffers of existing strings.
    full: Vec<String>,
}

impl Interner {
    // Constructor, using [cap] bytes for the initial string buffer.
    // If you have too many characters, the buffer will be extended, so
    // this is an initial guess.
    pub fn with_capacity(cap: usize) -> Interner {
        let cap = cap.next_power_of_two();
        Interner {
            map: HashMap::default(),
            vec: Vec::new(),
            buf: String::with_capacity(cap),
            full: Vec::new(),
        }
    }

    // Return the string corresponding to [id]. Will panic if [id]
    // is not valid (eg was not returned by a method defined here).
    pub fn lookup(&self, id: NameId) -> &str {
        self.vec[id.0 as usize].0
    }

    // Return the info associated to [id].
    pub fn get_info(&self, id: NameId) -> u32 {
        self.vec[id.0 as usize].1
    }

    // Set the info associated with [id]
    pub fn set_info(&mut self, id: NameId, info: u32) {
        self.vec[id.0 as usize].1 = info;
    }

    // Return the identifier for the string [name] if it has already been interned.
    pub fn get_id(&self, name: &str) -> Option<NameId> {
        self.map.get(name).copied()
    }

    // Intern string [name] and return the corresponding identifier.
    pub fn intern(&mut self, name: &str) -> NameId {
        if let Some(&id) = self.map.get(name) {
            return id;
        }
        let name = self.alloc(name);
        self.create(name)
    }

    // Intern static string [name] and return the corresponding identifier.
    // Because the lifetime of [name] is static, the string is not copied.
    // This is a slight optimization.
    // Note: [name] should be followed by a NULL character to follow the
    // convention.
    pub fn intern_static(&mut self, name: &'static str) -> NameId {
        if let Some(&id) = self.map.get(name) {
            return id;
        }
        self.create(name)
    }

    // Intern [name] when it is known not to be present.
    // Barely useful except for initialization.
    pub fn intern_extra(&mut self, name: &str) -> NameId {
        debug_assert!(self.get_id(name).is_none());
        let name = self.alloc(name);
        let id = NameId(self.vec.len() as u32);
        self.vec.push((name, 0));
        id
    }

    // Return the last known identifier, if any.
    pub fn get_last(&self) -> Option<NameId> {
        if self.vec.is_empty() {
            None
        } else {
            Some(NameId((self.vec.len() - 1) as u32))
        }
    }

    // Internal helper: create an identifier for [name].
    fn create(&mut self, name: &'static str) -> NameId {
        let id = NameId(self.vec.len() as u32);
        self.vec.push((name, 0));
        self.map.insert(name, id);

        debug_assert!(self.lookup(id) == name);
        debug_assert!(self.intern(name) == id);

        id
    }

    // Copy the string [name].
    fn alloc(&mut self, name: &str) -> &'static str {
        let cap = self.buf.capacity();
        let len = name.len() + 1;
        if cap < self.buf.len() + len {
            // Not enough space in the current buffer.  Create a new one
            // (with at least enough space).
            let new_cap = (cap.max(len) + 1).next_power_of_two();
            let new_buf = String::with_capacity(new_cap);
            let old_buf = mem::replace(&mut self.buf, new_buf);
            self.full.push(old_buf);
        }

        let interned = {
            let start = self.buf.len();
            self.buf.push_str(name);
            //  Append a NULL so that it interfaces easily with C
            self.buf.push('\0');
            &self.buf[start..start + len - 1]
        };

        unsafe { &*(interned as *const str) }
    }
}

#[cfg(test)]
mod tests {
    use super::Interner;

    #[test]
    fn sanity() {
        let mut int = Interner::with_capacity(8);

        // make sure we're not accidentally comparing addresses by having two
        // equal strings at different addresses
        let hello = "Hello";
        let hello2 = hello.to_owned();
        let hello2 = hello2.as_str();
        assert_eq!(hello, hello2);
        assert_ne!(hello.as_ptr(), hello2.as_ptr());

        let id_hello = int.intern(hello);
        let id_world = int.intern("world");
        assert_ne!(id_hello, id_world);

        let str_hello = int.lookup(id_hello);
        assert_eq!(str_hello, hello);
        assert_eq!(str_hello, hello2);

        let id_hello2 = int.intern(hello2);
        assert_eq!(id_hello2, id_hello);
        assert_ne!(id_hello2, id_world);

        // test reallocation
        let ids = ('A'..='Z').map(|c| int.intern(&c.to_string()).0);
        assert!(ids.eq(2..=27));
    }
}
