use crate::{api::auth::Claims, errors::ServiceError};
use diesel::prelude::*;
use std::collections::{BTreeSet, HashSet};

pub fn authorized_corpora_from_groups(
    claims: &Claims,
    conn: &SqliteConnection,
) -> Result<BTreeSet<String>, ServiceError> {
    use crate::schema::corpus_groups::dsl::*;

    let mut allowed_corpus_groups: HashSet<String> = claims.corpus_groups.iter().cloned().collect();
    // Always allow the "anonymous" corpus group
    allowed_corpus_groups.insert("anonymous".to_string());

    let allowed_corpora: BTreeSet<String> = corpus_groups
        .filter(group.eq_any(&allowed_corpus_groups))
        .load::<crate::models::CorpusGroup>(conn)?
        .iter()
        .map(|cg| cg.corpus.clone())
        .collect();
    Ok(allowed_corpora)
}
