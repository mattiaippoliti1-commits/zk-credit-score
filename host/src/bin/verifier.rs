use common::AssessmentResult;
use host::verify_receipt;
use risc0_zkvm::Receipt;
use std::fs;
use std::time::Instant;

fn main() -> Result<(), String> {
    println!("Loading receipt...");

    let receipt_json = fs::read_to_string("receipt.json")
        .map_err(|err| format!("Unable to read receipt.json: {err}"))?;

    let receipt: Receipt = serde_json::from_str(&receipt_json)
        .map_err(|err| format!("Unable to deserialize receipt: {err}"))?;

    println!("Receipt loaded successfully.");
    println!();

    let start = Instant::now();

    let metadata = fs::metadata("receipt.json")
        .map_err(|err| format!("Unable to read receipt metadata: {err}"))?;

    println!("Receipt size: {:.2} KB", metadata.len() as f64 / 1024.0);

    verify_receipt(&receipt)?;

    let verification_time = start.elapsed();

    println!("Receipt verification: VALID");
    println!(
        "Verification time: {:.3} seconds",
        verification_time.as_secs_f64()
    );

    let result: AssessmentResult = receipt
        .journal
        .decode()
        .map_err(|err| format!("Unable to decode receipt journal: {err}"))?;

    println!();
    println!("Verified creditworthiness assessment:");
    println!("  Credit score: {}/100", result.credit_score);
    println!("  Eligible: {}", result.eligible);

    print!("  Input SHA-256: ");
    for byte in result.input_hash {
        print!("{byte:02x}");
    }
    println!();
    Ok(())
}
