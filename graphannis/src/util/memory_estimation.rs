use std::collections::BTreeSet;
use std::mem::{size_of};
use heapsize::HeapSizeOf;

pub fn heap_size_of_children<V>(s : &BTreeSet<V>) -> usize
where V : HeapSizeOf  {
    // use the same estimation as for the BTreeMap
    // (https://github.com/servo/heapsize/blob/32615cb931b4871b24f72114cca9faa8bca48399/src/lib.rs#L296)
    // but assume a value size of 1 byte
    let mut size = 0;
    for value in s.iter() {
        size += size_of::<V>() + 1 +
                value.heap_size_of_children();
    }
    size
}
