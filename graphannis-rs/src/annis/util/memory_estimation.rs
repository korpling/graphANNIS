use std;
use std::collections::HashMap;
use std::collections::BTreeMap;

pub fn hash_map_size<K : std::cmp::Eq + std::hash::Hash,V>(map : &HashMap<K,V>) -> usize {
    let by_id_hashes = map.capacity() * std::mem::size_of::<usize>();
    let by_id_pairs = map.capacity() * std::mem::size_of::<(K, V)>();

    return by_id_hashes + by_id_pairs;
}


pub fn btree_map_size<K : std::cmp::Eq,V>(map : &BTreeMap<K,V>) -> usize {

    const BTREE_CAPACITY : usize = 11;

    let key_size = std::mem::size_of::<K>() * BTREE_CAPACITY;
    let val_size = std::mem::size_of::<V>() * BTREE_CAPACITY;
    let parent_idx_size = std::mem::size_of::<u16>();
    let len_size = std::mem::size_of::<u16>();

    let single_node_size = key_size + val_size + parent_idx_size + len_size;

    // this assumes a complete unbalanced tree which is not really realistic
    return map.len() * single_node_size;
}