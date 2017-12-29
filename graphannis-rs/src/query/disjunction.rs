use super::conjunction::Conjunction;

pub struct Disjunction {
    alts : Vec<Conjunction>,
}

impl Disjunction {
    pub fn new() -> Disjunction {
        Disjunction {
            alts : vec![],
        }
    }
}