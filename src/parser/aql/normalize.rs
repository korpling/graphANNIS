use super::ast::*;

/// Transforms an AQL query to the Disjunctive Normal Form.
pub fn to_disjunctive_normal_form(top_node : Disjunction) -> Disjunction {

    unimplemented!()
}

/// Iteration step to transform a disjunction into DNF. 
/// If the disjunctions is already in DNF, `None` is returned.
/// 
/// In DNF all OR relations must be toplevel. Thus constructions like
/// ~~~
///  AND             AND
///  / \      or     / \
/// X  OR          OR   X
///    / \        /  \
///   Y   Z       Y  Z
/// ```
/// are illegal and will be replaced with
/// ```
///       OR           
///     /    \   
///   AND    AND   
///   / \    / \  
///  X   Y  X   Z  
/// ```
/// according to the distributivity rule of Boolean Algebra.
/// 
/// Only one transformation will be done in this function, repeat it
/// in order to replace all illegal constructions.
/// 
/// * `top_node` The disjunction to normalize
/// 
fn make_dnf(top_node : &Disjunction) -> Option<Disjunction> {

    unimplemented!()
}