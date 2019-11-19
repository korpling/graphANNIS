use crate::annis::db;
use crate::annis::db::corpusstorage::PreparationResult;
use crate::annis::db::corpusstorage::QueryLanguage;
use crate::annis::db::corpusstorage::ResultOrder;
use crate::annis::db::plan::ExecutionPlan;
use crate::annis::db::query;
use crate::annis::db::query::disjunction::Disjunction;
use crate::annis::db::sort_matches::CollationType;
use crate::annis::db::token_helper::TokenHelper;
use crate::annis::db::{Graph, Match, ValueSearch, ANNIS_NS};
use crate::annis::types::{Component, ComponentType};
use crate::annis::util::quicksort;
use crate::errors::*;
use std::collections::BTreeMap;

use rand;
use rand::seq::SliceRandom;

pub struct FindIterator<'a> {
    preps_by_corpusname: BTreeMap<String, PreparationResult<'a>>,
    query: String,
    query_language: QueryLanguage,
    offset: usize,
    limit: usize,
    order: ResultOrder,
}

impl<'a> FindIterator<'a> {
    pub fn new(
        preps_by_corpusname: BTreeMap<String, PreparationResult<'a>>,
        query: &str,
        query_language: QueryLanguage,
        offset: usize,
        limit: usize,
        order: ResultOrder,
    ) -> FindIterator<'a> {
        FindIterator {
            preps_by_corpusname,
            query: query.to_string(),
            query_language,
            offset,
            limit,
            order,
        }
    }
}

impl<'a> std::iter::Iterator for FindIterator<'a> {
    type Item = Vec<Match>;

    fn next(&mut self) -> std::option::Option<Vec<Match>> {
        None
    }
}

pub fn create_iterator_for_query<'b>(
    db: &'b Graph,
    query: &'b Disjunction,
    offset: usize,
    limit: usize,
    order: ResultOrder,
    quirks_mode: bool,
    query_config: &query::Config,
) -> Result<(Box<dyn Iterator<Item = Vec<Match>> + 'b>, Option<usize>)> {
    let mut query_config = query_config.clone();
    if order == ResultOrder::NotSorted {
        // Do execute query in parallel if the order should not be sorted to have a more stable result ordering.
        // Even if we do not promise to have a stable ordering, it should be the same
        // for the same session on the same corpus.
        query_config.use_parallel_joins = false;
    }

    let plan = ExecutionPlan::from_disjunction(query, &db, &query_config)?;

    // Try to find the relANNIS version by getting the attribute value which should be attached to the
    // toplevel corpus node.
    let mut relannis_version_33 = false;
    if quirks_mode {
        let mut relannis_version_it =
            db.node_annos
                .exact_anno_search(Some(ANNIS_NS), "relannis-version", ValueSearch::Any);
        if let Some(m) = relannis_version_it.next() {
            if let Some(v) = db.node_annos.get_value_for_item(&m.node, &m.anno_key) {
                if v == "3.3" {
                    relannis_version_33 = true;
                }
            }
        }
    }
    let mut expected_size: Option<usize> = None;
    let base_it: Box<dyn Iterator<Item = Vec<Match>>> = if order == ResultOrder::NotSorted
        || (order == ResultOrder::Normal && plan.is_sorted_by_text() && !quirks_mode)
    {
        // If the output is already sorted correctly, directly return the iterator.
        // Quirks mode may change the order of the results, thus don't use the shortcut
        // if quirks mode is active.
        Box::from(plan)
    } else {
        let estimated_result_size = plan.estimated_output_size();
        let mut tmp_results: Vec<Vec<Match>> = Vec::with_capacity(estimated_result_size);

        for mgroup in plan {
            // add all matches to temporary vector
            tmp_results.push(mgroup);
        }

        // either sort or randomly shuffle results
        if order == ResultOrder::Randomized {
            let mut rng = rand::thread_rng();
            tmp_results.shuffle(&mut rng);
        } else {
            let token_helper = TokenHelper::new(db);
            let component_order = Component {
                ctype: ComponentType::Ordering,
                layer: String::from("annis"),
                name: String::from(""),
            };

            let collation = if quirks_mode && !relannis_version_33 {
                CollationType::Locale
            } else {
                CollationType::Default
            };

            let gs_order = db.get_graphstorage_as_ref(&component_order);
            let order_func = |m1: &Vec<Match>, m2: &Vec<Match>| -> std::cmp::Ordering {
                if order == ResultOrder::Inverted {
                    db::sort_matches::compare_matchgroup_by_text_pos(
                        m1,
                        m2,
                        db.node_annos.as_ref(),
                        token_helper.as_ref(),
                        gs_order,
                        collation,
                        quirks_mode,
                    )
                    .reverse()
                } else {
                    db::sort_matches::compare_matchgroup_by_text_pos(
                        m1,
                        m2,
                        db.node_annos.as_ref(),
                        token_helper.as_ref(),
                        gs_order,
                        collation,
                        quirks_mode,
                    )
                }
            };

            if query_config.use_parallel_joins {
                quicksort::sort_first_n_items_parallel(
                    &mut tmp_results,
                    offset + limit,
                    order_func,
                );
            } else {
                quicksort::sort_first_n_items(&mut tmp_results, offset + limit, order_func);
            }
        }
        expected_size = Some(tmp_results.len());
        Box::from(tmp_results.into_iter())
    };

    Ok((base_it, expected_size))
}
