use {Annotation, Match};
use operator::Operator;
use std::rc::Rc;
use std;

pub fn new<'a>(
    lhs: Box<Iterator<Item = Match>>,
    op: Rc<Operator>,
    anno_cond: Box<Fn(Annotation) -> bool + 'a>,
) -> Box<Iterator<Item = (Match, Match)> + 'a> {
    let it = lhs.flat_map(move |m_lhs| {
        std::iter::repeat(m_lhs.clone()).zip(op.retrieve_matches(&m_lhs))
    }).filter(move |m| anno_cond(m.1.anno.clone()));

    return Box::new(it);
}
