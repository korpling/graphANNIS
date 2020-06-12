/// A struct that contains the extended results of the count query.
#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct CountExtra {
    /// Total number of matches.
    pub match_count: u64,
    /// Number of documents with at least one match.
    pub document_count: u64,
}

/// Definition of the result of a `frequency` query.
///
/// This is a vector of rows, and each row is a vector of columns with the different
/// attribute values and a number of matches having this combination of attribute values.
pub type FrequencyTable<T> = Vec<(Vec<T>, usize)>;

/// Description of an attribute of a query.
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LineColumn {
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for LineColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CorpusConfiguration {
    #[serde(default)]
    context: ContextConfiguration,
    view: ViewConfiguration,
}

/// Configuration for configuring context in subgraph queries.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextConfiguration {
    /// The default context size.
    default: usize,
    /// Available context sizes to choose from.
    sizes: Vec<usize>,
    /// If set, a maximum context size which should be enforced by the query system.
    max: Option<usize>,
    /// Default segmentation to use for defining the context, `None` if tokens should be used.
    segmentation: Option<String>,
}

impl Default for ContextConfiguration {
    fn default() -> Self {
        ContextConfiguration {
            default: 5,
            segmentation: None,
            max: None,
            sizes: vec![1, 2, 5, 10],
        }
    }
}

/// Configuration how the results of a query should be shown
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViewConfiguration {
    /// Default segmentation to use for the displaying the text, `None` if tokens should be used.
    base_text_segmentation: Option<String>,
    /// Default number of results to show at once for paginated queries.
    page_size: usize,
    /// Default available settings for how many results should be part of a paginated result
    available_page_sizes: Vec<usize>,
}

impl Default for ViewConfiguration {
    fn default() -> Self {
        ViewConfiguration {
            base_text_segmentation: None,
            page_size: 10,
            available_page_sizes: vec![1, 2, 5, 10, 20, 25],
        }
    }
}
