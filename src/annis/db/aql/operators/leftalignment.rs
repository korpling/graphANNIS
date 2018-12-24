use crate::annis::db::graphstorage::GraphStorage;
use crate::annis::db::token_helper;
use crate::annis::db::token_helper::TokenHelper;
use crate::annis::db::Graph;
use crate::annis::db::Match;
use crate::annis::operator::Operator;
use crate::annis::operator::OperatorSpec;
use crate::annis::types::{AnnoKeyID, Component, ComponentType};
use std::sync::Arc;

#[derive(Clone, Debug, PartialOrd, Ord, Hash, PartialEq, Eq)]
pub struct LeftAlignmentSpec;

pub struct LeftAlignment {
    gs_cov: Arc<GraphStorage>,
    tok_helper: TokenHelper,
}

lazy_static! {
    static ref COMPONENT_COV: Component = {
        Component {
            ctype: ComponentType::Coverage,
            layer: String::from("annis"),
            name: String::from(""),
        }
    };
}

impl OperatorSpec for LeftAlignmentSpec {
    fn necessary_components(&self, _db: &Graph) -> Vec<Component> {
        let mut v: Vec<Component> = vec![COMPONENT_COV.clone()];
        v.append(&mut token_helper::necessary_components());
        v
    }

    fn create_operator(&self, db: &Graph) -> Option<Box<Operator>> {
        let optional_op = LeftAlignment::new(db);
        if let Some(op) = optional_op {
            return Some(Box::new(op));
        } else {
            return None;
        }
    }
}

impl LeftAlignment {
    pub fn new(db: &Graph) -> Option<LeftAlignment> {
        let gs_cov = db.get_graphstorage(&COMPONENT_COV)?;

        let tok_helper = TokenHelper::new(db)?;

        Some(LeftAlignment { gs_cov, tok_helper })
    }
}

impl std::fmt::Display for LeftAlignment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "_l_")
    }
}

impl Operator for LeftAlignment {
    fn retrieve_matches(&self, lhs: &Match) -> Box<Iterator<Item = Match>> {
        let mut aligned = Vec::default();

        if let Some(lhs_token) = self.tok_helper.left_token_for(lhs.node) {
            if lhs.node != lhs_token {
                aligned.push(Match {
                    node: lhs_token,
                    anno_key: AnnoKeyID::default(),
                });
            }
            aligned.extend(self.gs_cov.get_outgoing_edges(lhs_token).map(|n| Match {
                node: n,
                anno_key: AnnoKeyID::default(),
            }));
        }

        Box::from(aligned.into_iter())
    }

    fn filter_match(&self, lhs: &Match, rhs: &Match) -> bool {
        if let (Some(lhs_token), Some(rhs_token)) = (
            self.tok_helper.left_token_for(lhs.node),
            self.tok_helper.left_token_for(rhs.node),
        ) {
            return lhs_token == rhs_token;
        } else {
            return false;
        }
    }
}
