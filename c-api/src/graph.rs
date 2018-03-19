use libc;
use std;
use std::ffi::CString;
use data::IterPtr;
use graphannis::{NodeID, Match};
use graphannis::graphdb::{GraphDB};
use graphannis::util;


#[no_mangle]
pub extern "C" fn annis_graph_get_anno_nodes(g : * const GraphDB) -> * mut IterPtr<Match> {
    let db : &GraphDB = cast_const!(g);

    let type_key = db.get_node_type_key();
    if let Some(val_id) = db.strings.find_id("node") {
        let it = db.node_annos.exact_anno_search(Some(type_key.ns), type_key.name, Some(val_id.clone()));
        return Box::into_raw(Box::new(it));
    }
    return std::ptr::null_mut();
}

#[no_mangle]
pub extern "C" fn annis_graph_get_node_label_value(g : * const GraphDB,  node : NodeID, qname : * const libc::c_char) -> * mut libc::c_char {
    let db : &GraphDB = cast_const!(g);
    
    let anno_key = util::qname_to_anno_key(&cstr!(qname), db);
    if let Some(anno_key) = anno_key {
        let anno_val_id = db.node_annos.get(&(node as NodeID), &anno_key);
        if let Some(anno_val_id) = anno_val_id {
            if let Some(anno_val) = db.strings.str(anno_val_id.clone()) {
                let result = CString::new(anno_val.clone()).unwrap_or_default();
                return result.into_raw();
            }
        }
    }

    return std::ptr::null_mut();
}

