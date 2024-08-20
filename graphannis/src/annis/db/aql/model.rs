use crate::{
    annis::{db::token_helper::TokenHelper, util::quicksort},
    errors::GraphAnnisError,
    graph::{Edge, EdgeContainer, GraphStorage, NodeID},
};
use graphannis_core::{
    annostorage::ValueSearch,
    dfs::CycleSafeDFS,
    errors::ComponentTypeError,
    graph::{storage::union::UnionEdgeContainer, ANNIS_NS, NODE_TYPE_KEY},
    types::ComponentType,
    util::disk_collections::{DiskMap, EvictionStrategy, DEFAULT_BLOCK_CACHE_CAPACITY},
};
use std::{
    collections::{BTreeMap, HashMap},
    fmt,
};
use transient_btree_index::BtreeConfig;

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
    Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Debug, EnumIter, EnumString,
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

impl From<AnnotationComponentType> for u16 {
    fn from(t: AnnotationComponentType) -> Self {
        t as u16
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

/// Collects information about the corpus size, e.g. in token.
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum CorpusSize {
    /// The corpus size is unknown.
    #[default]
    Unknown,
    /// The corpus size can be measured in base token ("annis:tok" without
    /// outgoing coverage edges) and segmentation layers.
    Token {
        base_token_count: u64,
        segmentation_count: BTreeMap<String, u64>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AQLGlobalStatistics {
    #[serde(default)]
    pub all_token_in_order_component: bool,
    #[serde(default)]
    pub corpus_size: CorpusSize,
}

fn calculate_inherited_coverage_edges(
    graph: &mut AnnotationGraph,
    n: NodeID,
    other_cov_gs: &[Arc<dyn GraphStorage>],
    all_text_coverage_components: &[AnnotationComponent],
    inherited_cov_component: &AnnotationComponent,
) -> std::result::Result<FxHashSet<NodeID>, ComponentTypeError> {
    // Iterate over all  all nodes that are somehow covered (by coverage or
    // dominance edges) starting from the given node.
    let all_text_cov_components_gs: Vec<_> = all_text_coverage_components
        .iter()
        .filter_map(|c| graph.get_graphstorage_as_ref(c))
        .map(|gs| gs.as_edgecontainer())
        .collect();

    let all_text_cov_components_combined = UnionEdgeContainer::new(all_text_cov_components_gs);

    let mut covered_token = FxHashSet::default();
    {
        let tok_helper = TokenHelper::new(graph)?;
        for step in CycleSafeDFS::new(&all_text_cov_components_combined, n, 1, usize::MAX) {
            let step = step?;
            if tok_helper.is_token(step.node)? {
                covered_token.insert(step.node);
            }
        }
    };

    // Connect all non-token nodes to the covered token nodes if no such direct coverage already exists
    let mut direct_coverage_targets = FxHashSet::default();
    for gs in other_cov_gs.iter() {
        for target in gs.get_outgoing_edges(n) {
            direct_coverage_targets.insert(target?);
        }
    }
    let inherited_gs_cov = graph.get_or_create_writable(inherited_cov_component)?;

    for target in &covered_token {
        if n != *target && !direct_coverage_targets.contains(target) {
            inherited_gs_cov.add_edge(Edge {
                source: n,
                target: *target,
            })?;
        }
    }

    Ok(covered_token)
}

pub struct AQLUpdateGraphIndex {
    node_ids: DiskMap<String, NodeID>,
    graph_without_nodes: bool,
    invalid_nodes: DiskMap<NodeID, bool>,
    text_coverage_components: FxHashSet<AnnotationComponent>,
}

impl AQLUpdateGraphIndex {
    fn get_cached_node_id_from_name(
        &mut self,
        node_name: Cow<String>,
        graph: &AnnotationGraph,
    ) -> std::result::Result<NodeID, ComponentTypeError> {
        if let Some(id) = self.node_ids.get(&node_name)? {
            return Ok(*id);
        } else if let Some(id) = graph.get_node_annos().get_node_id_from_name(&node_name)? {
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

        let dfs = CycleSafeDFS::new_inverse(&union, node, 0, usize::MAX);
        for step in dfs {
            let step = step?;
            self.invalid_nodes.insert(step.node, true)?;
        }

        Ok(())
    }

    fn clear_left_right_token(
        &self,
        graph: &mut AnnotationGraph,
    ) -> std::result::Result<(), ComponentTypeError> {
        let component_left = AnnotationComponent::new(
            AnnotationComponentType::LeftToken,
            ANNIS_NS.into(),
            "".into(),
        );
        let component_right = AnnotationComponent::new(
            AnnotationComponentType::RightToken,
            ANNIS_NS.into(),
            "".into(),
        );
        let component_cov = AnnotationComponent::new(
            AnnotationComponentType::Coverage,
            ANNIS_NS.into(),
            "inherited-coverage".into(),
        );

        let gs_left = graph.get_or_create_writable(&component_left)?;
        if self.graph_without_nodes {
            // Make sure none of the relevant graph storages has any entries
            gs_left.clear()?;

            let gs_right = graph.get_or_create_writable(&component_right)?;
            gs_right.clear()?;

            let gs_cov = graph.get_or_create_writable(&component_cov)?;
            gs_cov.clear()?;
        } else {
            // Remove existing left/right token edges for the invalidated nodes only
            for invalid in self.invalid_nodes.iter()? {
                let (n, _) = invalid?;
                gs_left.delete_node(n)?;
            }

            let gs_right = graph.get_or_create_writable(&component_right)?;
            for invalid in self.invalid_nodes.iter()? {
                let (n, _) = invalid?;
                gs_right.delete_node(n)?;
            }

            let gs_cov = graph.get_or_create_writable(&component_cov)?;
            for invalid in self.invalid_nodes.iter()? {
                let (n, _) = invalid?;
                gs_cov.delete_node(n)?;
            }
        }
        Ok(())
    }

    fn reindex_inherited_coverage(
        &self,
        graph: &mut AnnotationGraph,
        gs_order: Arc<dyn GraphStorage>,
    ) -> std::result::Result<(), ComponentTypeError> {
        self.clear_left_right_token(graph)?;

        let inherited_cov_component = AnnotationComponent::new(
            AnnotationComponentType::Coverage,
            ANNIS_NS.into(),
            "inherited-coverage".into(),
        );
        let all_cov_components: Vec<_> = graph
            .get_all_components(Some(AnnotationComponentType::Coverage), None)
            .into_iter()
            .filter(|c| c != &inherited_cov_component)
            .collect();

        let all_cov_gs: Vec<_> = all_cov_components
            .iter()
            .filter_map(|c| graph.get_graphstorage(c))
            .collect();

        let all_dom_components =
            graph.get_all_components(Some(AnnotationComponentType::Dominance), None);
        let all_text_coverage_components: Vec<AnnotationComponent> =
            [all_cov_components, all_dom_components].concat();

        // go over each node and calculate the left-most and right-most token
        for invalid in self.invalid_nodes.iter()? {
            let (n, _) = invalid?;
            let covered_token = calculate_inherited_coverage_edges(
                graph,
                n,
                &all_cov_gs,
                &all_text_coverage_components,
                &inherited_cov_component,
            )?;
            self.calculate_token_alignment(
                graph,
                n,
                AnnotationComponentType::LeftToken,
                gs_order.as_ref(),
                &covered_token,
            )?;
            self.calculate_token_alignment(
                graph,
                n,
                AnnotationComponentType::RightToken,
                gs_order.as_ref(),
                &covered_token,
            )?;
        }

        Ok(())
    }

    fn calculate_token_alignment(
        &self,
        graph: &mut AnnotationGraph,
        n: NodeID,
        ctype: AnnotationComponentType,
        gs_order: &dyn GraphStorage,
        covered_token: &FxHashSet<u64>,
    ) -> std::result::Result<Option<NodeID>, ComponentTypeError> {
        let alignment_component =
            AnnotationComponent::new(ctype.clone(), ANNIS_NS.into(), "".into());

        // if this is a token (and not only a segmentation node), return the token itself
        if graph
            .get_node_annos()
            .get_value_for_item(&n, &TOKEN_KEY)?
            .is_some()
            && covered_token.is_empty()
        {
            return Ok(Some(n));
        }

        // if the node already has a left/right token, just return this value
        if let Some(alignment_gs) = graph.get_graphstorage_as_ref(&alignment_component) {
            if let Some(existing) = alignment_gs.get_outgoing_edges(n).next() {
                let existing = existing?;
                return Ok(Some(existing));
            }
        }

        // order the candidate token by their position in the order chain
        let mut candidates: Vec<u64> = covered_token.iter().copied().collect();
        quicksort::sort(&mut candidates, move |a, b| {
            if *a == *b {
                return Ok(std::cmp::Ordering::Equal);
            }
            if gs_order.is_connected(*a, *b, 1, std::ops::Bound::Unbounded)? {
                return Ok(std::cmp::Ordering::Less);
            } else if gs_order.is_connected(*b, *a, 1, std::ops::Bound::Unbounded)? {
                return Ok(std::cmp::Ordering::Greater);
            }
            Ok(std::cmp::Ordering::Equal)
        })?;

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
    type GlobalStatistics = AQLGlobalStatistics;

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
        let node_ids = DiskMap::new(
            None,
            EvictionStrategy::MaximumItems(1_000_000),
            DEFAULT_BLOCK_CACHE_CAPACITY,
            BtreeConfig::default().fixed_value_size(9),
        )?;

        // Calculating the invalid nodes adds additional computational overhead. If there are no nodes yet in the graph,
        // we already know that all new nodes are invalid and don't need calculate the invalid ones.
        let graph_without_nodes = graph.get_node_annos().is_empty()?;

        let invalid_nodes: DiskMap<NodeID, bool> = DiskMap::new(
            None,
            EvictionStrategy::MaximumItems(1_000_000),
            DEFAULT_BLOCK_CACHE_CAPACITY,
            BtreeConfig::default().fixed_key_size(8).fixed_value_size(2),
        )?;

        let mut text_coverage_components = FxHashSet::default();
        text_coverage_components
            .extend(graph.get_all_components(Some(AnnotationComponentType::Dominance), Some("")));
        text_coverage_components
            .extend(graph.get_all_components(Some(AnnotationComponentType::Coverage), None));
        Ok(AQLUpdateGraphIndex {
            node_ids,
            graph_without_nodes,
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
                if !index.graph_without_nodes {
                    let existing_node_id =
                        index.get_cached_node_id_from_name(Cow::Borrowed(node_name), graph)?;
                    if !index.invalid_nodes.contains_key(&existing_node_id)? {
                        index.calculate_invalidated_nodes_by_coverage(graph, existing_node_id)?;
                    }
                }
            }
            UpdateEvent::DeleteEdge {
                source_node,
                target_node,
                component_type,
                ..
            } => {
                if !index.graph_without_nodes {
                    if let Ok(ctype) = AnnotationComponentType::from_str(component_type) {
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
        if let UpdateEvent::AddEdge {
            component_type,
            component_name,
            layer,
            source_node,
            target_node,
            ..
        } = update
        {
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

                if !index.graph_without_nodes {
                    if ctype == AnnotationComponentType::Coverage
                        || ctype == AnnotationComponentType::Dominance
                        || ctype == AnnotationComponentType::Ordering
                        || ctype == AnnotationComponentType::LeftToken
                        || ctype == AnnotationComponentType::RightToken
                    {
                        let source =
                            index.get_cached_node_id_from_name(Cow::Owned(source_node), graph)?;

                        index.calculate_invalidated_nodes_by_coverage(graph, source)?;
                    }

                    if ctype == AnnotationComponentType::Ordering {
                        let target =
                            index.get_cached_node_id_from_name(Cow::Owned(target_node), graph)?;
                        index.calculate_invalidated_nodes_by_coverage(graph, target)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn apply_update_graph_index(
        mut index: Self::UpdateGraphIndex,
        graph: &mut AnnotationGraph,
    ) -> std::result::Result<(), ComponentTypeError> {
        if index.graph_without_nodes {
            // All new added nodes need to be marked as invalid
            // Do not use the node name for this because extracting it can be
            // quite expensive. Instead, query for all nodes and directly
            // get their numeric node ID
            let node_search = graph.get_node_annos().exact_anno_search(
                Some(&NODE_TYPE_KEY.ns),
                &NODE_TYPE_KEY.name,
                ValueSearch::Any,
            );
            for m in node_search {
                let m = m?;
                index.invalid_nodes.insert(m.node, true)?;
            }
        }
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

    fn calculate_global_statistics(
        graph: &mut Graph<Self>,
    ) -> std::result::Result<(), ComponentTypeError> {
        // Determine if all nodes having an "annis::tok" label are actually part
        // of an ordering component or if there are texts with only one token.
        let default_ordering_component = Component::new(
            AnnotationComponentType::Ordering,
            ANNIS_NS.into(),
            "".into(),
        );

        let token_helper = TokenHelper::new(graph)?;
        let mut all_token_in_order_component = false;
        let mut base_token_count = 0;
        let mut token_count_by_ordering_component = HashMap::new();

        if let Some(ordering_gs) = graph.get_graphstorage_as_ref(&default_ordering_component) {
            all_token_in_order_component = true;
            for m in graph.get_node_annos().exact_anno_search(
                Some(&TOKEN_KEY.ns),
                &TOKEN_KEY.name,
                ValueSearch::Any,
            ) {
                let n = m?.node;
                // Check if this is an actual token or  a segmentation node

                if !token_helper.has_outgoing_coverage_edges(n)? {
                    all_token_in_order_component = all_token_in_order_component
                        && ordering_gs.has_outgoing_edges(n)?
                        || ordering_gs.has_ingoing_edges(n)?;
                    // Update the token counter
                    base_token_count += 1;
                }
            }
        }

        // Get all non-default ordering components and count their members
        for ordering_component in
            graph.get_all_components(Some(AnnotationComponentType::Ordering), None)
        {
            if !ordering_component.name.is_empty() {
                if let Some(gs_stats) = graph
                    .get_graphstorage_as_ref(&ordering_component)
                    .and_then(|gs| gs.get_statistics())
                {
                    token_count_by_ordering_component
                        .insert(ordering_component, gs_stats.nodes as u64);
                }
            }
        }

        graph.global_statistics = Some(AQLGlobalStatistics {
            all_token_in_order_component,
            corpus_size: CorpusSize::Token {
                base_token_count,
                segmentation_count: token_count_by_ordering_component
                    .into_iter()
                    .map(|(k, v)| (k.name.into(), v))
                    .collect(),
            },
        });
        Ok(())
    }
}

impl fmt::Display for AnnotationComponentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests;
