table! {
    corpus_groups (group, corpus) {
        group -> Text,
        corpus -> Text,
    }
}

table! {
    groups (name) {
        name -> Text,
    }
}

joinable!(corpus_groups -> groups (group));

allow_tables_to_appear_in_same_query!(corpus_groups, groups,);
