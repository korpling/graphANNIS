use {Annotation, Match};
use operator::Operator;
use std::rc::Rc;
use std;

pub fn new<'a>(
    lhs: Box<Iterator<Item = Vec<Match>>>,
    lhs_idx : usize,
    op: Rc<Operator>,
    anno_cond: Box<Fn(Annotation) -> bool + 'a>,
) -> Box<Iterator<Item = Vec<Match>> + 'a> {
    let it = lhs.flat_map(move |m_lhs| {
        std::iter::repeat(m_lhs.clone()).zip(op.retrieve_matches(&m_lhs[lhs_idx]))
    }).filter(move |m| anno_cond(m.1.anno.clone()))
    .map(|match_pair| {let mut result = match_pair.0.clone(); result.push(match_pair.1); result});

    return Box::new(it);
}
