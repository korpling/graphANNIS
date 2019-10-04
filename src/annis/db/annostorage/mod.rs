pub mod inmemory;
mod symboltable;

use crate::annis::db::{Match, ValueSearch};
use crate::annis::types::{AnnoKey, Annotation};
use std::borrow::Cow;

/// Access annotations for nodes or edges.
pub trait AnnotationStorage<T>: Send + Sync
where
    T: Send + Sync,
{
    /// Insert an annotation `anno` (with annotation key and value) for an item `item`.
    fn insert(&mut self, item: T, anno: Annotation);

    /// Get all the annotation keys of a node
    fn get_all_keys_for_item(&self, item: &T) -> Vec<AnnoKey>;

    fn remove_annotation_for_item(&mut self, item: &T, key: &AnnoKey) -> Option<String>;

    fn clear(&mut self);

    /// Get all qualified annotation names (including namespace) for a given annotation name
    fn get_qnames(&self, name: &str) -> Vec<AnnoKey>;

    /// Get all annotations for an `item` (node or edge).
    fn get_annotations_for_item(&self, item: &T) -> Vec<Annotation>;

    fn get_value_for_item(&self, item: &T, key: &AnnoKey) -> Option<Cow<str>>;

    /// Get the annotation keys for each item in the iterator.
    /// 
    /// This function allows to filter the received annotation keys by the specifying the namespace and name.
    fn get_keys_for_iterator(
        &self,
        ns: Option<String>,
        name: Option<String>,
        it: Box<dyn Iterator<Item = T>>,
    ) -> Vec<Match>;

    /// Return the total number of annotations contained in this `AnnotationStorage`.
    fn number_of_annotations(&self) -> usize;

    /// Return the number of annotations contained in this `AnnotationStorage` filtered by `name` and optional namespace (`ns`).
    fn number_of_annotations_by_name(&self, ns: Option<String>, name: String) -> usize;

    /// Returns an iterator for all items that exactly match the given annotation constraints.
    /// The annotation `name` must be given as argument, the other arguments are optional.
    ///
    /// - `namespace`- If given, only annotations having this namespace are returned.
    /// - `name`  - Only annotations with this name are returned.
    /// - `value` - If given, only annotation having exactly the given value are returned.
    ///
    /// The result is an iterator over matches.
    /// A match contains the node ID and the qualifed name of the matched annotation
    /// (e.g. there can be multiple annotations with the same name if the namespace is different).
    fn exact_anno_search<'a>(
        &'a self,
        namespace: Option<String>,
        name: String,
        value: ValueSearch<String>,
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
        namespace: Option<String>,
        name: String,
        pattern: &str,
        negated: bool,
    ) -> Box<dyn Iterator<Item = Match> + 'a>;

    fn find_annotations_for_item(
        &self,
        item: &T,
        ns: Option<String>,
        name: Option<String>,
    ) -> Vec<AnnoKey>;

    /// Estimate the number of results for an [annotation exact search](#tymethod.exact_anno_search) for a given an inclusive value range.
    ///
    /// - `ns` - If given, only annotations having this namespace are considered.
    /// - `name`  - Only annotations with this name are considered.
    /// - `lower_val`- Inclusive lower bound for the annotation value.
    /// - `upper_val`- Inclusive upper bound for the annotation value.
    fn guess_max_count(
        &self,
        ns: Option<String>,
        name: String,
        lower_val: &str,
        upper_val: &str,
    ) -> usize;

    /// Estimate the number of results for an [annotation regular expression search](#tymethod.regex_anno_search)
    /// for a given pattern.
    ///
    /// - `ns` - If given, only annotations having this namespace are considered.
    /// - `name`  - Only annotations with this name are considered.
    /// - `pattern`- The regular expression pattern.
    fn guess_max_count_regex(&self, ns: Option<String>, name: String, pattern: &str) -> usize;

    fn guess_most_frequent_value(&self, ns: Option<String>, name: String) -> Option<String>;

    /// Return a list of all existing values for a given annotation `key`.
    /// If the `most_frequent_first`parameter is true, the results are sorted by their frequency.
    fn get_all_values(&self, key: &AnnoKey, most_frequent_first: bool) -> Vec<Cow<str>>;

    /// Get all the annotation keys which are part of this annotation storage
    fn annotation_keys(&self) -> Vec<AnnoKey>;

    fn get_largest_item(&self) -> Option<T>;

    fn calculate_statistics(&mut self);
}
