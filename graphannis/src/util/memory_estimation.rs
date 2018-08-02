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

