use common::AssessmentResult;
use methods::METHOD_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv};
use std::fs;
use std::time::Instant;

fn main() {
    let input = common::FinancialData {
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

    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    println!("Generating proof...");

    let prover = default_prover();

    // let prove_info = prover
    //     .prove(env, METHOD_ELF)
    //     .unwrap();

    let start = Instant::now();

    let prove_info = prover
        .prove(env, METHOD_ELF)
        .unwrap();

    let proving_time = start.elapsed();

    println!(
        "Proving time: {:.3} seconds",
        proving_time.as_secs_f64()
    );

    let receipt = prove_info.receipt;

    let result: AssessmentResult =
        receipt.journal.decode().unwrap();

    println!("Proof generated successfully.");
    println!();
    println!("Creditworthiness assessment:");
    println!("  Credit score: {}/100", result.credit_score);
    println!("  Eligible: {}", result.eligible);

    print!("  Input SHA-256: ");
    for byte in result.input_hash {
        print!("{byte:02x}");
    }
    println!();

    let receipt_json = serde_json::to_string(&receipt).unwrap();

    fs::write("receipt.json", receipt_json)
        .expect("Failed to write receipt.json");

    println!();
    println!("Receipt saved to receipt.json");
}