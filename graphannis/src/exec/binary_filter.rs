use Match;
use super::{ExecutionNode,Desc};
use operator::Operator;
pub struct BinaryFilter<'a> {
    it: Box<Iterator<Item = Vec<Match>> + 'a>,
    desc: Option<Desc>,
}

impl<'a> BinaryFilter<'a> {
    pub fn new(
        exec: Box<ExecutionNode<Item = Vec<Match>> + 'a>,
        lhs_idx: usize,
        rhs_idx: usize,
        op: Box<Operator + 'a>,
    ) -> BinaryFilter<'a> {
        let desc = exec.get_desc().cloned();
        let it = exec.filter(move |tuple| op.filter_match(&tuple[lhs_idx], &tuple[rhs_idx]));
        let filter = BinaryFilter {
            desc,
            it: Box::new(it),

        };
        return filter;
    }
}


impl<'a> ExecutionNode for BinaryFilter<'a> {

    fn as_iter(&mut self) -> &mut Iterator<Item = Vec<Match>> {
        self
    }

    fn get_desc(&self) -> Option<&Desc> {
        self.desc.as_ref()
    }
}



impl<'a> Iterator for BinaryFilter<'a> {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        self.it.next()
    }
}
