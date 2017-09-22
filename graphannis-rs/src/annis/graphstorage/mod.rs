use std;
use annis::{NodeID, Edge, Annotation, AnnoKey};

pub trait ReadableGraphStorage {

    fn find_connected(source : &NodeID, min_distance : u32, max_distance : u32) -> Box<Iterator<Item = NodeID>> ;
    fn distance(source : &NodeID, target : &NodeID) -> u32;
    fn is_connected(source: &NodeID, target: &NodeID, min_distance : u32, max_distance : u32);

    fn clear();
}

pub trait WriteableGraphStorage : ReadableGraphStorage {

    fn add_edge(edge : Edge);
    fn add_edge_annotation(edge : &Edge, anno : Annotation);
    
    fn delete_edge(edge : &Edge);
    fn delete_edge_annotation(edge : &Edge, anno_key : &AnnoKey);
    fn delete_node(node : &NodeID);
} 
