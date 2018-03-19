use libc;
use std;
use std::ffi::CString;
use data::IterPtr;
use graphannis::{NodeID, Match, Annotation, StringID};
use graphannis::graphdb::{GraphDB};


#[no_mangle]
pub extern "C" fn annis_graph_nodes_by_type(g : * const GraphDB, node_type : * const libc::c_char) -> * mut IterPtr<NodeID> {
    let db : &GraphDB = cast_const!(g);
    let node_type = cstr!(node_type);

    let type_key = db.get_node_type_key();
    if let Some(val_id) = db.strings.find_id(&node_type) {
        let it = db.node_annos.exact_anno_search(Some(type_key.ns), type_key.name, Some(val_id.clone()))
            .map(|m : Match| m.node);
        return Box::into_raw(Box::new(Box::new(it)));
    }
    return std::ptr::null_mut();
}

#[no_mangle]
pub extern "C" fn annis_graph_node_labels(g : * const GraphDB,  node : NodeID) -> * mut Vec<Annotation> {
    let db : &GraphDB = cast_const!(g);

    Box::into_raw(Box::new(db.node_annos.get_all(&node)))
}

#[no_mangle]
pub extern "C" fn annis_graph_str(g : * const GraphDB,  str_id : StringID) -> * mut libc::c_char {
    let db : &GraphDB = cast_const!(g);

    if let Some(v) = db.strings.str(str_id) {
        let result = CString::new(v.clone()).unwrap_or_default();
        return result.into_raw();
    }
    return std::ptr::null_mut();
}

