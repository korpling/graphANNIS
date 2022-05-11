use goose::prelude::*;

const DDD_QUERIES: [&str; 3] = [
    r#"edition & edition & text & #1_o_#3 & #2_o_#3 & #1.#2"#,
    r#"text & text & edition & #1 _o_ #3 & #2 _o_ #3 & #1 . #2"#,
    r#"text & text & edition & #1 _o_ #3 & #2 _o_ #3"#,
];

async fn loadtest_ddd(user: &mut GooseUser) -> TransactionResult {
    // Get all corpora and select the DDD-AD version 1.2 ones
    let response = user.get("/v1/corpora").await?;
    let corpus_list: Vec<String> = response.response?.json().await?;
    let ddd_corpora: Vec<_> = corpus_list
        .into_iter()
        .filter(|c| c.starts_with("DDD-AD") && c.ends_with("_1.2"))
        .collect();

    // Execute all find queries on all corpora and get the subgraph for the
    // first 10 matches. Randomize output order so we get different corpora all
    // the time.
    for q in DDD_QUERIES {
        let json = serde_json::json!({
          "query": q,
          "query_language": "AQL",
          "corpora": ddd_corpora,
          "limit": 10,
          "offset": 0,
          "order": "Randomized"
        });
        user.post_json("/v1/search/find", &json).await?;
        // TODO: subgraph query
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    GooseAttack::initialize()?
        .register_scenario(
            scenario!("ParallelCorpusAccess").register_transaction(transaction!(loadtest_ddd)),
        )
        .execute()
        .await?;

    Ok(())
}
