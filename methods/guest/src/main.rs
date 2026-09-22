use common::{
    build_assessment_result,
    calculate_credit_score,
    calculate_metrics,
    FinancialData,
};
use risc0_zkvm::guest::env;

fn main() {
    // Read private financial data.
    let data: FinancialData = env::read();

    // Compute financial metrics.
    let metrics = calculate_metrics(&data);

    // Compute credit score and eligibility.
    let credit_score = calculate_credit_score(&data, &metrics);

    // Build the public assessment result and input digest.
    let result = build_assessment_result(&data, &credit_score);

    // Commit only public information to the journal.
    env::commit(&result);
}
