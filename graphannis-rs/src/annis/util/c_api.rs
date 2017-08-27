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
pub struct annis_Option_String {
    pub valid: bool,
    pub value : annis_String,
}

#[repr(C)]
pub struct annis_Option_u32 {
    pub valid: bool,
    pub value: libc::uint32_t,
}