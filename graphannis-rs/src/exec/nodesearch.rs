use super::{Desc, ExecutionNode, NodeSearchDesc};
use graphdb::GraphDB;
use {Match, StringID, Annotation};

use std::rc::Rc;

/// An [ExecutionNode](#impl-ExecutionNode) which wraps base node (annotation) searches.
pub struct NodeSearch<'a> {
    /// The actual search implementation
    it: Box<Iterator<Item = Vec<Match>> + 'a>,

    desc: Option<Desc>,
    node_search_desc: Rc<NodeSearchDesc>,
}

impl<'a> NodeSearch<'a> {

    pub fn exact_value(ns : Option<&str>, name : &str, val : Option<&str>, db: &'a GraphDB) -> Option<NodeSearch<'a>> {

        let name_id : StringID = db.strings.find_id(name)?.clone();
        // not finding the strings will result in an None result, not in an less specific search
        let ns_id : Option<StringID> = if let Some(ns) = ns {Some(db.strings.find_id(ns)?).cloned()} else {None};
        let val_id : Option<StringID> = if let Some(val) = val {Some(db.strings.find_id(val)?).cloned()} else {None};

        let it = db.node_annos.exact_anno_search(ns_id, name_id, val_id).map(|n| vec![n]);
        
        let query_fragment = if ns.is_some() && val.is_some() {
            format!("{}:{}=\"{}\"", ns.unwrap(), name, val.unwrap())
        } else if ns.is_some() {
            format!("{}:{}", ns.unwrap(), name)
        } else if val.is_some() {
            format!("{}=\"{}\"", name, val.unwrap())
        } else {
            String::from(name)
        };

        let filter_func : Box<Fn(Annotation) -> bool> = if ns_id.is_some() && val_id.is_some() {
            Box::new(move |anno : Annotation|  {return anno.key.ns == ns_id.unwrap() && anno.key.name == name_id && anno.val == val_id.unwrap() })
        } else if ns.is_some() {
            Box::new(move |anno : Annotation|  {return anno.key.ns == ns_id.unwrap() && anno.key.name == name_id})
        } else if val.is_some() {
            Box::new(move |anno : Annotation|  {return anno.key.name == name_id && anno.val == val_id.unwrap() })
        } else {
            Box::new(move |anno : Annotation|  {return anno.key.name == name_id })
        };
 
        Some(NodeSearch {
            it: Box::from(it),
            desc: Some(Desc::empty_with_fragment(&query_fragment)),
            node_search_desc: Rc::new(NodeSearchDesc {
                qname : (ns_id, Some(name_id)),
                cond: filter_func,
            })
        })
    }

    pub fn set_desc(&mut self, desc : Option<Desc>) {
        self.desc = desc;
    }


    pub fn get_node_search_desc(&self) -> Rc<NodeSearchDesc> {
        self.node_search_desc.clone()
    }
}

impl<'a> ExecutionNode for NodeSearch<'a> {
    fn as_iter(&mut self) -> &mut Iterator<Item = Vec<Match>> {
        self
    }

    fn get_desc(&self) -> Option<&Desc> {
        self.desc.as_ref()
    }

    fn as_nodesearch(&self) -> Option<&NodeSearch> {
        Some(self)
    }

}

impl<'a> Iterator for NodeSearch<'a> {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        self.it.next()
    }
}
