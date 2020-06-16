use crate::{api::auth::Claims, errors::ServiceError, models::CorpusGroup};
use diesel::prelude::*;
use std::collections::{BTreeSet, HashSet};

pub fn authorized_corpora_from_groups(
    claims: &Claims,
    conn: &SqliteConnection,
) -> Result<BTreeSet<String>, ServiceError> {
    use crate::schema::corpus_groups::dsl::*;

    let mut allowed_corpus_groups: HashSet<String> = claims.groups.iter().cloned().collect();
    // Always allow the "anonymous" group
    allowed_corpus_groups.insert("anonymous".to_string());

    let allowed_corpora: BTreeSet<String> = corpus_groups
        .filter(group.eq_any(&allowed_corpus_groups))
        .load::<CorpusGroup>(conn)?
        .iter()
        .map(|cg| cg.corpus.clone())
        .collect();
    Ok(allowed_corpora)
}

pub fn get_group_names(conn: &SqliteConnection) -> Result<Vec<String>, ServiceError> {
    use crate::schema::groups::dsl::*;

    Ok(groups
        .select(name)
        .load::<String>(conn)?
        .into_iter()
        .collect())
}

pub fn delete_group(group_name: &str, conn: &SqliteConnection) -> Result<(), ServiceError> {
    use crate::schema::groups::dsl;

    diesel::delete(dsl::groups)
        .filter(dsl::name.eq(group_name))
        .execute(conn)?;

    Ok(())
}

pub fn get_corpora_for_group(
    group_name: &str,
    conn: &SqliteConnection,
) -> Result<Vec<String>, ServiceError> {
    use crate::schema::corpus_groups::dsl;

    Ok(dsl::corpus_groups
        .select(dsl::corpus)
        .filter(dsl::group.eq(group_name))
        .load::<String>(conn)?)
}
