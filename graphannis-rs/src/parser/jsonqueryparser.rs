use json;
use query::conjunction::Conjunction;
use query::disjunction::Disjunction;

pub fn parse(query_as_string : &str) -> Option<Disjunction> {
    let parsed = json::parse(query_as_string);

    if let Ok(root) = parsed {

        let mut conjunctions : Vec<Conjunction> = Vec::new();
        // iterate over all alternatives
        match root["alternatives"] {
            json::JsonValue::Array (ref alternatices) => {
                for alt in alternatices.iter() {

                    let mut q = Conjunction::new();

                    conjunctions.push(q);
                    unimplemented!();
                }
            },
            _ => {
                return None;
            }
        };

        if !conjunctions.is_empty() {
            return Some(Disjunction::new(conjunctions));
        }     
    }

    return None;
}