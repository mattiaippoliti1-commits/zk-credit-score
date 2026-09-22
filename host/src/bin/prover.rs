use host::generate_proof;

fn main() -> Result<(), String> {
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

    println!("Generating proof...");

    let proof = generate_proof(input)?;

    println!(
        "Proving time: {:.3} seconds",
        proof.proving_time.as_secs_f64()
    );

    let result = proof.assessment;

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

    println!();
    println!("Receipt saved to receipt.json");
    Ok(())
}
