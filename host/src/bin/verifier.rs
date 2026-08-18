use common::AssessmentResult;
use methods::METHOD_ID;
use risc0_zkvm::Receipt;
use std::fs;

fn main() {
    println!("Loading receipt...");

    let receipt_json =
        fs::read_to_string("receipt.json")
            .expect("Failed to read receipt.json");

    let receipt: Receipt =
        serde_json::from_str(&receipt_json)
            .expect("Failed to deserialize receipt");

    println!("Receipt loaded successfully.");
    println!();

    receipt
        .verify(METHOD_ID)
        .expect("Receipt verification failed");

    let result: AssessmentResult =
        receipt.journal.decode().unwrap();

    println!("Receipt verification: VALID");
    println!();
    println!("Verified creditworthiness assessment:");
    println!("  Credit score: {}/100", result.credit_score);
    println!("  Eligible: {}", result.eligible);

    print!("  Input SHA-256: ");
    for byte in result.input_hash {
        print!("{byte:02x}");
    }
    println!();
}