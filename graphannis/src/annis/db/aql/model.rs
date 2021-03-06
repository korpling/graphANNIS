use crate::{
    errors::GraphAnnisError,
    graph::{Edge, EdgeContainer, GraphStorage, NodeID},
};
use graphannis_core::{
    dfs::CycleSafeDFS,
    errors::ComponentTypeError,
    graph::{storage::union::UnionEdgeContainer, ANNIS_NS},
    types::ComponentType,
    util::disk_collections::{DiskMap, EvictionStrategy},
};
use std::fmt;

use std::borrow::Cow;
use std::{str::FromStr, sync::Arc};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};

use crate::{update::UpdateEvent, AnnotationGraph};
use rustc_hash::FxHashSet;

use crate::{
    graph::{AnnoKey, Component},
    model::AnnotationComponent,
    Graph,
};

pub const TOK: &str = "tok";
pub const TOK_WHITESPACE_BEFORE: &str = "tok-whitespace-before";
pub const TOK_WHITESPACE_AFTER: &str = "tok-whitespace-after";

lazy_static! {
    pub static ref TOKEN_KEY: Arc<AnnoKey> = Arc::from(AnnoKey {
        ns: ANNIS_NS.into(),
        name: TOK.into(),
    });
}

/// Specifies the type of component of the annotation graph. The types of this enum carray certain semantics about the edges of the graph components their are used in.
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
pub enum AnnotationComponentType {
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

impl Into<u16> for AnnotationComponentType {
    fn into(self) -> u16 {
        self as u16
    }
}

impl From<u16> for AnnotationComponentType {
    fn from(idx: u16) -> AnnotationComponentType {
        match idx {
            0 => AnnotationComponentType::Coverage,
            2 => AnnotationComponentType::Dominance,
            3 => AnnotationComponentType::Pointing,
            4 => AnnotationComponentType::Ordering,
            5 => AnnotationComponentType::LeftToken,
            6 => AnnotationComponentType::RightToken,
            7 => AnnotationComponentType::PartOf,
            _ => AnnotationComponentType::Pointing,
        }
    }
}

pub struct AQLUpdateGraphIndex {
    node_ids: DiskMap<String, NodeID>,
    calculate_invalid_nodes: bool,
    invalid_nodes: DiskMap<NodeID, bool>,
    text_coverage_components: FxHashSet<AnnotationComponent>,
}

impl AQLUpdateGraphIndex {
    fn get_cached_node_id_from_name(
        &mut self,
        node_name: Cow<String>,
        graph: &AnnotationGraph,
    ) -> std::result::Result<NodeID, ComponentTypeError> {
        if let Some(id) = self.node_ids.try_get(&node_name)? {
            return Ok(id);
        } else if let Some(id) = graph.get_node_id_from_name(&node_name) {
            self.node_ids.insert(node_name.to_string(), id)?;
            return Ok(id);
        }
        Err(ComponentTypeError(
            GraphAnnisError::NoSuchNodeID(node_name.to_string()).into(),
        ))
    }

