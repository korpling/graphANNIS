use super::conjunction::Conjunction;

pub struct Disjunction<'a> {
    alternatives : Vec<Conjunction<'a>>,
}

impl<'a> Disjunction<'a> {
    pub fn new(alt : Conjunction<'a>) -> Disjunction<'a> {
        Disjunction {
            alternatives : vec![alt],
        }
    }
}