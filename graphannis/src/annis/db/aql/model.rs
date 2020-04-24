use crate::graph::{Component, Edge, EdgeContainer, GraphStorage, NodeID};
use graphannis_core::{
    dfs::CycleSafeDFS,
    graph::{storage::union::UnionEdgeContainer, ANNIS_NS},
    types::ComponentType,
    util::disk_collections::{DiskMap, EvictionStrategy},
};
use std::fmt;
use std::iter::FromIterator;

use std::borrow::Cow;
use std::{str::FromStr, sync::Arc};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};

use crate::{update::UpdateEvent, Graph};
use anyhow::Result;
use rustc_hash::FxHashSet;

use crate::graph::AnnoKey;

pub const TOK: &str = "tok";
lazy_static! {
    pub static ref TOKEN_KEY: Arc<AnnoKey> = Arc::from(AnnoKey {
        ns: ANNIS_NS.to_owned(),
        name: TOK.to_owned(),
    });
}

/// Specifies the type of component. Types determine certain semantics about the edges of this graph components.
#[derive(
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Hash,
    Clone,
    Debug,
    EnumIter,
    EnumString,
    MallocSizeOf,
)]
#[repr(C)]
pub enum AQLComponentType {
    /// Edges between a span node and its tokens. Implies text coverage.
    Coverage,
    /// Edges between a structural node and any other structural node, span or token. Implies text coverage.
    Dominance = 2,
    /// Edge between any node.
    Pointing,
    /// Edge between two tokens implying that the source node comes before the target node in the textflow.
    Ordering,
    /// Explicit edge between any non-token node and the left-most token it covers.
    LeftToken,
    /// Explicit edge between any non-token node and the right-most token it covers.
    RightToken,
    /// Implies that the source node belongs to the parent corpus/subcorpus/document/datasource node.
    PartOf,
}

impl Into<u16> for AQLComponentType {
    fn into(self) -> u16 {
        self as u16
    }
}

impl From<u16> for AQLComponentType {
    fn from(idx: u16) -> AQLComponentType {
        match idx {
            0 => AQLComponentType::Coverage,
            2 => AQLComponentType::Dominance,
            3 => AQLComponentType::Pointing,
            4 => AQLComponentType::Ordering,
            5 => AQLComponentType::LeftToken,
            6 => AQLComponentType::RightToken,
            7 => AQLComponentType::PartOf,
            _ => AQLComponentType::Pointing,
        }
    }
}

pub struct AQLUpdateGraphIndex {
    node_ids: DiskMap<String, NodeID>,
    calculate_invalid_nodes: bool,
    invalid_nodes: DiskMap<NodeID, bool>,
    text_coverage_components: FxHashSet<Component>,
}

impl AQLUpdateGraphIndex {
    fn get_cached_node_id_from_name(
        &mut self,
        node_name: Cow<String>,
        graph: &Graph,
    ) -> Result<NodeID> {
        if let Some(id) = self.node_ids.try_get(&node_name)? {
            return Ok(id);
        } else {
            if let Some(id) = graph.get_node_id_from_name(&node_name) {
                self.node_ids.insert(node_name.to_string(), id.clone())?;
                return Ok(id);
            }
        }
        Err(anyhow!(
            "Could not get internal node ID for node {} when processing graph update",
            node_name
        ))
    }

    fn calculate_invalidated_nodes_by_coverage(
        &mut self,
        graph: &Graph,
        node: NodeID,
    ) -> Result<()> {
        let containers: Vec<&dyn EdgeContainer> = self
            .text_coverage_components
            .iter()
            .filter_map(|c| graph.get_graphstorage_as_ref(c))
            .map(|gs| gs.as_edgecontainer())
            .collect();

        let union = UnionEdgeContainer::new(containers);

        let dfs = CycleSafeDFS::new_inverse(&union, node, 0, usize::max_value());
        for step in dfs {
            self.invalid_nodes.insert(step.node, true)?;
        }

        Ok(())
    }

