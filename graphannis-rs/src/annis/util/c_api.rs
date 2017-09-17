use libc;
use annis::Annotation;
use annis::annostorage::Match;

macro_rules! cast_mut {
    ($x:expr) => {
        {
            unsafe {
                assert!(!$x.is_null());
                (&mut (*$x).0)
            }
        }
    };
}

macro_rules! cast_const {
    ($x:expr) => {
        {
            unsafe {
                assert!(!$x.is_null());
                (&(*$x).0)
            }
        }
    };
}

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
    pub value: annis_String,
}

#[repr(C)]
pub struct annis_Option_u32 {
    pub valid: bool,
    pub value: libc::uint32_t,
}

#[allow(non_camel_case_types)]
pub type annis_Option_StringID = annis_Option_u32;

impl annis_Option_u32 {
    pub fn from(orig: Option<u32>) -> annis_Option_u32 {
        match orig {
            Some(x) => annis_Option_u32 {
                valid: true,
                value: x,
            },
            None => annis_Option_u32 {
                valid: false,
                value: 0,
            },
        }
    }

    pub fn from_ref(orig: Option<&u32>) -> annis_Option_u32 {
        match orig {
            Some(x) => annis_Option_u32 {
                valid: true,
                value: *x,
            },
            None => annis_Option_u32 {
                valid: false,
                value: 0,
            },
        }
    }

    pub fn invalid() -> annis_Option_u32 {
        return annis_Option_u32 {
            valid: false,
            value: 0,
        };
    }

    pub fn to_option(&self) -> Option<u32> {
        match self.valid {
            true => Some(self.value),
            false => None,
        }
    }
}

#[repr(C)]
pub struct annis_Vec_Annotation {
    pub v: *const Annotation,
    pub length: libc::size_t,
}

impl annis_Vec_Annotation {
    pub fn wrap(v : &Vec<Annotation>) -> annis_Vec_Annotation {
        annis_Vec_Annotation {
            v: v.as_ptr(),
            length: v.len(),
        }
    }
}

#[repr(C)]
pub struct annis_MatchIt(pub Box<Iterator<Item = Match>>);

#[repr(C)]
pub struct annis_Option_Match {
    pub valid: bool,
    pub value: Match,
}

impl annis_Option_Match {
     pub fn from(orig: Option<Match>) -> annis_Option_Match {
         match orig {
             Some(v) => annis_Option_Match{valid: true, value: v},
             None => annis_Option_Match{valid:false, value : Match::default()},
         }
     }
}


#[no_mangle]
pub extern "C" fn annis_matchit_free(ptr: *mut annis_MatchIt) {
    // take ownership and destroy the pointer
    unsafe { Box::from_raw(ptr) };
} 

#[no_mangle]
pub extern "C" fn annis_matchit_next(ptr: *mut annis_MatchIt) -> annis_Option_Match {
    annis_Option_Match::from(cast_mut!(ptr).next())
}