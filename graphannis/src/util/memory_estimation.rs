#[cfg(not(windows))]
pub mod platform {
    extern crate jemalloc_sys;
    extern crate libc;
    use std::os::raw::c_void;

   

    // The C prototype is `je_malloc_usable_size(JEMALLOC_USABLE_SIZE_CONST void *ptr)`. On some
    // platforms `JEMALLOC_USABLE_SIZE_CONST` is `const` and on some it is empty. But in practice
    // this function doesn't modify the contents of the block that `ptr` points to, so we use
    // `*const c_void` here.
    extern "C" {
        #[cfg_attr(
            any(prefixed_jemalloc, target_os = "macos", target_os = "ios", target_os = "android"),
            link_name = "je_malloc_usable_size"
        )]
        fn malloc_usable_size(ptr: *const c_void) -> usize;
    }

     /// Get the size of a heap block.
    pub unsafe extern "C" fn usable_size(ptr: *const c_void) -> usize {
        if ptr.is_null() {
            return 0;
        } else {
            return malloc_usable_size(ptr);
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
