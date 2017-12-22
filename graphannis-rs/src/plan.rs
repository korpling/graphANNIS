use Match;

#[derive(Debug, Clone)]
pub struct Desc {
    pub component_nr : usize,
}

impl Desc {
    pub fn join(lhs : Option<&Desc>, rhs : Option<&Desc>) -> Desc {
        let component_nr = if let Some(d) = lhs  {
            d.component_nr
        } else if let Some(d) = rhs {
            d.component_nr
        } else {
            0
        };

        Desc {
            component_nr,
        }
    }
}

pub trait ExecutionNode : Iterator {
    fn as_iter(& mut self) -> &mut Iterator<Item = Vec<Match>>;

    fn get_lhs_desc(&self) -> Option<&Desc> {
        None
    }
    fn get_rhs_desc(&self) -> Option<&Desc> {
        None
    }

    fn get_desc(&self) -> Option<&Desc> {
        None
    }
}



pub struct ExecutionPlan {
    root: Box<ExecutionNode<Item = Vec<Match>>>,
}

impl Iterator for ExecutionPlan {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        let n = self.root.as_iter().next();
        // TODO: re-organize the match positions
        return n;
    }
}
