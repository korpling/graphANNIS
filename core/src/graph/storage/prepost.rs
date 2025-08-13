use super::{
    EdgeContainer, GraphStatistic, GraphStorage, deserialize_gs_field,
    legacy::PrePostOrderStorageV1, load_statistics_from_location, save_statistics_to_toml,
    serialize_gs_field,
};
use crate::{
    annostorage::{
        AnnotationStorage, EdgeAnnotationStorage, NodeAnnotationStorage, inmemory::AnnoStorageImpl,
    },
    dfs::CycleSafeDFS,
    errors::Result,
    types::{Edge, NodeID, NumValue},
};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::{ops::Bound::*, path::Path};

#[derive(PartialOrd, PartialEq, Ord, Eq, Clone, Serialize, Deserialize)]
pub struct PrePost<OrderT, LevelT> {
    pub pre: OrderT,
    pub post: OrderT,
    pub level: LevelT,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum OrderVecEntry<OrderT, LevelT> {
    None,
    Pre {
        post: OrderT,
        level: LevelT,
        node: NodeID,
    },
    Post {
        pre: OrderT,
        level: LevelT,
        node: NodeID,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PrePostOrderStorage<OrderT: NumValue, LevelT: NumValue> {
    node_to_order: FxHashMap<NodeID, Vec<PrePost<OrderT, LevelT>>>,
    order_to_node: Vec<OrderVecEntry<OrderT, LevelT>>,
    annos: AnnoStorageImpl<Edge>,
    stats: Option<GraphStatistic>,
}

struct NodeStackEntry<OrderT, LevelT> {
    pub id: NodeID,
    pub order: PrePost<OrderT, LevelT>,
}

impl<OrderT, LevelT> Default for PrePostOrderStorage<OrderT, LevelT>
where
    OrderT: NumValue,
    LevelT: NumValue,
{
    fn default() -> Self {
        PrePostOrderStorage::new()
    }
}

impl<OrderT, LevelT> PrePostOrderStorage<OrderT, LevelT>
where
    OrderT: NumValue,
    LevelT: NumValue,
{
    pub fn new() -> PrePostOrderStorage<OrderT, LevelT> {
        PrePostOrderStorage {
            node_to_order: FxHashMap::default(),
            order_to_node: Vec::new(),
            annos: AnnoStorageImpl::new(),
            stats: None,
        }
    }

    pub fn clear(&mut self) -> Result<()> {
        self.node_to_order.clear();
        self.order_to_node.clear();
        self.annos.clear()?;
        self.stats = None;
        Ok(())
    }

    fn enter_node(
        current_order: &mut OrderT,
        node_id: NodeID,
        level: LevelT,
        node_stack: &mut NStack<OrderT, LevelT>,
    ) {
        let new_entry = NodeStackEntry {
            id: node_id,
            order: PrePost {
                pre: current_order.clone(),
                level,
                post: OrderT::zero(),
            },
        };
        current_order.add_assign(OrderT::one());
        node_stack.push_front(new_entry);
    }

    fn exit_node(&mut self, current_order: &mut OrderT, node_stack: &mut NStack<OrderT, LevelT>) {
        // find the correct pre/post entry and update the post-value
        if let Some(entry) = node_stack.front_mut() {
            entry.order.post = current_order.clone();
            current_order.add_assign(OrderT::one());

            self.node_to_order
                .entry(entry.id)
                .or_insert_with(|| Vec::with_capacity(1))
                .push(entry.order.clone());
        }
        node_stack.pop_front();
    }

    fn copy_edge_annos_for_node(
        &mut self,
        source_node: NodeID,
        orig: &dyn GraphStorage,
    ) -> Result<()> {
        // Iterate over the outgoing edges of this node to add the edge
        // annotations
        let out_edges = orig.get_outgoing_edges(source_node);
        for target in out_edges {
            let target = target?;
            let e = Edge {
                source: source_node,
                target,
            };
            let edge_annos = orig.get_anno_storage().get_annotations_for_item(&e)?;
            for a in edge_annos {
                self.annos.insert(e.clone(), a)?;
            }
        }
        Ok(())
    }
}

type NStack<OrderT, LevelT> = std::collections::LinkedList<NodeStackEntry<OrderT, LevelT>>;

impl<OrderT: 'static, LevelT: 'static> EdgeContainer for PrePostOrderStorage<OrderT, LevelT>
where
    for<'de> OrderT: NumValue + Deserialize<'de> + Serialize,
    for<'de> LevelT: NumValue + Deserialize<'de> + Serialize,
{
    fn get_outgoing_edges<'a>(
        &'a self,
        node: NodeID,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        self.find_connected(node, 1, Included(1))
    }

    fn get_ingoing_edges<'a>(
        &'a self,
        node: NodeID,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        self.find_connected_inverse(node, 1, Included(1))
    }

    fn source_nodes<'a>(&'a self) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        let it = self
            .node_to_order
            .iter()
            .filter_map(move |(n, _order)| {
                // check if this is actual a source node (and not only a target node)
                if self.get_outgoing_edges(*n).next().is_some() {
                    Some(*n)
                } else {
                    None
                }
            })
            .map(Ok);
        Box::new(it)
    }

    fn get_statistics(&self) -> Option<&GraphStatistic> {
        self.stats.as_ref()
    }
}

