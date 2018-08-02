use std::collections::BTreeSet;
use std::mem::{size_of};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};

#[cfg(not(windows))]
pub mod platform {
    extern crate jemalloc_sys as ffi;
    use std::os::raw::{c_void};

    /// Get the size of a heap block.
    pub unsafe extern "C" fn usable_size(ptr: *const c_void) -> usize {
        ffi::malloc_usable_size(ptr as *const _)
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


pub fn size_of_btreeset<V>(s : &BTreeSet<V>, ops : &mut MallocSizeOfOps) -> usize
where V : MallocSizeOf  {
    // use the same estimation as for the BTreeMap
    // (https://github.com/servo/heapsize/blob/32615cb931b4871b24f72114cca9faa8bca48399/src/lib.rs#L296)
    // but assume a value size of 1 byte
    let mut size = 0;
    for value in s.iter() {
        size += size_of::<V>() + 1 +
                value.size_of(ops);
    }
    size
}
