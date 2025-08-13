use crate::errors::{GraphAnnisCoreError, Result};
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use std::borrow::Cow;

pub mod disk_collections;

#[cfg(test)]
pub(crate) mod example_graphs;

const QNAME_ENCODE_SET: &AsciiSet = &CONTROLS.add(b' ').add(b':').add(b'%');

pub fn join_qname(ns: &str, name: &str) -> String {
    let mut result = String::with_capacity(ns.len() + name.len() + 2);
    if !ns.is_empty() {
        let encoded_anno_ns: Cow<str> = utf8_percent_encode(ns, QNAME_ENCODE_SET).into();
        result.push_str(&encoded_anno_ns);
        result.push_str("::");
    }
    let encoded_anno_name: Cow<str> = utf8_percent_encode(name, QNAME_ENCODE_SET).into();
    result.push_str(&encoded_anno_name);
    result
}

pub fn split_qname(qname: &str) -> (Option<&str>, &str) {
    let sep_pos = qname.find("::");
    if let Some(sep_pos) = sep_pos {
        (Some(&qname[..sep_pos]), &qname[sep_pos + 2..])
    } else {
        (None, qname)
    }
}

pub fn regex_full_match(pattern: &str) -> String {
    let mut full_match_pattern = String::new();
    full_match_pattern.push_str(r"\A(");
    full_match_pattern.push_str(pattern);
    full_match_pattern.push_str(r")\z");

    full_match_pattern
}

/// Parse a string as both a `Regex` that can be used for matching and as the
/// more abstract `Hir` representation that gives as information such as
/// prefixes for this regular expression.
pub fn compile_and_parse_regex(pattern: &str) -> Result<(regex::Regex, regex_syntax::hir::Hir)> {
    let compiled_regex =
        regex::Regex::new(pattern).map_err(|e| GraphAnnisCoreError::Other(Box::new(e)))?;
    let parsed_regex = regex_syntax::Parser::new()
        .parse(pattern)
        .map_err(|e| GraphAnnisCoreError::Other(Box::new(e)))?;
    Ok((compiled_regex, parsed_regex))
}