impl<OrderT: 'static, LevelT: 'static> GraphStorage for PrePostOrderStorage<OrderT, LevelT>
where
    for<'de> OrderT: NumValue + Deserialize<'de> + Serialize,
    for<'de> LevelT: NumValue + Deserialize<'de> + Serialize,
{
    fn get_anno_storage(&self) -> &dyn EdgeAnnotationStorage {
        &self.annos
    }

    fn serialization_id(&self) -> String {
        format!(
            "PrePostOrderO{}L{}V1",
            std::mem::size_of::<OrderT>() * 8,
            std::mem::size_of::<LevelT>() * 8
        )
    }

    fn load_from(location: &Path) -> Result<Self>
    where
        for<'de> Self: std::marker::Sized + Deserialize<'de>,
    {
        let legacy_path = location.join("component.bin");
        let mut result: Self = if legacy_path.is_file() {
            let component: PrePostOrderStorageV1<OrderT, LevelT> =
                deserialize_gs_field(location, "component")?;
            Self {
                node_to_order: component.node_to_order,
                order_to_node: component.order_to_node,
                annos: component.annos,
                stats: component.stats.map(GraphStatistic::from),
            }
        } else {
            let stats = load_statistics_from_location(location)?;
            Self {
                node_to_order: deserialize_gs_field(location, "node_to_order")?,
                order_to_node: deserialize_gs_field(location, "order_to_node")?,
                annos: deserialize_gs_field(location, "annos")?,
                stats,
            }
        };

        result.annos.after_deserialization();

        Ok(result)
    }

    fn save_to(&self, location: &Path) -> Result<()> {
        serialize_gs_field(&self.node_to_order, "node_to_order", location)?;
        serialize_gs_field(&self.order_to_node, "order_to_node", location)?;
        serialize_gs_field(&self.annos, "annos", location)?;
        save_statistics_to_toml(location, self.stats.as_ref())?;
        Ok(())
    }

    fn find_connected<'a>(
        &'a self,
        node: NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        if let Some(start_orders) = self.node_to_order.get(&node) {
            let mut visited = FxHashSet::<NodeID>::default();

            let max_distance = match max_distance {
                Unbounded => usize::MAX,
                Included(max_distance) => max_distance,
                Excluded(max_distance) => max_distance - 1,
            };

            let it = start_orders
                .iter()
                .flat_map(move |root_order: &PrePost<OrderT, LevelT>| {
                    let start = root_order.pre.to_usize().unwrap_or(0);
                    let end = root_order
                        .post
                        .to_usize()
                        .unwrap_or(self.order_to_node.len() - 1)
                        + 1;
                    self.order_to_node[start..end]
                        .iter()
                        .map(move |order| (root_order.clone(), order))
                })
                .filter_map(move |(root, order)| match order {
                    OrderVecEntry::Pre { post, level, node } => {
                        if let (Some(current_level), Some(root_level)) =
                            (level.to_usize(), root.level.to_usize())
                        {
                            let diff_level = current_level - root_level;
                            if *post <= root.post
                                && min_distance <= diff_level
                                && diff_level <= max_distance
                            {
                                Some(*node)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .filter(move |n| visited.insert(*n))
                .map(Ok);
            Box::new(it)
        } else {
            Box::new(std::iter::empty())
        }
    }

    fn find_connected_inverse<'a>(
        &'a self,
        start_node: NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> Box<dyn Iterator<Item = Result<NodeID>> + 'a> {
        if let Some(start_orders) = self.node_to_order.get(&start_node) {
            let mut visited = FxHashSet::<NodeID>::default();

            let max_distance = match max_distance {
                Unbounded => usize::MAX,
                Included(max_distance) => max_distance,
                Excluded(max_distance) => max_distance - 1,
            };

            let it = start_orders
                .iter()
                .flat_map(move |root_order: &PrePost<OrderT, LevelT>| {
                    let root_pre = root_order.pre.clone().to_usize().unwrap_or(0);
                    let root_post = root_order
                        .post
                        .clone()
                        .to_usize()
                        .unwrap_or(self.order_to_node.len() - 1);

                    // decide which search range (either 0..pre or post..len()) is smaller
                    let (start, end, use_post) = if self.order_to_node.len() - root_post < root_pre
                    {
                        // use post..len()
                        (root_post, self.order_to_node.len(), true)
                    } else {
                        // use 0..pre
                        (0, root_pre + 1, false)
                    };

                    self.order_to_node[start..end]
                        .iter()
                        .enumerate()
                        .map(move |(idx, order)| (use_post, root_order.clone(), start + idx, order))
                })
                .filter_map(move |(use_post, root, idx, order)| {
                    let (current_pre, current_post, current_level, current_node) = if use_post {
                        match order {
                            OrderVecEntry::Post { pre, level, node } => {
                                (pre.to_usize(), Some(idx), level.to_usize(), Some(node))
                            }
                            _ => (None, None, None, None),
                        }
                    } else {
                        match order {
                            OrderVecEntry::Pre { post, level, node } => {
                                (Some(idx), post.to_usize(), level.to_usize(), Some(node))
                            }
                            _ => (None, None, None, None),
                        }
                    };

                    if let (
                        Some(current_node),
                        Some(current_pre),
                        Some(current_post),
                        Some(current_level),
                        Some(root_level),
                        Some(root_pre),
                        Some(root_post),
                    ) = (
                        current_node,
                        current_pre,
                        current_post,
                        current_level,
                        root.level.to_usize(),
                        root.pre.to_usize(),
                        root.post.to_usize(),
                    ) {
                        if let Some(diff_level) = root_level.checked_sub(current_level) {
                            if current_pre <= root_pre
                                && current_post >= root_post
                                && min_distance <= diff_level
                                && diff_level <= max_distance
                            {
                                Some(*current_node)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .filter(move |n| visited.insert(*n))
                .map(Ok);
            Box::new(it)
        } else {
            Box::new(std::iter::empty())
        }
    }

    fn distance(&self, source: NodeID, target: NodeID) -> Result<Option<usize>> {
        if source == target {
            return Ok(Some(0));
        }

        let mut min_level = usize::MAX;
        let mut was_found = false;

        if let (Some(order_source), Some(order_target)) = (
            self.node_to_order.get(&source),
            self.node_to_order.get(&target),
        ) {
            for order_source in order_source.iter() {
                for order_target in order_target.iter() {
                    if order_source.pre <= order_target.pre
                        && order_target.post <= order_source.post
                    {
                        // check the level
                        if let (Some(source_level), Some(target_level)) =
                            (order_source.level.to_usize(), order_target.level.to_usize())
                            && source_level <= target_level
                        {
                            was_found = true;
                            min_level = std::cmp::min(target_level - source_level, min_level);
                        }
                    }
                }
            }
        }

        if was_found {
            Ok(Some(min_level))
        } else {
            Ok(None)
        }
    }
    fn is_connected(
        &self,
        source: NodeID,
        target: NodeID,
        min_distance: usize,
        max_distance: std::ops::Bound<usize>,
    ) -> Result<bool> {
        if let (Some(order_source), Some(order_target)) = (
            self.node_to_order.get(&source),
            self.node_to_order.get(&target),
        ) {
            let max_distance = match max_distance {
                Unbounded => usize::MAX,
                Included(max_distance) => max_distance,
                Excluded(max_distance) => max_distance - 1,
            };

            for order_source in order_source.iter() {
                for order_target in order_target.iter() {
                    if order_source.pre <= order_target.pre
                        && order_target.post <= order_source.post
                    {
                        // check the level
                        if let (Some(source_level), Some(target_level)) =
                            (order_source.level.to_usize(), order_target.level.to_usize())
                            && source_level <= target_level
                        {
                            let diff_level = target_level - source_level;
                            return Ok(min_distance <= diff_level && diff_level <= max_distance);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    fn copy(
        &mut self,
        _node_annos: &dyn NodeAnnotationStorage,
        orig: &dyn GraphStorage,
    ) -> Result<()> {
        self.clear()?;

        // find all roots of the component
        let roots: Result<FxHashSet<NodeID>> = orig.root_nodes().collect();
        let roots = roots?;

        let mut current_order = OrderT::zero();
        // traverse the graph for each sub-component
        for start_node in &roots {
            let mut last_distance: usize = 0;

            let mut node_stack: NStack<OrderT, LevelT> = NStack::new();

            self.copy_edge_annos_for_node(*start_node, orig)?;

            PrePostOrderStorage::enter_node(
                &mut current_order,
                *start_node,
                LevelT::zero(),
                &mut node_stack,
            );

            let dfs = CycleSafeDFS::new(orig.as_edgecontainer(), *start_node, 1, usize::MAX);
            for step in dfs {
                let step = step?;

                self.copy_edge_annos_for_node(step.node, orig)?;

                if step.distance <= last_distance {
                    // Neighbor node, the last subtree was iterated completely, thus the last node
                    // can be assigned a post-order.
                    // The parent node must be at the top of the node stack,
                    // thus exit every node which comes after the parent node.
                    // Distance starts with 0 but the stack size starts with 1.
                    while node_stack.len() > step.distance {
                        self.exit_node(&mut current_order, &mut node_stack);
                    }
                }
                // set pre-order
                if let Some(dist) = LevelT::from_usize(step.distance) {
                    PrePostOrderStorage::enter_node(
                        &mut current_order,
                        step.node,
                        dist,
                        &mut node_stack,
                    );
                }
                last_distance = step.distance;
            } // end for each DFS step

            while !node_stack.is_empty() {
                self.exit_node(&mut current_order, &mut node_stack);
            }
        } // end for each root

        // there must be an entry in the vector for all possible order values
        self.order_to_node
            .resize(current_order.to_usize().unwrap_or(0), OrderVecEntry::None);
        for (node, orders_for_node) in &self.node_to_order {
            for order in orders_for_node {
                if let Some(pre) = order.pre.to_usize() {
                    self.order_to_node[pre] = OrderVecEntry::Pre {
                        post: order.post.clone(),
                        level: order.level.clone(),
                        node: *node,
                    };
                }
                if let Some(post) = order.post.to_usize() {
                    self.order_to_node[post] = OrderVecEntry::Post {
                        pre: order.pre.clone(),
                        level: order.level.clone(),
                        node: *node,
                    };
                }
            }
        }

        self.stats = orig.get_statistics().cloned();
        self.annos.calculate_statistics()?;

        self.node_to_order.shrink_to_fit();

        Ok(())
    }

    fn as_edgecontainer(&self) -> &dyn EdgeContainer {
        self
    }
}

#[cfg(test)]
mod tests;
