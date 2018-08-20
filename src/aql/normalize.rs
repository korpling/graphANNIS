use super::ast::*;
use std::collections::VecDeque;

/// Transforms an AQL query to the Disjunctive Normal Form.
/// 
/// 
/// In DNF all OR relations must be toplevel. Thus constructions like
/// ```plaintext
///  AND             AND
///  / \      or     / \
/// X  OR          OR   X
///    / \        /  \
///   Y   Z       Y  Z
/// ```
/// are illegal and will be replaced with
/// ```plaintext
///       OR           
///     /    \   
///   AND    AND   
///   / \    / \  
///  X   Y  X   Z  
/// ```
/// according to the distributivity rule of Boolean Algebra.
/// 
/// * `top_disjunction` The disjunction to normalize
/// 
pub fn to_disjunctive_normal_form(top_disjunction : &mut Disjunction) {
    // iterate over all conjunctions and check if they contain disjunctions
    let mut additional_conjunctions = VecDeque::new();
    for top_conjunction in top_disjunction.iter_mut() {
        let top_conjunction : &mut Conjunction = top_conjunction;
        // split up factors into terminal ones and other disjunctions
        let mut literal_factors : Vec<Literal>  = Vec::new();
        let mut disjunction_factors : Vec<Disjunction> = Vec::new();
        for f in top_conjunction.drain(..) {
            match f {
                Factor::Disjunction(mut inner_disjunction) => {
                    // make sure the nested disjunction is in DNF
                    to_disjunctive_normal_form(&mut inner_disjunction);

                    // add the resulting disjunction to extra list
                    disjunction_factors.push(inner_disjunction);
                },
                Factor::Literal(l) => {
                    literal_factors.push(l);
                },
            }
        }

        if disjunction_factors.is_empty() {
            // Disjunction is in DNF, re-add all literals
            for f in literal_factors.into_iter() {
                top_conjunction.push_back(Factor::Literal(f));
            }
        } else {
            // For each disjunction create a new conjunction containing the union of the existing terms with the ones
            // from the disjunction
            for child_disjunct in disjunction_factors.into_iter() {
                for child_conjunct in child_disjunct.into_iter() {
                    let mut new_conjunction : Conjunction = Conjunction::new();
                    for existing_literal in literal_factors.iter() {
                        new_conjunction.push_back(Factor::Literal(existing_literal.clone()));
                    }
                    for new_literal in child_conjunct.into_iter() {
                        new_conjunction.push_back(new_literal);
                    }
                    // add the new conjunction to the disjunction
                    additional_conjunctions.push_back(new_conjunction);
                } 
            } 
        }
    }

    top_disjunction.append(&mut additional_conjunctions);

    // clean all empty conjunctions
    top_disjunction.retain(|ref c| !c.is_empty());
}
