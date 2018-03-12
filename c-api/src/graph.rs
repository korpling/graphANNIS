use graphannis::api::graph::{Node,Edge};
use libc;
use std;
use std::ffi::CString;

#[no_mangle]
pub extern "C" fn annis_node_id(n : * const Node) -> libc::uint64_t {
    let n : &Node = cast_const!(n);
    return n.id as libc::uint64_t;
}

#[no_mangle]
pub extern "C" fn annis_node_outgoing_len(n : * const Node) -> libc::size_t {
    let n : &Node = cast_const!(n);
    return n.outgoing_edges.len();
}

#[no_mangle]
pub extern "C" fn annis_node_outgoing_get(n : * const Node, i : libc::size_t) -> * const Edge {
    let n : &Node = cast_const!(n);
    if i < n.outgoing_edges.len() {
        return &n.outgoing_edges[i] as *const Edge;
    }
    return std::ptr::null();
}

#[no_mangle]
pub extern "C" fn annis_edge_source(e : * const Edge) -> libc::uint64_t {
    let n : &Edge = cast_const!(e);
    return n.source_id as libc::uint64_t;
}

#[no_mangle]
pub extern "C" fn annis_edge_target(e : * const Edge) -> libc::uint64_t {
    let n : &Edge = cast_const!(e);
    return n.target_id as libc::uint64_t;
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

#[no_mangle]
pub extern "C" fn annis_node_label_value(n : * const Node, name : * const libc::c_char) -> * mut libc::c_char {
    let n : &Node = cast_const!(n);
    let name = cstr!(name);
    if let Some(v) = n.labels.get(&String::from(name)) {
        if let Ok(v) = CString::new(v.as_str()) {
            return v.into_raw();
        }
    }
    return std::ptr::null_mut();
}

#[no_mangle]
pub extern "C" fn annis_edge_label_names(n : * const Edge) -> * mut Vec<CString> {
    let n : &Edge = cast_const!(n);
    let mut result : Vec<CString> = vec![];
    for l in n.labels.keys() {
        let l : &str = l;
        if let Ok(l) = CString::new(l) {
            result.push(l);
        }
    };
    return Box::into_raw(Box::from(result));
}

#[no_mangle]
pub extern "C" fn annis_edge_label_value(n : * const Edge, name : * const libc::c_char) -> * mut libc::c_char {
    let n : &Edge = cast_const!(n);
    let name = cstr!(name);
    if let Some(v) = n.labels.get(&String::from(name)) {
        if let Ok(v) = CString::new(v.as_str()) {
            return v.into_raw();
        }
    }
    return std::ptr::null_mut();
}