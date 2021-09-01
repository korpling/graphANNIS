use super::*;

#[test]
fn parse_negation_expressions() {
    let exp = parse("tok . node & #1 !> #2", false).unwrap();
    assert_eq!(1, exp.alternatives.len());
    let exp = &exp.alternatives[0];
    //assert_eq!(2, exp.nodes.len());
    //assert_eq!(2, exp.binary_operators.len());
}
