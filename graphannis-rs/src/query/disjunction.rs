use super::conjunction::Conjunction;
use {Component};

pub struct Disjunction<'a> {
    pub alternatives : Vec<Conjunction<'a>>,
}

impl<'a> Disjunction<'a> {
    pub fn new(alt : Conjunction<'a>) -> Disjunction<'a> {
        Disjunction {
            alternatives : vec![alt],
        }
    }

    pub fn necessary_components(&self) -> Vec<Component> {
        let mut result = vec![];
        
        for alt in self.alternatives.iter() {
            let mut c = alt.necessary_components();
            result.append(&mut c);
        }

        return result;
    }
}