use {Match, StringID, Annotation};
use self::nodesearch::NodeSearch;
use stringstorage::StringStorage;

use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Desc {
    pub component_nr : usize,
    pub lhs: Option<Box<Desc>>,
    pub rhs: Option<Box<Desc>>,
    pub node_pos : BTreeMap<usize, usize>,
    pub impl_description: String,
    pub query_fragment : String,
}

impl Desc {

    pub fn empty_with_fragment(query_fragment : &str, node_nr : usize) -> Desc {
        let mut node_pos = BTreeMap::new();
        node_pos.insert(node_nr, 0);
        Desc {
            component_nr: 0,
            lhs: None,
            rhs: None,
            node_pos,
            impl_description: String::from(""),
            query_fragment: String::from(query_fragment),
        }
    }

    pub fn join(lhs : Option<&Desc>, rhs : Option<&Desc>, impl_description : &str, query_fragment : &str) -> Desc {
        let component_nr = if let Some(d) = lhs  {
            d.component_nr
        } else if let Some(d) = rhs {
            d.component_nr
        } else {
            0
        };

        // merge both node positions
        let mut node_pos = BTreeMap::new();
        let offset = if let Some(ref lhs) = lhs {
            node_pos = lhs.node_pos.clone();
            node_pos.len()
        } else {
            0
        };
        if let Some(ref rhs) = rhs {
            for e in rhs.node_pos.iter() {
                // the RHS has an offset after the join
                node_pos.insert(e.0.clone(), e.1 + offset);
            }
        }

        // TODO: add query fragment
        Desc {
            component_nr,
            lhs: lhs.map(|x| Box::new(x.clone())),
            rhs: rhs.map(|x| Box::new(x.clone())),
            node_pos,
            impl_description: String::from(impl_description),
            query_fragment: String::from(query_fragment),
        }
    }

    pub fn debug_string(&self, indention : &str) -> String {
        let mut result = String::from(indention);
       
        // output the node number and query fragment for base nodes
        if self.lhs.is_none() && self.rhs.is_none() {
            let node_nr = self.node_pos.keys().next().cloned().unwrap_or(0) + 1; 

            result.push_str(&format!("#{} ({})\n", &node_nr.to_string(), &self.query_fragment));
        } else {
            result.push_str(&format!("+|{} ({})\n", &self.impl_description , &self.query_fragment));
          
            let new_indention = format!("{}    ", indention);
            if let Some(ref lhs) = self.lhs {
                result.push_str(&lhs.debug_string(&new_indention));
            }
            if let Some(ref rhs) = self.rhs {
                 result.push_str(&rhs.debug_string(&new_indention));
            }

            return result;
        }
        // TODO: cost info

        return result;
    } 
}

pub struct NodeSearchDesc {
    pub qname: (Option<StringID>, Option<StringID>),
    pub cond: Box<Fn(Annotation, &StringStorage) -> bool>,
}

pub trait ExecutionNode : Iterator {
    fn as_iter(& mut self) -> &mut Iterator<Item = Vec<Match>>;
    fn as_nodesearch(&self) -> Option<&NodeSearch> {
        None
    }


    fn get_desc(&self) -> Option<&Desc> {
        None
    }
    

}

pub mod nestedloop;
pub mod indexjoin;
pub mod nodesearch;
pub mod binary_filter;