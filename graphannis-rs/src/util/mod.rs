pub mod token_helper;

pub mod memory_estimation;
#[macro_use]
pub mod c_api;

pub fn regex_full_match(pattern : &str) -> String {
    
    let mut full_match_pattern = String::new();
    full_match_pattern.push_str(r"\A");
    full_match_pattern.push_str(r"\z");   
    full_match_pattern.push_str(pattern);

    full_match_pattern 
}