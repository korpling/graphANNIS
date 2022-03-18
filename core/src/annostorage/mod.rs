pub mod inmemory;
pub mod ondisk;
pub mod symboltable;

use smallvec::SmallVec;

use crate::{
    errors::Result,
    types::{AnnoKey, Annotation, Edge, NodeID},
};
use std::sync::Arc;
use std::{borrow::Cow, error::Error};
use std::{boxed::Box, path::Path};

use crate::malloc_size_of::MallocSizeOf;

/// A match is the result of a query on an annotation storage.
#[derive(Debug, Default, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Match {
    /// The node identifier this match refers to.
    pub node: NodeID,
    /// The qualified annotation name.
    pub anno_key: Arc<AnnoKey>,
}

/// A group of single matched nodes.
///
/// cbindgen:ignore
pub type MatchGroup = SmallVec<[Match; 8]>;

impl Match {
    /// Extract the annotation for this match . The annotation value
    /// is retrieved from the `node_annos` given as argument.
    pub fn extract_annotation(
        &self,
        node_annos: &dyn AnnotationStorage<NodeID>,
    ) -> Option<Annotation> {
        let val = node_annos
            .get_value_for_item(&self.node, &self.anno_key)?
            .to_owned();
        Some(Annotation {
            key: self.anno_key.as_ref().clone(),
            val: val.into(),
        })
    }

    /// Returns true if this match is different to all the other matches given as argument.
    ///
    /// A single match is different if the node ID or the annotation key are different.
    pub fn different_to_all(&self, other: &[Match]) -> bool {
        for o in other.iter() {
            if self.node == o.node && self.anno_key == o.anno_key {
                return false;
            }
        }
        true
    }

    /// Returns true if this match is different to the other match given as argument.
    ///
    /// A single match is different if the node ID or the annotation key are different.
    pub fn different_to(&self, other: &Match) -> bool {
        self.node != other.node || self.anno_key != other.anno_key
    }
}

impl From<(Edge, Arc<AnnoKey>)> for Match {
    fn from(t: (Edge, Arc<AnnoKey>)) -> Self {
        Match {
            node: t.0.source,
            anno_key: t.1,
        }
    }
}

impl From<(NodeID, Arc<AnnoKey>)> for Match {
    fn from(t: (NodeID, Arc<AnnoKey>)) -> Self {
        Match {
            node: t.0,
            anno_key: t.1,
        }
    }
}

#[derive(Clone)]
pub enum ValueSearch<T> {
    Any,
    Some(T),
    NotSome(T),
}

impl<T> From<Option<T>> for ValueSearch<T> {
    fn from(orig: Option<T>) -> ValueSearch<T> {
        match orig {
            None => ValueSearch::Any,
            Some(v) => ValueSearch::Some(v),
        }
    }
}

impl<T> ValueSearch<T> {
    #[inline]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> ValueSearch<U> {
        match self {
            ValueSearch::Any => ValueSearch::Any,
            ValueSearch::Some(v) => ValueSearch::Some(f(v)),
            ValueSearch::NotSome(v) => ValueSearch::NotSome(f(v)),
        }
    }

    #[inline]
    pub fn as_ref(&self) -> ValueSearch<&T> {
        match *self {
            ValueSearch::Any => ValueSearch::Any,
            ValueSearch::Some(ref v) => ValueSearch::Some(v),
            ValueSearch::NotSome(ref v) => ValueSearch::NotSome(v),
        }
    }
}

