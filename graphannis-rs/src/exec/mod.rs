use {Match, StringID, Annotation};

use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Desc {
    pub component_nr : usize,
    pub lhs: Option<Box<Desc>>,
    pub rhs: Option<Box<Desc>>,
    pub node_pos : BTreeMap<usize, usize>,
    pub query_fragment : String,
}

impl Desc {

    pub fn empty_with_fragment(query_fragment : &str) -> Desc {
        Desc {
            component_nr: 0,
            lhs: None,
            rhs: None,
            node_pos : BTreeMap::new(),
            query_fragment: String::from(query_fragment),
        }
    }

    pub fn join(lhs : Option<&Desc>, rhs : Option<&Desc>) -> Desc {
        let component_nr = if let Some(d) = lhs  {
            d.component_nr
        } else if let Some(d) = rhs {
            d.component_nr
        } else {
            0
        };


        let mut lhs = if let Some(d) = lhs {
            Some(Box::new(d.clone()))
        } else {
            None
        };

        let mut rhs = if let Some(d) = rhs {
            Some(Box::new(d.clone()))
        } else {
            None
        };

        // merge both node positions
        let mut node_pos = BTreeMap::new();
        let offset = if let Some(ref mut lhs) = lhs {
            node_pos.append(&mut lhs.node_pos);
            node_pos.len()
        } else {
            0
        };
        if let Some(ref mut rhs) = rhs {
            for e in rhs.node_pos.iter() {
                // the RHS has an offset after the join
                node_pos.insert(e.0.clone(), e.1 + offset);
            }
            // move values to new map, the old should be empty (as with the append() function)
            rhs.node_pos.clear();
        }

        // TODO: add query fragment
        Desc {
            component_nr,
            lhs,
            rhs,
            node_pos,
            query_fragment: String::from(""),
        }
    }
}

pub struct NodeSearchDesc<'a> {
    pub qname: (Option<StringID>, Option<StringID>),
    pub cond: Box<Fn(Annotation) -> bool + 'a>,
}

pub trait ExecutionNode : Iterator {
    fn as_iter(& mut self) -> &mut Iterator<Item = Vec<Match>>;


    fn get_desc(&self) -> Option<&Desc> {
        None
    }
    fn get_node_search_desc(&self) -> Option<&NodeSearchDesc> {
        None
    }

}

pub mod nestedloop;
pub mod indexjoin;
pub mod nodesearch;
pub mod binary_filter;