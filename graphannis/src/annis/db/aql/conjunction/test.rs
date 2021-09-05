use crate::annis::db::aql::{
    self,
    operators::{DominanceSpec, NegatedOpSpec, PrecedenceSpec, RangeSpec},
};

#[test]
fn parse_negation_filter_expression() {
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

#[test]
fn parse_negation_between_ops_expression() {
    let mut exp = aql::parse("tok . tok !. node & #2 > #1", false).unwrap();
    assert_eq!(1, exp.alternatives.len());

    let mut alt = exp.alternatives.remove(0);

    assert_eq!(3, alt.nodes.len());
    assert_eq!(3, alt.binary_operators.len());

    let op_entry1 = alt.binary_operators.remove(1);
    let op1 = op_entry1.op.into_any();

    let op1 = op1.downcast::<NegatedOpSpec>().unwrap();
    let negated_op = op1
        .negated_op
        .into_any()
        .downcast::<PrecedenceSpec>()
        .unwrap();

    assert_eq!(None, negated_op.segmentation);
    assert_eq!(
        RangeSpec::Bound {
            min_dist: 1,
            max_dist: 1
        },
        negated_op.dist
    );

    let op_entry2 = alt.binary_operators.remove(1);
    let op2 = op_entry2.op.into_any();

    assert_eq!(true, op2.is::<DominanceSpec>());
}

#[test]
fn parse_invalid_negation() {
    assert_eq!(
        true,
        aql::parse("node !. node", false).unwrap().alternatives[0]
            .check_components_connected()
            .is_err()
    );
    assert_eq!(
        true,
        aql::parse("node !->dep node", false).unwrap().alternatives[0]
            .check_components_connected()
            .is_err()
    );
    assert_eq!(
        true,
        aql::parse("node !> node", false).unwrap().alternatives[0]
            .check_components_connected()
            .is_err()
    );
    assert_eq!(
        true,
        aql::parse("node !_=_ node", false).unwrap().alternatives[0]
            .check_components_connected()
            .is_err()
    );
    assert_eq!(
        true,
        aql::parse("node !_i_ node", false).unwrap().alternatives[0]
            .check_components_connected()
            .is_err()
    );
    assert_eq!(
        true,
        aql::parse("node !_o_ node", false).unwrap().alternatives[0]
            .check_components_connected()
            .is_err()
    );
    assert_eq!(
        true,
        aql::parse("node !_l_ node", false).unwrap().alternatives[0]
            .check_components_connected()
            .is_err()
    );
    assert_eq!(
        true,
        aql::parse("node !_r_ node", false).unwrap().alternatives[0]
            .check_components_connected()
            .is_err()
    );
}