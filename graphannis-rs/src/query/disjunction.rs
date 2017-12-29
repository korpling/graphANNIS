use super::conjunction::Conjunction;

pub struct Disjunction {
    alternatives : Vec<Conjunction>,
}

impl Disjunction {
    pub fn new(alt : Conjunction) -> Disjunction {
        Disjunction {
            alternatives : vec![alt],
        }
    }
}