/// Access annotations for nodes or edges.
pub trait AnnotationStorage<T>: Send + Sync + MallocSizeOf
where
    T: Send + Sync + MallocSizeOf,
{
    /// Insert an annotation `anno` (with annotation key and value) for an item `item`.
    fn insert(&mut self, item: T, anno: Annotation) -> Result<()>;

    /// Get all the annotation keys of a node, filtered by the optional namespace (`ns`) and `name`.
    fn get_all_keys_for_item(
        &self,
        item: &T,
        ns: Option<&str>,
        name: Option<&str>,
    ) -> Result<Vec<Arc<AnnoKey>>>;

    /// Remove the annotation given by its `key` for a specific `item`
    /// Returns the value for that annotation, if it existed.
    fn remove_annotation_for_item(&mut self, item: &T, key: &AnnoKey) -> Result<Option<Cow<str>>>;

    /// Remove all annotations.
    fn clear(&mut self) -> Result<()>;

    /// Get all qualified annotation names (including namespace) for a given annotation name
    fn get_qnames(&self, name: &str) -> Vec<AnnoKey>;

    /// Get all annotations for an `item` (node or edge).
    fn get_annotations_for_item(&self, item: &T) -> Vec<Annotation>;

    /// Get the annotation for a given `item` and the annotation `key`.
    fn get_value_for_item(&self, item: &T, key: &AnnoKey) -> Option<Cow<str>>;

    /// Returns `true` if the given `item` has an annotation for the given `key`.
    fn has_value_for_item(&self, item: &T, key: &AnnoKey) -> bool;

    /// Get the matching annotation keys for each item in the iterator.
    ///
    /// This function allows to filter the received annotation keys by specifying the namespace and name.
    fn get_keys_for_iterator<'a>(
        &'a self,
        ns: Option<&str>,
        name: Option<&str>,
        it: Box<dyn Iterator<Item = std::result::Result<T, Box<dyn Error + Send + Sync>>> + 'a>,
    ) -> Result<Vec<Match>>;

    /// Return the total number of annotations contained in this `AnnotationStorage`.
    fn number_of_annotations(&self) -> usize;

    /// Return true if there are no annotations in this `AnnotationStorage`.
    fn is_empty(&self) -> bool;

    /// Return the number of annotations contained in this `AnnotationStorage` filtered by `name` and optional namespace (`ns`).
    fn number_of_annotations_by_name(&self, ns: Option<&str>, name: &str) -> usize;

    /// Returns an iterator for all items that exactly match the given annotation constraints.
    /// The annotation `name` must be given as argument, the other arguments are optional.
    ///
    /// - `namespace`- If given, only annotations having this namespace are returned.
    /// - `name`  - Only annotations with this name are returned.
    /// - `value` - Constrain the value of the annotation.
    ///
    /// The result is an iterator over matches.
    /// A match contains the node ID and the qualifed name of the matched annotation
    /// (e.g. there can be multiple annotations with the same name if the namespace is different).
    fn exact_anno_search<'a>(
        &'a self,
        namespace: Option<&str>,
        name: &str,
        value: ValueSearch<&str>,
    ) -> Box<dyn Iterator<Item = Match> + 'a>;

    /// Returns an iterator for all items where the value matches the regular expression.
    /// The annotation `name` and the `pattern` for the value must be given as argument, the  
    /// `namespace` argument is optional and can be used as additional constraint.
    ///
    /// - `namespace`- If given, only annotations having this namespace are returned.
    /// - `name`  - Only annotations with this name are returned.
    /// - `pattern` - If given, only annotation having a value that mattches this pattern are returned.
    /// - `negated` - If true, find all annotations that do not match the value
    ///
    /// The result is an iterator over matches.
    /// A match contains the node ID and the qualifed name of the matched annotation
    /// (e.g. there can be multiple annotations with the same name if the namespace is different).
    fn regex_anno_search<'a>(
        &'a self,
        namespace: Option<&str>,
        name: &str,
        pattern: &str,
        negated: bool,
    ) -> Box<dyn Iterator<Item = Match> + 'a>;

    /// Estimate the number of results for an [annotation exact search](#tymethod.exact_anno_search) for a given an inclusive value range.
    ///
    /// - `ns` - If given, only annotations having this namespace are considered.
    /// - `name`  - Only annotations with this name are considered.
    /// - `lower_val`- Inclusive lower bound for the annotation value.
    /// - `upper_val`- Inclusive upper bound for the annotation value.
    fn guess_max_count(
        &self,
        ns: Option<&str>,
        name: &str,
        lower_val: &str,
        upper_val: &str,
    ) -> usize;

    /// Estimate the number of results for an [annotation regular expression search](#tymethod.regex_anno_search)
    /// for a given pattern.
    ///
    /// - `ns` - If given, only annotations having this namespace are considered.
    /// - `name`  - Only annotations with this name are considered.
    /// - `pattern`- The regular expression pattern.
    fn guess_max_count_regex(&self, ns: Option<&str>, name: &str, pattern: &str) -> usize;

    /// Estimate the most frequent value for a given annotation `name` with an optional namespace (`ns`).
    ///
    /// If more than one qualified annotation name matches the defnition, the more frequent value is used.
    fn guess_most_frequent_value(&self, ns: Option<&str>, name: &str) -> Option<Cow<str>>;

    /// Return a list of all existing values for a given annotation `key`.
    /// If the `most_frequent_first` parameter is true, the results are sorted by their frequency.
    fn get_all_values(&self, key: &AnnoKey, most_frequent_first: bool) -> Vec<Cow<str>>;

    /// Get all the annotation keys which are part of this annotation storage
    fn annotation_keys(&self) -> Vec<AnnoKey>;

    /// Return the item with the largest item which has an annotation value in this annotation storage.
    ///
    /// This can be used to calculate new IDs for new items.
    fn get_largest_item(&self) -> Option<T>;

    /// (Re-) calculate the internal statistics needed for estimating annotation values.
    ///
    /// An annotation storage can invalid statistics, in which case the estimation function will not return
    /// valid results.
    fn calculate_statistics(&mut self);

    /// Load the annotation from an external `location`.
    fn load_annotations_from(&mut self, location: &Path) -> Result<()>;

    /// Save the current annotation to a `location` on the disk, but do not remember this location.
    fn save_annotations_to(&self, location: &Path) -> Result<()>;
}
