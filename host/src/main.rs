use common::FinancialData;
use host::{generate_proof, verify_receipt};

fn main() -> Result<(), String> {
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    // Financial input data.
    let input = FinancialData {
        monthly_income: 3500,
        monthly_expenses: 1500,
        total_debt: 8000,
        monthly_debt_service: 450,
        requested_loan: 10000,
        loan_duration_months: 36,
        interest_rate_bps: 350,
        dependents: 0,
        age_years: 30,
    };

    let proof = generate_proof(input)?;
    let result = proof.assessment;

    println!("Creditworthiness assessment:");
    println!("  Credit score: {}/100", result.credit_score);
    println!("  Eligible: {}", result.eligible);

    print!("  Input SHA-256: ");
    for byte in result.input_hash {
        print!("{byte:02x}");
    }
    println!();

    verify_receipt(&proof.receipt)?;

    println!("Proof verification: VALID");
    Ok(())
}
