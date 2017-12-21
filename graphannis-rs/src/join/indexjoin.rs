use {Annotation, Match};
use operator::Operator;
use plan::ExecutionNode;
use std;

pub struct IndexJoin {
    op: Box<Operator>,
}

impl IndexJoin {
    pub fn new(
        lhs: Box<Iterator<Item = Vec<Match>>>,
        lhs_idx: usize,
        op: Box<Operator>,
        anno_cond: Box<Fn(Annotation) -> bool>,
    ) -> IndexJoin {

       /*  let it_reachable = lhs.flat_map(|m_lhs| {
            std::iter::repeat(m_lhs.clone()).zip(op.retrieve_matches(&m_lhs[lhs_idx]))
        });

        let it_annofilter = it_reachable
            .filter(|m| anno_cond(m.1.anno.clone()))
            .map(|match_pair| {
                let mut result = match_pair.0.clone();
                result.push(match_pair.1);
                result
            });
        */
        return IndexJoin {
            op,
        }
    }
}

impl<'a> Iterator for IndexJoin {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        unimplemented!()
//        self.it.next()
    }
}

impl<'a> ExecutionNode for IndexJoin {
    fn as_iter(&mut self) -> &mut Iterator<Item = Vec<Match>> {
        self
    }
}
