use super::adjacencylist::AdjacencyListStorage;
use super::linear::LinearGraphStorage;
use super::prepost::PrePostOrderStorage;
use errors::*;
use graphstorage::{GraphStatistic, GraphStorage};
use serde::Deserialize;
use std;
use std::collections::HashMap;
use std::sync::Arc;

pub struct GSInfo {
    pub id: String,
    constructor: fn() -> Arc<GraphStorage>,
    deserialize_func: fn(&mut std::io::Read) -> Result<Arc<GraphStorage>>,
}

lazy_static! {
    static ref REGISTRY: HashMap<String, GSInfo> = {
        let mut m = HashMap::new();

        insert_info::<AdjacencyListStorage>(&mut m);

        insert_info::<PrePostOrderStorage<u64, u64>>(&mut m);
        insert_info::<PrePostOrderStorage<u64, u32>>(&mut m);
        insert_info::<PrePostOrderStorage<u64, u8>>(&mut m);
        insert_info::<PrePostOrderStorage<u32, u32>>(&mut m);
        insert_info::<PrePostOrderStorage<u32, u8>>(&mut m);
        insert_info::<PrePostOrderStorage<u16, u32>>(&mut m);
        insert_info::<PrePostOrderStorage<u16, u8>>(&mut m);

        insert_info::<LinearGraphStorage<u64>>(&mut m);
        insert_info::<LinearGraphStorage<u32>>(&mut m);
        insert_info::<LinearGraphStorage<u16>>(&mut m);
        insert_info::<LinearGraphStorage<u8>>(&mut m);
        m
    };
}

pub fn create_writeable() -> AdjacencyListStorage {
    // TODO: make this configurable when there are more writeable graph storage implementations
    AdjacencyListStorage::new()
}

pub fn get_optimal_impl_heuristic(stats: &GraphStatistic) -> GSInfo {
    if stats.max_depth <= 1 {
        // if we don't have any deep graph structures an adjencency list is always fasted (and has no overhead)
        return create_info::<AdjacencyListStorage>();
    } else if stats.rooted_tree {
        if stats.max_fan_out <= 1 {
            return get_linear_by_size(stats);
        } else {
            return get_prepostorder_by_size(stats);
        }
    } else if !stats.cyclic {
        // it might be still wise to use pre/post order if the graph is "almost" a tree, thus
        // does not have many exceptions
        if stats.dfs_visit_ratio <= 1.03 {
            // there is no more than 3% overhead
            // TODO: how to determine the border?
            return get_prepostorder_by_size(stats);
        }
    }

    // fallback
    return create_info::<AdjacencyListStorage>();;
}

fn get_prepostorder_by_size(stats: &GraphStatistic) -> GSInfo {
    if stats.rooted_tree {
        // There are exactly two order values per node and there can be only one order value per node
        // in a tree.
        if stats.nodes < (u16::max_value() / 2) as usize {
            if stats.max_depth < u8::max_value() as usize {
                return create_info::<PrePostOrderStorage<u16, u8>>();
            } else if stats.max_depth < u32::max_value() as usize {
                return create_info::<PrePostOrderStorage<u16, u32>>();
            }
        } else if stats.nodes < (u32::max_value() / 2) as usize {
            if stats.max_depth < u8::max_value() as usize {
                return create_info::<PrePostOrderStorage<u32, u8>>();
            } else if stats.max_depth < u32::max_value() as usize {
                return create_info::<PrePostOrderStorage<u32, u32>>();
            }
        } else {
            if stats.max_depth < u8::max_value() as usize {
                return create_info::<PrePostOrderStorage<u64, u8>>();
            } else if stats.max_depth < u32::max_value() as usize {
                return create_info::<PrePostOrderStorage<u64, u32>>();
            }
        }
    } else {
        if stats.max_depth < u8::max_value() as usize {
            return create_info::<PrePostOrderStorage<u64, u8>>();
        }
    }
    return create_info::<PrePostOrderStorage<u64, u64>>();;
}

fn get_linear_by_size(stats: &GraphStatistic) -> GSInfo {
    if stats.max_depth < u8::max_value() as usize {
        return create_info::<LinearGraphStorage<u8>>();
    } else if stats.max_depth < u16::max_value() as usize {
        return create_info::<LinearGraphStorage<u16>>();
    } else if stats.max_depth < u32::max_value() as usize {
        return create_info::<LinearGraphStorage<u32>>();
    } else {
        return create_info::<LinearGraphStorage<u64>>();
    }
}

fn insert_info<GS: 'static>(registry: &mut HashMap<String, GSInfo>)
where
    for<'de> GS: GraphStorage + Default + Deserialize<'de>,
{
    let info = create_info::<GS>();
    registry.insert(info.id.clone(), info);
}

fn create_info<GS: 'static>() -> GSInfo
where
    for<'de> GS: GraphStorage + Default + Deserialize<'de>,
{
    // create an instance to get the name
    let instance = GS::default();

    GSInfo {
        id: instance.serialization_id(),
        constructor: || Arc::new(GS::default()),
        deserialize_func: |input| Ok(Arc::new(GS::deserialize_gs(input)?)),
    }
}

pub fn create_from_info(info: &GSInfo) -> Arc<GraphStorage> {
    (info.constructor)()
}

pub fn deserialize(impl_name: &str, input: &mut std::io::Read) -> Result<Arc<GraphStorage>> {
    let info = REGISTRY.get(impl_name).ok_or(format!(
        "Could not find implementation for graph storage with name '{}'",
        impl_name
    ))?;
    return (info.deserialize_func)(input);
}

pub fn serialize(data: Arc<GraphStorage>, writer: &mut std::io::Write) -> Result<String> {
    data.serialize_gs(writer)?;
    Ok(data.serialization_id())
}
