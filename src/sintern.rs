//! String interning.
//! Derived from:
//! <https://matklad.github.io/2020/03/22/fast-simple-rust-interner.html>
use std::{collections::HashMap, mem};

mod ffi;

/// Identifiers.
/// They represent a interned string and can be obtained using the [`Interner::lookup`]
/// family of methods.
/// Strings are equal iff the identifiers are equal.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(C)]
pub struct NameId(u32);
type Info = u32;

/// The interner provides almost the same API as the old `name_table` package, but
/// with many differences in the implementation:
/// * the overhead is higher.  Because this implementation uses the standard
///   [`HashMap`], it needs a corresponding [`Vec`].
/// * the strings are never moved.  In a sense, this is safer.  The reference
///   to a string returned by lookup is valid as long as the interner is live.
///   There is no lie.  In the previous implementation, they could be moved when
///   a new string was added.
///
/// Compared to other interners, it provides these extra features:
/// * The identifier is a [`u32`]
/// * The identifiers are allocated sequentially. So the behaviour is
///   deterministic which is used to reserve identifiers for keywords.
/// * As a consequence, it is easy to iterate over all entries.
///
/// TODO (in the future) :
/// * Reduce memory requirement (this will certainly need to write our own hash map)
/// * Make it multi-thread ready (an [`RwLock`][std::sync::RwLock] should be enough).
pub struct Interner<'a> {
    ///  Map strings to identifiers.
    map: HashMap<&'a str, NameId>,
    ///  Map identifiers to strings + info
    vec: Vec<(&'a str, Info)>,
    ///  Current buffer where new strings are put.
    buf: String,
    ///  Old (and full) buffers of existing strings.
    full: Vec<String>,
}

impl<'a> Interner<'a> {
    /// Constructor, using `cap` bytes for the initial string buffer.
    /// If you have too many characters, the buffer will be extended, so
    /// this is an initial guess.
    pub fn with_capacity(cap: usize) -> Interner<'a> {
        let cap = cap.next_power_of_two();
        Interner {
            map: HashMap::default(),
            vec: Vec::new(),
            buf: String::with_capacity(cap),
            full: Vec::new(),
        }
    }

    /// Return the string corresponding to `id`.
    ///
    /// The returned string slice is guaranteed to be followed by a NULL byte, allowing
    /// its pointer to be passed to C code as-is.
    ///
    /// # Panics
    ///
    /// Panics if `id` is not valid (i.e. was not returned by a method of this instance).
    pub fn lookup(&self, id: NameId) -> &'a str {
        self.vec[id.0 as usize].0
    }

    /// Return the info associated with `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is not valid (i.e. was not returned by a method of this instance).
    pub fn get_info(&self, id: NameId) -> Info {
        self.vec[id.0 as usize].1
    }

    /// Set the info associated with `id`
    ///
    /// # Panics
    ///
    /// Panics if `id` is not valid (i.e. was not returned by a method of this instance).
    pub fn set_info(&mut self, id: NameId, info: Info) {
        self.vec[id.0 as usize].1 = info;
    }

    /// Return the identifier for the string `name` if it has already been interned.
    pub fn get_id(&self, name: &str) -> Option<NameId> {
        self.map.get(name).copied()
    }

    /// Intern string `name` and return the corresponding identifier.
    pub fn intern(&mut self, name: &str) -> NameId {
        if let Some(&id) = self.map.get(name) {
            return id;
        }
        let name = self.alloc(name);
        self.create(name)
    }

    /// Intern static string `name` and return the corresponding identifier.
    /// Because the lifetime of `name` is static, the string is not copied.
    /// This is a slight optimization.
    ///
    /// # Safety
    ///
    /// `name` must be followed by a NULL character to uphold the guarantees of [`lookup`].
    ///
    /// [`lookup`]: Interner::lookup
    pub unsafe fn intern_static(&mut self, name: &'a str) -> NameId {
        if let Some(&id) = self.map.get(name) {
            return id;
        }
        self.create(name)
    }

    /// Intern `name` without adding it to the `string->id` lookup table.
    /// Barely useful except for initialization.
    ///
    /// # Safety
    ///
    /// This method breaks the invariant of [`NameId`], since the ID returned by this method
    /// will be different from an ID returned by a subsequent [`intern`] call with the same
    /// string. Use at your own risk.
    ///
    /// [`intern`]: Interner::intern
    pub unsafe fn intern_extra(&mut self, name: &str) -> NameId {
        debug_assert!(self.get_id(name).is_none());
        let name = self.alloc(name);
        let id = NameId(self.vec.len() as u32);
        self.vec.push((name, 0));
        id
    }

    /// Return the last known identifier, if any.
    pub fn get_last(&self) -> Option<NameId> {
        if self.vec.is_empty() {
            None
        } else {
            Some(NameId((self.vec.len() - 1) as u32))
        }
    }

    // Internal helper: create an identifier for [name].
    fn create(&mut self, name: &'a str) -> NameId {
        let id = NameId(self.vec.len() as u32);
        self.vec.push((name, 0));
        self.map.insert(name, id);

        debug_assert!(self.lookup(id) == name);
        debug_assert!(self.intern(name) == id);

        id
    }

    // Copy the string [name].
    fn alloc(&mut self, name: &str) -> &'a str {
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

        let str_world = int.lookup(id_world);
        assert_ne!(str_hello, str_world);

        // test reallocation
        let ids = ('A'..='Z').map(|c| int.intern(&c.to_string()).0);
        assert!(ids.eq(2..=27));
    }
}