    fn reindex_inherited_coverage(
        &self,
        graph: &mut Graph,
        gs_order: Arc<dyn GraphStorage>,
    ) -> Result<()> {
        {
            // remove existing left/right token edges for the invalidated nodes
            let gs_left = graph.get_or_create_writable(&Component {
                ctype: AQLComponentType::LeftToken.into(),
                name: "".to_owned(),
                layer: ANNIS_NS.to_owned(),
            })?;

            for (n, _) in self.invalid_nodes.iter() {
                gs_left.delete_node(n)?;
            }

            let gs_right = graph.get_or_create_writable(&Component {
                ctype: AQLComponentType::RightToken.into(),
                name: "".to_owned(),
                layer: ANNIS_NS.to_owned(),
            })?;

            for (n, _) in self.invalid_nodes.iter() {
                gs_right.delete_node(n)?;
            }

            let gs_cov = graph.get_or_create_writable(&Component {
                ctype: AQLComponentType::Coverage.into(),
                name: "inherited-coverage".to_owned(),
                layer: ANNIS_NS.to_owned(),
            })?;
            for (n, _) in self.invalid_nodes.iter() {
                gs_cov.delete_node(n)?;
            }
        }

        let all_cov_components = graph.get_all_components(Some(AQLComponentType::Coverage), None);
        let all_dom_gs: Vec<Arc<dyn GraphStorage>> = graph
            .get_all_components(Some(AQLComponentType::Dominance), Some(""))
            .into_iter()
            .filter_map(|c| graph.get_graphstorage(&c))
            .collect();
        {
            // go over each node and calculate the left-most and right-most token

            let all_cov_gs: Vec<Arc<dyn GraphStorage>> = all_cov_components
                .iter()
                .filter_map(|c| graph.get_graphstorage(c))
                .collect();

            for (n, _) in self.invalid_nodes.iter() {
                self.calculate_token_alignment(
                    graph,
                    n,
                    AQLComponentType::LeftToken,
                    gs_order.as_ref(),
                    &all_cov_gs,
                    &all_dom_gs,
                )?;
                self.calculate_token_alignment(
                    graph,
                    n,
                    AQLComponentType::RightToken,
                    gs_order.as_ref(),
                    &all_cov_gs,
                    &all_dom_gs,
                )?;
            }
        }

        for (n, _) in self.invalid_nodes.iter() {
            self.calculate_inherited_coverage_edges(graph, n, &all_cov_components, &all_dom_gs)?;
        }

        Ok(())
    }

    fn calculate_inherited_coverage_edges(
        &self,
        graph: &mut Graph,
        n: NodeID,
        all_cov_components: &[Component],
        all_dom_gs: &[Arc<dyn GraphStorage>],
    ) -> Result<FxHashSet<NodeID>> {
        let mut covered_token = FxHashSet::default();
        for c in all_cov_components.iter() {
            if let Some(gs) = graph.get_graphstorage_as_ref(c) {
                covered_token.extend(gs.find_connected(n, 1, std::ops::Bound::Included(1)));
            }
        }

        if covered_token.is_empty() {
            if graph
                .get_node_annos()
                .get_value_for_item(&n, &TOKEN_KEY)
                .is_some()
            {
                covered_token.insert(n);
            } else {
                // recursivly get the covered token from all children connected by a dominance relation
                for dom_gs in all_dom_gs {
                    for out in dom_gs.get_outgoing_edges(n) {
                        covered_token.extend(self.calculate_inherited_coverage_edges(
                            graph,
                            out,
                            all_cov_components,
                            all_dom_gs,
                        )?);
                    }
                }
            }
        }

        if let Ok(gs_cov) = graph.get_or_create_writable(&Component {
            ctype: AQLComponentType::Coverage.into(),
            name: "inherited-coverage".to_owned(),
            layer: ANNIS_NS.to_owned(),
        }) {
            for t in covered_token.iter() {
                gs_cov.add_edge(Edge {
                    source: n,
                    target: *t,
                })?;
            }
        }

        Ok(covered_token)
    }

