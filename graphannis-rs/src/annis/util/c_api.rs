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

impl annis_Option_u32 {
    pub fn from(orig : Option<u32>) -> annis_Option_u32 {
        match orig {
            Some(x) => annis_Option_u32{valid: true, value: x},
            None => annis_Option_u32{valid: false, value: 0},
        }
    }

    pub fn from_ref(orig : Option<&u32>) -> annis_Option_u32 {
        match orig {
            Some(x) => annis_Option_u32{valid: true, value: *x},
            None => annis_Option_u32{valid: false, value: 0},
        }
    }

    pub fn invalid() -> annis_Option_u32 {
        return annis_Option_u32{valid: false, value: 0};
    }
}