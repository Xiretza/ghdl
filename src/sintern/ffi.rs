use crate::sintern::{Interner, NameId};
use std::slice;

// Create an interner
#[no_mangle]
pub extern "C" fn sintern_new_interner<'a>(cap: u32) -> Box<Interner<'a>> {
    let res = Interner::with_capacity(cap as usize);
    Box::new(res)
}

// Delete the interner
#[no_mangle]
pub extern "C" fn sintern_delete_interner(_: Box<Interner>) {
    //  Will be droped
}

// Return a unique id
#[no_mangle]
pub extern "C" fn sintern_get_identifier_with_len(
    inst: &mut Interner,
    name: *const u8,
    len: u32,
) -> NameId {
    inst.intern(unsafe { std::str::from_utf8_unchecked(slice::from_raw_parts(name, len as usize)) })
}

// Return a unique id if it already exists.
#[no_mangle]
pub extern "C" fn sintern_get_identifier_no_create_with_len(
    inst: &mut Interner,
    name: *const u8,
    len: u32,
) -> NameId {
    let id = inst.get_id(unsafe {
        std::str::from_utf8_unchecked(slice::from_raw_parts(name, len as usize))
    });
    if let Some(NameId(0)) = id {
        panic!("Tried to get string with ID 0, but this ID is reserved to represent the string not being present");
    }
    id.unwrap_or(NameId(0))
}

/// Return the unique id without copying the string [name].
///
/// # Safety
///
/// See [Interner::intern_static].
#[no_mangle]
pub unsafe extern "C" fn sintern_get_identifier_static_with_len(
    inst: &mut Interner,
    name: *const u8,
    len: u32,
) -> NameId {
    inst.intern_static(std::str::from_utf8_unchecked(slice::from_raw_parts(
        name,
        len as usize,
    )))
}

/// Intern `name` without adding it to the `string->id` lookup table.
///
/// # Safety
///
/// See [Interner::intern_extra].
#[no_mangle]
pub unsafe extern "C" fn sintern_get_identifier_extra_with_len(
    inst: &mut Interner,
    name: *const u8,
    len: u32,
) -> NameId {
    inst.intern_extra(std::str::from_utf8_unchecked(slice::from_raw_parts(
        name,
        len as usize,
    )))
}

// Get the C string for the [id].  It is NULL terminated.
#[no_mangle]
pub extern "C" fn sintern_get_address(inst: &Interner, id: NameId) -> *const u8 {
    inst.lookup(id).as_ptr()
}

// Get the length of the identifier (in bytes).
#[no_mangle]
pub extern "C" fn sintern_get_length(inst: &Interner, id: NameId) -> u32 {
    inst.lookup(id).len() as u32
}

// Get the last known identifier.
#[no_mangle]
pub extern "C" fn sintern_get_last(inst: &Interner) -> NameId {
    inst.get_last()
        .expect("Tried to get last string ID, but interner contains no strings")
}

#[no_mangle]
pub extern "C" fn sintern_get_info(inst: &Interner, id: NameId) -> u32 {
    inst.get_info(id)
}

#[no_mangle]
pub extern "C" fn sintern_set_info(inst: &mut Interner, id: NameId, info: u32) {
    inst.set_info(id, info)
}
