use goose::prelude::*;

async fn loadtest_list_corpora(user: &mut GooseUser) -> TransactionResult {
    let _goose_metrics = user.get("/v1/corpora").await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    GooseAttack::initialize()?
        .register_scenario(
            scenario!("LoadtestTransactions")
                .register_transaction(transaction!(loadtest_list_corpora)),
        )
        .execute()
        .await?;

    Ok(())
}