    fn calculate_token_alignment(
        &self,
        graph: &mut Graph,
        n: NodeID,
        ctype: AQLComponentType,
        gs_order: &dyn GraphStorage,
        all_cov_gs: &[Arc<dyn GraphStorage>],
        all_dom_gs: &[Arc<dyn GraphStorage>],
    ) -> Result<Option<NodeID>> {
        let alignment_component = Component {
            ctype: ctype.clone().into(),
            name: "".to_owned(),
            layer: ANNIS_NS.to_owned(),
        };

        // if this is a token, return the token itself
        if graph
            .get_node_annos()
            .get_value_for_item(&n, &TOKEN_KEY)
            .is_some()
        {
            // also check if this is an actually token and not only a segmentation
            let mut is_token = true;
            for gs_coverage in all_cov_gs.iter() {
                if gs_coverage.has_outgoing_edges(n) {
                    is_token = false;
                    break;
                }
            }
            if is_token {
                return Ok(Some(n));
            }
        }

        // if the node already has a left/right token, just return this value
        if let Some(alignment_gs) = graph.get_graphstorage_as_ref(&alignment_component) {
            let existing = alignment_gs.get_outgoing_edges(n).next();
            if let Some(existing) = existing {
                return Ok(Some(existing));
            }
        }

        // recursively get all candidate token by iterating over text-coverage edges
        let mut candidates = FxHashSet::default();

        for gs_for_component in all_dom_gs.iter().chain(all_cov_gs.iter()) {
            for target in gs_for_component.get_outgoing_edges(n) {
                if let Some(candidate_for_target) = self.calculate_token_alignment(
                    graph,
                    target,
                    ctype.clone(),
                    gs_order,
                    all_cov_gs,
                    all_dom_gs,
                )? {
                    candidates.insert(candidate_for_target);
                } else {
                    return Ok(None);
                }
            }
        }

        // order the candidate token by their position in the order chain
        let mut candidates = Vec::from_iter(candidates.into_iter());
        candidates.sort_unstable_by(move |a, b| {
            if a == b {
                return std::cmp::Ordering::Equal;
            }
            if gs_order.is_connected(*a, *b, 1, std::ops::Bound::Unbounded) {
                return std::cmp::Ordering::Less;
            } else if gs_order.is_connected(*b, *a, 1, std::ops::Bound::Unbounded) {
                return std::cmp::Ordering::Greater;
            }
            std::cmp::Ordering::Equal
        });

        // add edge to left/right most candidate token
        let t = if ctype == AQLComponentType::RightToken {
            candidates.last()
        } else {
            candidates.first()
        };
        if let Some(t) = t {
            let gs = graph.get_or_create_writable(&alignment_component)?;
            let e = Edge {
                source: n,
                target: *t,
            };
            gs.add_edge(e)?;

            Ok(Some(*t))
        } else {
            Ok(None)
        }
    }
}

impl ComponentType for AQLComponentType {
    type UpdateGraphIndex = AQLUpdateGraphIndex;

    fn all_component_types() -> Vec<Self> {
        AQLComponentType::iter().collect()
    }

    fn default_components() -> Vec<Component> {
        vec![
            Component {
                ctype: AQLComponentType::Coverage.into(),
                layer: ANNIS_NS.to_owned(),
                name: "".to_owned(),
            },
            Component {
                ctype: AQLComponentType::Ordering.into(),
                layer: ANNIS_NS.to_owned(),
                name: "".to_owned(),
            },
            Component {
                ctype: AQLComponentType::LeftToken.into(),
                layer: ANNIS_NS.to_owned(),
                name: "".to_owned(),
            },
            Component {
                ctype: AQLComponentType::RightToken.into(),
                layer: ANNIS_NS.to_owned(),
                name: "".to_owned(),
            },
            Component {
                ctype: AQLComponentType::PartOf.into(),
                layer: ANNIS_NS.to_owned(),
                name: "".to_owned(),
            },
        ]
    }

    fn init_graph_update_index(graph: &Graph) -> Result<Self::UpdateGraphIndex> {
        // Cache the expensive mapping of node names to IDs
        let node_ids = DiskMap::new(None, EvictionStrategy::MaximumItems(1_000_000))?;

        // Calculating the invalid nodes adds additional computational overhead. If there are no nodes yet in the graph,
        // we already know that all new nodes are invalid and don't need calculate the invalid ones.
        let calculate_invalid_nodes = !graph.get_node_annos().is_empty();

        let invalid_nodes: DiskMap<NodeID, bool> =
            DiskMap::new(None, EvictionStrategy::MaximumItems(1_000_000))?;

        let mut text_coverage_components = FxHashSet::default();
        text_coverage_components
            .extend(graph.get_all_components(Some(AQLComponentType::Dominance), Some("")));
        text_coverage_components
            .extend(graph.get_all_components(Some(AQLComponentType::Coverage), None));
        Ok(AQLUpdateGraphIndex {
            node_ids,
            calculate_invalid_nodes,
            text_coverage_components,
            invalid_nodes,
        })
    }

