use crate::corpusstorage::QueryLanguage;
use std::collections::BTreeMap;

/// A struct that contains the extended results of the count query.
#[derive(Debug, Default, Clone, Serialize)]
#[repr(C)]
pub struct CountExtra {
    /// Total number of matches.
    pub match_count: u64,
    /// Number of documents with at least one match.
    pub document_count: u64,
}

/// Definition of the result of a `frequency` query.
pub type FrequencyTable<T> = Vec<FrequencyTableRow<T>>;

/// Represents the unique combination of attribute values and how often this combination occurs.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrequencyTableRow<T> {
    /// Combination of different attribute values.
    pub values: Vec<T>,
    /// Number of matches having this combination of attribute values.
    pub count: usize,
}

/// Description of an attribute of a query.
#[derive(Serialize)]
pub struct QueryAttributeDescription {
    /// ID of the alternative this attribute is part of.
    pub alternative: usize,
    /// Textual representation of the query fragment for this attribute.
    pub query_fragment: String,
    // Variable name of this attribute.
    pub variable: String,
    // Optional annotation name represented by this attribute.
    pub anno_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct LineColumn {
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for LineColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct LineColumnRange {
    pub start: LineColumn,
    pub end: Option<LineColumn>,
}

impl std::fmt::Display for LineColumnRange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(end) = self.end.clone() {
            if self.start == end {
                write!(f, "{}", self.start)
            } else {
                write!(f, "{}-{}", self.start, end)
            }
        } else {
            write!(f, "{}", self.start)
        }
    }
}

/// Configuration for a corpus as defined by the corpus authors.
///
/// This allows to add certain meta-information for corpus search systems in a human-writable configuration file.
/// It should be added as linked file with the name "corpus-config.toml" to the top-level corpus.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CorpusConfiguration {
    #[serde(default)]
    pub context: ContextConfiguration,
    #[serde(default)]
    pub view: ViewConfiguration,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub example_queries: Vec<ExampleQuery>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub visualizers: Vec<VisualizerRule>,
}

/// Configuration for configuring context in subgraph queries.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextConfiguration {
    /// The default context size.
    pub default: usize,
    /// Available context sizes to choose from.
    pub sizes: Vec<usize>,
    /// If set, a maximum context size which should be enforced by the query system.
    pub max: Option<usize>,
    /// Default segmentation to use for defining the context, `None` if tokens should be used.
    pub segmentation: Option<String>,
}

impl Default for ContextConfiguration {
    fn default() -> Self {
        ContextConfiguration {
            default: 5,
            segmentation: None,
            max: None,
            sizes: vec![0, 1, 2, 5, 10],
        }
    }
}

/// Configuration how the results of a query should be shown
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViewConfiguration {
    /// Default segmentation to use for the displaying the text, `None` if tokens should be used.
    pub base_text_segmentation: Option<String>,
    /// Default number of results to show at once for paginated queries.
    pub page_size: usize,
    // A list of fully qualified annotation names that should be hidden when displayed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hidden_annos: Vec<String>,
}

impl Default for ViewConfiguration {
    fn default() -> Self {
        ViewConfiguration {
            base_text_segmentation: None,
            page_size: 10,
            hidden_annos: Vec::default(),
        }
    }
}

/// An example query for the corpus with a description.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExampleQuery {
    pub query: String,
    pub description: String,
    pub query_language: QueryLanguage,
}

/// A rule when to trigger a visualizer for a specific result.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisualizerRule {
    /// Which type of elements trigger this visualizer. If not given, all element types can trigger it.
    pub element: Option<VisualizerRuleElement>,
    /// In which layer the element needs to be part of to trigger this visualizer.
    /// Only relevant for edges, since only they are part of layers.
    /// If not given, elements of all layers trigger this visualization.
    pub layer: Option<String>,
    /// The abstract type of visualization, e.g. "tree", "discourse", "grid", ...
    pub vis_type: String,
    /// A text displayed to the user describing this visualization
    pub display_name: String,
    /// The default display state of the visualizer before any user interaction.
    #[serde(default)]
    pub visibility: VisualizerVisibility,
    /// Additional configuration given as generic map of key values to the visualizer.
    #[serde(
        default,
        serialize_with = "toml::ser::tables_last",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub mappings: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum VisualizerVisibility {
    #[serde(rename = "hidden")]
    Hidden,
    #[serde(rename = "visible")]
    Visible,
    #[serde(rename = "permanent")]
    Permanent,
    #[serde(rename = "preloaded")]
    Preloaded,
}

impl Default for VisualizerVisibility {
    fn default() -> Self {
        VisualizerVisibility::Hidden
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum VisualizerRuleElement {
    #[serde(rename = "node")]
    Node,
    #[serde(rename = "edge")]
    Edge,
}
