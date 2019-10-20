use crate::malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use rustc_hash;
use std;

pub fn shallow_size_of_fxhashmap<K, V>(
    val: &rustc_hash::FxHashMap<K, V>,
    ops: &mut MallocSizeOfOps,
) -> usize {
    if ops.has_malloc_enclosing_size_of() {
        val.values()
            .next()
            .map_or(0, |v| unsafe { ops.malloc_enclosing_size_of(v) })
    } else {
        val.capacity()
            * (std::mem::size_of::<V>() + std::mem::size_of::<K>() + std::mem::size_of::<usize>())
    }
}

pub fn size_of_btreemap<K, V>(
    val: &std::collections::BTreeMap<K, V>,
    ops: &mut MallocSizeOfOps,
) -> usize
where
    K: MallocSizeOf,
    V: MallocSizeOf,
{
    // FIXME: Overhead for the BTreeMap nodes is not accounted for.
    let mut size = 0;
    for (key, value) in val.iter() {
        size += std::mem::size_of::<(K, V)>() + key.size_of(ops) + value.size_of(ops);
    }
    size
}

#[cfg(not(windows))]
pub mod platform {
    use std::os::raw::c_void;

    /// Defines which actual function is used.
    ///
    /// We always use the system malloc instead of jemalloc.
    /// On MacOS X, the external function is not called "malloc_usable_size", but "malloc_size"
    /// (it basically does the same).
    extern "C" {
        #[cfg_attr(any(target_os = "macos", target_os = "ios"), link_name = "malloc_size")]
        fn malloc_usable_size(ptr: *const c_void) -> usize;
    }

    /// Get the size of a heap block.
    pub unsafe extern "C" fn usable_size(ptr: *const c_void) -> usize {
        if ptr.is_null() {
            0
        } else {
            malloc_usable_size(ptr)
        }
    }
}

#[cfg(windows)]
pub mod platform {
    extern crate kernel32;
    use self::kernel32::{GetProcessHeap, HeapSize, HeapValidate};
    use std::os::raw::c_void;

    /// Get the size of a heap block.
    pub unsafe extern "C" fn usable_size(mut ptr: *const c_void) -> usize {
        let heap = GetProcessHeap();

        if HeapValidate(heap, 0, ptr) == 0 {
            ptr = *(ptr as *const *const c_void).offset(-1);
        }

        HeapSize(heap, 0, ptr) as usize
    }
}