    fn before_update_event(
        update: &UpdateEvent,
        graph: &Graph,
        index: &mut Self::UpdateGraphIndex,
    ) -> Result<()> {
        match update {
            UpdateEvent::DeleteNode { node_name } => {
                let existing_node_id =
                    index.get_cached_node_id_from_name(Cow::Borrowed(node_name), graph)?;
                if !index.calculate_invalid_nodes
                    && index.invalid_nodes.get(&existing_node_id).is_none()
                {
                    index.calculate_invalidated_nodes_by_coverage(graph, existing_node_id)?;
                }
            }
            UpdateEvent::DeleteEdge {
                source_node,
                target_node,
                component_type,
                ..
            } => {
                if index.calculate_invalid_nodes {
                    if let Ok(ctype) = AQLComponentType::from_str(&component_type) {
                        if ctype == AQLComponentType::Coverage
                            || ctype == AQLComponentType::Dominance
                            || ctype == AQLComponentType::Ordering
                            || ctype == AQLComponentType::LeftToken
                            || ctype == AQLComponentType::RightToken
                        {
                            let source = index
                                .get_cached_node_id_from_name(Cow::Borrowed(source_node), graph)?;
                            index.calculate_invalidated_nodes_by_coverage(graph, source)?;
                        }

                        if ctype == AQLComponentType::Ordering {
                            let target = index
                                .get_cached_node_id_from_name(Cow::Borrowed(target_node), graph)?;
                            index.calculate_invalidated_nodes_by_coverage(graph, target)?;
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn after_update_event(
        update: UpdateEvent,
        graph: &Graph,
        index: &mut Self::UpdateGraphIndex,
    ) -> Result<()> {
        match update {
            UpdateEvent::AddNode { node_name, .. } => {
                if !index.calculate_invalid_nodes {
                    // All new added nodes need to be marked as invalid
                    let node_id =
                        index.get_cached_node_id_from_name(Cow::Owned(node_name), graph)?;
                    index.invalid_nodes.insert(node_id, true)?;
                }
            }
            UpdateEvent::AddEdge {
                component_type,
                component_name,
                layer,
                source_node,
                target_node,
                ..
            } => {
                if let Ok(ctype) = AQLComponentType::from_str(&component_type) {
                    if (ctype == AQLComponentType::Dominance || ctype == AQLComponentType::Coverage)
                        && component_name.is_empty()
                    {
                        // might be a new text coverage component
                        let c = Component {
                            ctype: ctype.clone().into(),
                            layer: layer,
                            name: component_name,
                        };
                        index.text_coverage_components.insert(c.clone());
                    }

                    if index.calculate_invalid_nodes {
                        if ctype == AQLComponentType::Coverage
                            || ctype == AQLComponentType::Dominance
                            || ctype == AQLComponentType::Ordering
                            || ctype == AQLComponentType::LeftToken
                            || ctype == AQLComponentType::RightToken
                        {
                            let source = index
                                .get_cached_node_id_from_name(Cow::Owned(source_node), graph)?;

                            index.calculate_invalidated_nodes_by_coverage(graph, source)?;
                        }

                        if ctype == AQLComponentType::Ordering {
                            let target = index
                                .get_cached_node_id_from_name(Cow::Owned(target_node), graph)?;
                            index.calculate_invalidated_nodes_by_coverage(graph, target)?;
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn apply_update_graph_index(
        mut index: Self::UpdateGraphIndex,
        graph: &mut Graph,
    ) -> Result<()> {
        index.invalid_nodes.compact()?;

        // Re-index the inherited coverage component.
        // To make this operation fast, we need to optimize the order component first
        let order_component = Component {
            ctype: AQLComponentType::Ordering.into(),
            layer: ANNIS_NS.to_owned(),
            name: "".to_owned(),
        };
        let order_stats_exist = graph
            .get_graphstorage(&order_component)
            .map(|gs_order| gs_order.get_statistics().is_some())
            .unwrap_or(false);
        if !order_stats_exist {
            graph.calculate_component_statistics(&order_component)?;
        }
        graph.optimize_impl(&order_component)?;
        if let Some(gs_order) = graph.get_graphstorage(&order_component) {
            index.reindex_inherited_coverage(graph, gs_order)?;
        }

        Ok(())
    }
}

impl fmt::Display for AQLComponentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
