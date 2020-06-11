use crate::{api::auth::Claims, errors::ServiceError};
use diesel::prelude::*;
use std::collections::HashSet;

pub fn corpus_access_allowed(
    requested_corpora: &Vec<String>,
    claims: &Claims,
    conn: &SqliteConnection,
) -> Result<bool, ServiceError> {
    use crate::schema::corpus_groups::dsl::*;

    if claims.admin {
        // Adminstrators are always allowed to access all corpora
        return Ok(true);
    }

    let mut allowed_corpus_groups: HashSet<String> = claims.corpus_groups.iter().cloned().collect();
    // Always allow the "anonymous" corpus group
    allowed_corpus_groups.insert("anonymous".to_string());

    let allowed_corpora: HashSet<String> = corpus_groups
        .filter(group.eq_any(&allowed_corpus_groups))
        .load::<crate::models::CorpusGroup>(conn)?
        .iter()
        .map(|cg| cg.corpus.clone())
        .collect();

    // Check if all requested corpora are allowed
    let mut allowed = true;
    for c in requested_corpora {
        if !allowed_corpora.contains(c.as_str()) {
            allowed = false;
            break;
        }
    }

    Ok(allowed)
}
