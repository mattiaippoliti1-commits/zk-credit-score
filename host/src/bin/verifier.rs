use common::AssessmentResult;
use methods::METHOD_ID;
use risc0_zkvm::Receipt;
use std::fs;
use std::time::Instant;

fn main() {
    println!("Loading receipt...");

    let receipt_json =
        fs::read_to_string("receipt.json")
        // fs::read_to_string("receipt-tampered.json")
            .expect("Failed to read receipt.json");

    let receipt: Receipt =
        serde_json::from_str(&receipt_json)
            .expect("Failed to deserialize receipt");

    println!("Receipt loaded successfully.");
    println!();

    let start = Instant::now();

    // Get the size of the receipt file in bytes
    let metadata =
    fs::metadata("receipt.json")
        .expect("Failed to read receipt metadata");

    println!(
        "Receipt size: {:.2} KB",
        metadata.len() as f64 / 1024.0
    );



    receipt
        .verify(METHOD_ID)
        .expect("Receipt verification failed");

    let verification_time = start.elapsed();

    println!("Receipt verification: VALID");
    println!(
        "Verification time: {:.3} seconds",
        verification_time.as_secs_f64()
    );

    let result: AssessmentResult =
        receipt.journal.decode().unwrap();

    // println!("Receipt verification: VALID");
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