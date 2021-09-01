use crate::annis::db::aql::{
    self,
    operators::{DominanceSpec, NegatedOpSpec, PrecedenceSpec, RangeSpec},
};

#[test]
fn parse_negation_expression() {
    let mut exp = aql::parse("tok . node & #1 !> #2", false).unwrap();
    assert_eq!(1, exp.alternatives.len());

    let mut alt = exp.alternatives.remove(0);

    assert_eq!(2, alt.nodes.len());
    assert_eq!(2, alt.binary_operators.len());

    let op_entry1 = alt.binary_operators.remove(0);
    let op1 = op_entry1.op.into_any();

    let op_entry2 = alt.binary_operators.remove(0);
    let op2 = op_entry2.op.into_any();

    assert_eq!(true, op1.is::<PrecedenceSpec>());

    let op2 = op2.downcast::<NegatedOpSpec>().unwrap();
    let negated_op = op2
        .negated_op
        .into_any()
        .downcast::<DominanceSpec>()
        .unwrap();

    assert_eq!("", negated_op.name);
    assert_eq!(
        RangeSpec::Bound {
            min_dist: 1,
            max_dist: 1
        },
        negated_op.dist
    );
}
