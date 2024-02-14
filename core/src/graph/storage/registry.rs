use super::adjacencylist::AdjacencyListStorage;
use super::dense_adjacency::DenseAdjacencyListStorage;
use super::disk_adjacency::DiskAdjacencyListStorage;
use super::disk_path::DiskPathStorage;
use super::linear::LinearGraphStorage;
use super::{disk_adjacency, disk_path};
use super::{prepost::PrePostOrderStorage, GraphStatistic, GraphStorage};
use crate::{
    errors::{GraphAnnisCoreError, Result},
    graph::Graph,
    types::ComponentType,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::{path::Path, sync::Arc};

pub struct GSInfo {
    pub id: String,
    constructor: fn() -> Result<Arc<dyn GraphStorage>>,
    deserialize_func: fn(&Path) -> Result<Arc<dyn GraphStorage>>,
}

lazy_static! {
    static ref REGISTRY: HashMap<String, GSInfo> = {
        let mut m = HashMap::new();

        insert_info::<AdjacencyListStorage>(&mut m);
        m.insert(
            disk_adjacency::SERIALIZATION_ID.to_owned(),
            create_info_diskadjacency(),
        );

        m.insert(
            disk_path::SERIALIZATION_ID.to_owned(),
            create_info_diskpath(),
        );
        insert_info::<DenseAdjacencyListStorage>(&mut m);

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

pub fn create_writeable<CT: ComponentType>(
    graph: &Graph<CT>,
    orig: Option<&dyn GraphStorage>,
) -> Result<Arc<dyn GraphStorage>> {
    if graph.disk_based {
        let mut result = DiskAdjacencyListStorage::new()?;
        if let Some(orig) = orig {
            result.copy(graph.get_node_annos(), orig)?;
        }
        Ok(Arc::from(result))
    } else {
        let mut result = AdjacencyListStorage::new();
        if let Some(orig) = orig {
            result.copy(graph.get_node_annos(), orig)?;
        }
        Ok(Arc::from(result))
    }
}

pub fn get_optimal_impl_heuristic<CT: ComponentType>(
    db: &Graph<CT>,
    stats: &GraphStatistic,
) -> GSInfo {
    if stats.max_depth <= 1 {
        // if we don't have any deep graph structures an adjencency list is always fasted (and has no overhead)
        return get_adjacencylist_impl(db, stats);
    } else if db.disk_based
        && stats.max_depth <= disk_path::MAX_DEPTH
        && stats.max_fan_out == 1
        && !stats.cyclic
    {
        // If we need to use a disk-based implementation and have short paths
        // without any branching (e.g. PartOf is often structured that way), use
        // an optimized implementation that stores the single path for each
        // source node.
        return create_info_diskpath();
    } else if stats.rooted_tree {
        if stats.max_fan_out <= 1 {
            return get_linear_by_size(stats);
        } else {
            return get_prepostorder_by_size(stats);
        }
    // it might be still wise to use pre/post order if the graph is "almost" a tree, thus
    // does not have many exceptions
    } else if !stats.cyclic && stats.dfs_visit_ratio <= 1.03 {
        // there is no more than 3% overhead
        // TODO: how to determine the border?
        return get_prepostorder_by_size(stats);
    }

    // fallback
    get_adjacencylist_impl(db, stats)
}

fn get_adjacencylist_impl<CT: ComponentType>(db: &Graph<CT>, stats: &GraphStatistic) -> GSInfo {
    if db.disk_based {
        create_info_diskadjacency()
    } else {
        // check if a large percentage of nodes are part of the graph storage
        if let Ok(Some(largest_node_id)) = db.node_annos.get_largest_item() {
            if stats.max_fan_out <= 1 && (stats.nodes as f64 / largest_node_id as f64) >= 0.75 {
                return create_info::<DenseAdjacencyListStorage>();
            }
        }

        create_info::<AdjacencyListStorage>()
    }
}

fn get_prepostorder_by_size(stats: &GraphStatistic) -> GSInfo {
    if stats.rooted_tree {
        // There are exactly two order values per node and there can be only one order value per node
        // in a tree.
        if stats.nodes < (u16::MAX / 2) as usize {
            if stats.max_depth < u8::MAX as usize {
                return create_info::<PrePostOrderStorage<u16, u8>>();
            } else if stats.max_depth < u32::MAX as usize {
                return create_info::<PrePostOrderStorage<u16, u32>>();
            }
        } else if stats.nodes < (u32::MAX / 2) as usize {
            if stats.max_depth < u8::MAX as usize {
                return create_info::<PrePostOrderStorage<u32, u8>>();
            } else if stats.max_depth < u32::MAX as usize {
                return create_info::<PrePostOrderStorage<u32, u32>>();
            }
        } else if stats.max_depth < u8::MAX as usize {
            return create_info::<PrePostOrderStorage<u64, u8>>();
        } else if stats.max_depth < u32::MAX as usize {
            return create_info::<PrePostOrderStorage<u64, u32>>();
        }
    } else if stats.max_depth < u8::MAX as usize {
        return create_info::<PrePostOrderStorage<u64, u8>>();
    }
    create_info::<PrePostOrderStorage<u64, u64>>()
}

fn get_linear_by_size(stats: &GraphStatistic) -> GSInfo {
    if stats.max_depth < u8::MAX as usize {
        create_info::<LinearGraphStorage<u8>>()
    } else if stats.max_depth < u16::MAX as usize {
        create_info::<LinearGraphStorage<u16>>()
    } else if stats.max_depth < u32::MAX as usize {
        create_info::<LinearGraphStorage<u32>>()
    } else {
        create_info::<LinearGraphStorage<u64>>()
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
        constructor: || Ok(Arc::new(GS::default())),
        deserialize_func: |location| Ok(Arc::new(GS::load_from(location)?)),
    }
}

fn create_info_diskadjacency() -> GSInfo {
    GSInfo {
        id: disk_adjacency::SERIALIZATION_ID.to_owned(),
        constructor: || Ok(Arc::from(DiskAdjacencyListStorage::new()?)),
        deserialize_func: |path| {
            let result = DiskAdjacencyListStorage::load_from(path)?;
            Ok(Arc::from(result))
        },
    }
}

fn create_info_diskpath() -> GSInfo {
    GSInfo {
        id: disk_path::SERIALIZATION_ID.to_string(),
        constructor: || Ok(Arc::from(DiskPathStorage::new()?)),
        deserialize_func: |path| {
            let result = DiskPathStorage::load_from(path)?;
            Ok(Arc::from(result))
        },
    }
}

pub fn create_from_info(info: &GSInfo) -> Result<Arc<dyn GraphStorage>> {
    (info.constructor)()
}

pub fn deserialize(impl_name: &str, location: &Path) -> Result<Arc<dyn GraphStorage>> {
    let info = REGISTRY
        .get(impl_name)
        .ok_or_else(|| GraphAnnisCoreError::UnknownGraphStorageImpl(impl_name.to_string()))?;
    (info.deserialize_func)(location)
}
