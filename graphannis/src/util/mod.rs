pub mod token_helper;

pub mod memory_estimation;
#[macro_use]
pub mod c_api;

use Annotation;

pub fn regex_full_match(pattern: &str) -> String {
    let mut full_match_pattern = String::new();
    full_match_pattern.push_str(r"\A");
    full_match_pattern.push_str(pattern);
    full_match_pattern.push_str(r"\z");

    full_match_pattern
}

pub fn check_annotation_equal(a: &Annotation, b: &Annotation) -> bool {
    // compare by name (non lexical but just by the ID)
    if a.key.name != 0 && b.key.name != 0 && a.key.name != b.key.name {
        return false;
    }

    // if equal, compare by namespace (non lexical but just by the ID)
    if a.key.ns != 0 && b.key.ns != 0 && a.key.ns != b.key.ns {
        return false;
    }

    // if still equal compare by value (non lexical but just by the ID)
    if a.val != 0 && b.val != 0 && a.val != b.val {
        return false;
    }

    // they are equal
    return true;
}