    fn calculate_invalidated_nodes_by_coverage(
        &mut self,
        graph: &AnnotationGraph,
        node: NodeID,
    ) -> std::result::Result<(), ComponentTypeError> {
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
        graph: &mut AnnotationGraph,
        gs_order: Arc<dyn GraphStorage>,
    ) -> std::result::Result<(), ComponentTypeError> {
        {
            // remove existing left/right token edges for the invalidated nodes
            let gs_left = graph.get_or_create_writable(&AnnotationComponent::new(
                AnnotationComponentType::LeftToken,
                ANNIS_NS.into(),
                "".into(),
            ))?;

            for (n, _) in self.invalid_nodes.iter() {
                gs_left.delete_node(n)?;
            }

            let gs_right = graph.get_or_create_writable(&AnnotationComponent::new(
                AnnotationComponentType::RightToken,
                ANNIS_NS.into(),
                "".into(),
            ))?;

            for (n, _) in self.invalid_nodes.iter() {
                gs_right.delete_node(n)?;
            }

            let gs_cov = graph.get_or_create_writable(&AnnotationComponent::new(
                AnnotationComponentType::Coverage,
                ANNIS_NS.into(),
                "inherited-coverage".into(),
            ))?;
            for (n, _) in self.invalid_nodes.iter() {
                gs_cov.delete_node(n)?;
            }
        }

        let all_cov_components =
            graph.get_all_components(Some(AnnotationComponentType::Coverage), None);
        let all_dom_gs: Vec<Arc<dyn GraphStorage>> = graph
            .get_all_components(Some(AnnotationComponentType::Dominance), Some(""))
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
                    AnnotationComponentType::LeftToken,
                    gs_order.as_ref(),
                    &all_cov_gs,
                    &all_dom_gs,
                )?;
                self.calculate_token_alignment(
                    graph,
                    n,
                    AnnotationComponentType::RightToken,
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
        graph: &mut AnnotationGraph,
        n: NodeID,
        all_cov_components: &[AnnotationComponent],
        all_dom_gs: &[Arc<dyn GraphStorage>],
    ) -> std::result::Result<FxHashSet<NodeID>, ComponentTypeError> {
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

        if let Ok(gs_cov) = graph.get_or_create_writable(&AnnotationComponent::new(
            AnnotationComponentType::Coverage,
            ANNIS_NS.into(),
            "inherited-coverage".into(),
        )) {
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
        graph: &mut AnnotationGraph,
        n: NodeID,
        ctype: AnnotationComponentType,
        gs_order: &dyn GraphStorage,
        all_cov_gs: &[Arc<dyn GraphStorage>],
        all_dom_gs: &[Arc<dyn GraphStorage>],
    ) -> std::result::Result<Option<NodeID>, ComponentTypeError> {
        let alignment_component =
            AnnotationComponent::new(ctype.clone(), ANNIS_NS.into(), "".into());

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
        let mut candidates: Vec<_> = candidates.into_iter().collect();
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
        let t = if ctype == AnnotationComponentType::RightToken {
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

impl ComponentType for AnnotationComponentType {
    type UpdateGraphIndex = AQLUpdateGraphIndex;

    fn all_component_types() -> Vec<Self> {
        AnnotationComponentType::iter().collect()
    }

    fn default_components() -> Vec<AnnotationComponent> {
        vec![
            AnnotationComponent::new(
                AnnotationComponentType::Coverage,
                ANNIS_NS.into(),
                "".into(),
            ),
            AnnotationComponent::new(
                AnnotationComponentType::Ordering,
                ANNIS_NS.into(),
                "".into(),
            ),
            AnnotationComponent::new(
                AnnotationComponentType::LeftToken,
                ANNIS_NS.into(),
                "".into(),
            ),
            AnnotationComponent::new(
                AnnotationComponentType::RightToken,
                ANNIS_NS.into(),
                "".into(),
            ),
            AnnotationComponent::new(AnnotationComponentType::PartOf, ANNIS_NS.into(), "".into()),
        ]
    }

    fn init_update_graph_index(
        graph: &AnnotationGraph,
    ) -> std::result::Result<Self::UpdateGraphIndex, ComponentTypeError> {
        // Cache the expensive mapping of node names to IDs
        let node_ids = DiskMap::new(None, EvictionStrategy::MaximumItems(1_000_000))?;

        // Calculating the invalid nodes adds additional computational overhead. If there are no nodes yet in the graph,
        // we already know that all new nodes are invalid and don't need calculate the invalid ones.
        let calculate_invalid_nodes = !graph.get_node_annos().is_empty();

        let invalid_nodes: DiskMap<NodeID, bool> =
            DiskMap::new(None, EvictionStrategy::MaximumItems(1_000_000))?;

        let mut text_coverage_components = FxHashSet::default();
        text_coverage_components
            .extend(graph.get_all_components(Some(AnnotationComponentType::Dominance), Some("")));
        text_coverage_components
            .extend(graph.get_all_components(Some(AnnotationComponentType::Coverage), None));
        Ok(AQLUpdateGraphIndex {
            node_ids,
            calculate_invalid_nodes,
            text_coverage_components,
            invalid_nodes,
        })
    }

    fn before_update_event(
        update: &UpdateEvent,
        graph: &AnnotationGraph,
        index: &mut Self::UpdateGraphIndex,
    ) -> std::result::Result<(), ComponentTypeError> {
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
                    if let Ok(ctype) = AnnotationComponentType::from_str(&component_type) {
                        if ctype == AnnotationComponentType::Coverage
                            || ctype == AnnotationComponentType::Dominance
                            || ctype == AnnotationComponentType::Ordering
                            || ctype == AnnotationComponentType::LeftToken
                            || ctype == AnnotationComponentType::RightToken
                        {
                            let source = index
                                .get_cached_node_id_from_name(Cow::Borrowed(source_node), graph)?;
                            index.calculate_invalidated_nodes_by_coverage(graph, source)?;
                        }

                        if ctype == AnnotationComponentType::Ordering {
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
        graph: &AnnotationGraph,
        index: &mut Self::UpdateGraphIndex,
    ) -> std::result::Result<(), ComponentTypeError> {
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
                if let Ok(ctype) = AnnotationComponentType::from_str(&component_type) {
                    if (ctype == AnnotationComponentType::Dominance
                        || ctype == AnnotationComponentType::Coverage)
                        && component_name.is_empty()
                    {
                        // might be a new text coverage component
                        let c = AnnotationComponent::new(
                            ctype.clone(),
                            layer.into(),
                            component_name.into(),
                        );
                        index.text_coverage_components.insert(c);
                    }

                    if index.calculate_invalid_nodes {
                        if ctype == AnnotationComponentType::Coverage
                            || ctype == AnnotationComponentType::Dominance
                            || ctype == AnnotationComponentType::Ordering
                            || ctype == AnnotationComponentType::LeftToken
                            || ctype == AnnotationComponentType::RightToken
                        {
                            let source = index
                                .get_cached_node_id_from_name(Cow::Owned(source_node), graph)?;

                            index.calculate_invalidated_nodes_by_coverage(graph, source)?;
                        }

                        if ctype == AnnotationComponentType::Ordering {
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
        graph: &mut AnnotationGraph,
    ) -> std::result::Result<(), ComponentTypeError> {
        index.invalid_nodes.compact()?;

        // Re-index the inherited coverage component.
        // To make this operation fast, we need to optimize the order component first
        let order_component = AnnotationComponent::new(
            AnnotationComponentType::Ordering,
            ANNIS_NS.into(),
            "".into(),
        );
        let order_stats_exist = graph
            .get_graphstorage(&order_component)
            .map(|gs_order| gs_order.get_statistics().is_some())
            .unwrap_or(false);
        if !order_stats_exist {
            graph.calculate_component_statistics(&order_component)?;
        }
        graph.optimize_gs_impl(&order_component)?;
        if let Some(gs_order) = graph.get_graphstorage(&order_component) {
            index.reindex_inherited_coverage(graph, gs_order)?;
        }

        Ok(())
    }

    fn update_graph_index_components(_graph: &Graph<Self>) -> Vec<Component<Self>> {
        vec![
            AnnotationComponent::new(
                AnnotationComponentType::LeftToken,
                ANNIS_NS.into(),
                "".into(),
            ),
            AnnotationComponent::new(
                AnnotationComponentType::RightToken,
                ANNIS_NS.into(),
                "".into(),
            ),
            AnnotationComponent::new(
                AnnotationComponentType::Coverage,
                ANNIS_NS.into(),
                "inherited-coverage".into(),
            ),
        ]
    }
}

impl fmt::Display for AnnotationComponentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
