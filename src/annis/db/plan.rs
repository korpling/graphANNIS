use annis::db::exec::{Desc, EmptyResultSet, ExecutionNode};
use annis::db::query::disjunction::Disjunction;
use annis::db::query::Config;
use annis::db::{Graph, Match};
use annis::errors::*;
use annis::types::{AnnoKeyID, NodeID};
use std;
use std::collections::HashSet;
use std::fmt::Formatter;

pub struct ExecutionPlan<'a> {
    plans: Vec<Box<ExecutionNode<Item = Vec<Match>> + 'a>>,
    current_plan: usize,
    descriptions: Vec<Option<Desc>>,
    proxy_mode: bool,
    unique_result_set: HashSet<Vec<(NodeID, AnnoKeyID)>>,
}

impl<'a> ExecutionPlan<'a> {
    pub fn from_disjunction(
        query: &'a Disjunction<'a>,
        db: &'a Graph,
        config: Config,
    ) -> Result<ExecutionPlan<'a>> {
        let mut plans: Vec<Box<ExecutionNode<Item = Vec<Match>> + 'a>> = Vec::new();
        let mut descriptions: Vec<Option<Desc>> = Vec::new();
        for alt in query.alternatives.iter() {
            let p = alt.make_exec_node(db, &config);
            if let Ok(p) = p {
                descriptions.push(p.get_desc().cloned());
                plans.push(p);
            } else if let Err(e) = p {
                match e.kind() {
                    ErrorKind::AQLSemanticError(_, _) => return Err(e),
                    _ => {}
                }
            }
        }

        if plans.is_empty() {
            // add a dummy execution step that yields no results
            let no_results_exec = EmptyResultSet {};
            plans.push(Box::new(no_results_exec));
            descriptions.push(None);
        }
        Ok(ExecutionPlan {
            current_plan: 0,
            descriptions,
            proxy_mode: plans.len() == 1,
            plans,
            unique_result_set: HashSet::new(),
        })
    }
}

impl<'a> std::fmt::Display for ExecutionPlan<'a> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for (i, d) in self.descriptions.iter().enumerate() {
            if i > 0 {
                writeln!(f, "---[OR]---")?;
            }
            if let &Some(ref d) = d {
                write!(f, "{}", d.debug_string(""))?;
            } else {
                write!(f, "<no description>")?;
            }
        }
        Ok(())
    }
}

impl<'a> Iterator for ExecutionPlan<'a> {
    type Item = Vec<Match>;

    fn next(&mut self) -> Option<Vec<Match>> {
        let n = if self.proxy_mode {
            // just act as an proxy
            self.plans[0].next()
        } else {
            let mut n = None;
            while self.current_plan < self.plans.len() {
                n = self.plans[self.current_plan].next();
                if let Some(ref res) = n {
                    // check if we already outputted this result
                    let key: Vec<(NodeID, AnnoKeyID)> = res
                        .iter()
                        .map(|m: &Match| (m.node, m.anno_key.clone()))
                        .collect();
                    if self.unique_result_set.insert(key) {
                        // new result found, break out of while-loop and return the result
                        break;
                    }
                } else {
                    // proceed to next plan
                    self.current_plan += 1;
                }
            }
            n
        };

        if let Some(tmp) = n {
            if let Some(ref desc) = self.descriptions[self.current_plan] {
                let desc: &Desc = desc;
                // re-order the matched nodes by the original node position of the query
                let mut result: Vec<Match> = Vec::new();
                result.reserve(tmp.len());
                for i in 0..tmp.len() {
                    if let Some(mapped_pos) = desc.node_pos.get(&i) {
                        result.push(tmp[mapped_pos.clone()].clone());
                    } else {
                        result.push(tmp[i].clone());
                    }
                }
                return Some(result);
            } else {
                return Some(tmp);
            }
        } else {
            return None;
        }
    }
}
