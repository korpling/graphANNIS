use graphannis::api::graph::{Node,Edge};
use libc;
use std::ffi::CString;


#[no_mangle]
pub extern "C" fn annis_node_getid(n : * const Node) -> libc::uint64_t {
    let n : &Node = cast_const!(n);
    return n.id as libc::uint64_t;
}

#[no_mangle]
pub extern "C" fn annis_node_label_names(n : * const Node) -> * mut Vec<CString> {
    let n : &Node = cast_const!(n);
    let mut result : Vec<CString> = vec![];
    for l in n.labels.keys() {
        let l : &str = l;
        if let Ok(l) = CString::new(l) {
            result.push(l);
        }
    };
    return Box::into_raw(Box::from(result));
}