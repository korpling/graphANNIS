use crate::{api::administration::Group, auth::Claims, errors::ServiceError, models};
use diesel::prelude::*;
use models::CorpusGroup;
use std::collections::{BTreeSet, HashSet};

pub fn authorized_corpora_from_groups(
    claims: &Claims,
    conn: &mut SqliteConnection,
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

pub fn list_groups(conn: &mut SqliteConnection) -> Result<Vec<Group>, ServiceError> {
    use crate::schema::corpus_groups::dsl::*;
    use crate::schema::groups::dsl::*;

    let result = conn.transaction::<_, ServiceError, _>(move |conn| {
        let mut result: Vec<Group> = Vec::new();
        // Collect the corpora for each group name
        for group_name in groups.select(name).load::<String>(conn)? {
            let corpora = corpus_groups
                .select(corpus)
                .filter(group.eq(&group_name))
                .load::<String>(conn)?;
            result.push(Group {
                corpora,
                name: group_name.clone(),
            })
        }
        Ok(result)
    })?;
    Ok(result)
}

pub fn delete_group(group_name: &str, conn: &mut SqliteConnection) -> Result<(), ServiceError> {
    use crate::schema::groups::dsl;

    diesel::delete(dsl::groups)
        .filter(dsl::name.eq(group_name))
        .execute(conn)?;

    Ok(())
}

pub fn add_or_replace_group(group: Group, conn: &mut SqliteConnection) -> Result<(), ServiceError> {
    use crate::schema::corpus_groups::dsl as cg_dsl;
    use crate::schema::groups::dsl as g_dsl;

    conn.transaction::<_, ServiceError, _>(move |conn| {
        // Delete all group corpus relations for this group name
        diesel::delete(cg_dsl::corpus_groups)
            .filter(cg_dsl::group.eq(group.name.as_str()))
            .execute(conn)?;

        // Delete any possible group with the same name
        diesel::delete(g_dsl::groups)
            .filter(g_dsl::name.eq(&group.name))
            .execute(conn)?;
        // Insert the group with its name
        diesel::insert_into(g_dsl::groups)
            .values(models::Group {
                name: group.name.clone(),
            })
            .execute(conn)?;
        // Insert a group -> corpus relation for all corpora belonging to this group
        for corpus in group.corpora.into_iter() {
            diesel::insert_into(cg_dsl::corpus_groups)
                .values(CorpusGroup {
                    group: group.name.clone(),
                    corpus,
                })
                .execute(conn)?;
        }
        Ok(())
    })?;

    Ok(())
}
