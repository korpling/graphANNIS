use libc;

#[repr(C)]
/**
A non-null terminated string.
 */
pub struct annis_String {
    pub s: *const libc::c_char,
    pub length: libc::size_t,
}

#[repr(C)]
pub struct annis_OptionalString {
    pub valid: libc::c_int,
    pub value : annis_String,
}

#[repr(C)]
pub struct annis_Option_u32 {
    pub valid: libc::c_int,
    pub value: libc::uint32_t,
}