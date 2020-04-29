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
