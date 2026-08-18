use common::{AssessmentResult, FinancialData};
// use methods::METHOD_ELF;
use methods::{METHOD_ELF, METHOD_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};
use std::fs::File;
use std::time::{Duration, Instant};
use std::io::BufReader;

pub struct ProofResult {
    pub assessment: AssessmentResult,
    pub receipt: risc0_zkvm::Receipt,
    pub proving_time: Duration,
}

pub fn generate_proof(input: FinancialData) -> ProofResult {
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();

    let start = Instant::now();

    let prove_info = prover
        .prove(env, METHOD_ELF)
        .unwrap();

    let proving_time = start.elapsed();

    let receipt = prove_info.receipt;

    let assessment: AssessmentResult =
        receipt.journal.decode().unwrap();

    // Save receipt to disk.
    let file = File::create("receipt.json")
        .expect("Failed to create receipt.json");

    serde_json::to_writer(file, &receipt)
        .expect("Failed to serialize receipt");

    ProofResult {
        assessment,
        receipt,
        proving_time,
    }
}

// Verify the receipt against the expected guest program.
pub fn verify_receipt(receipt: &risc0_zkvm::Receipt) -> Result<(), String> {
    receipt
        .verify(METHOD_ID)
        .map_err(|err| format!("Receipt verification failed: {err}"))
}

// Load the receipt from disk and deserialize it.
pub fn load_receipt() -> Result<risc0_zkvm::Receipt, String> {
    let file = File::open("receipt.json")
        .map_err(|err| format!("Unable to open receipt.json: {err}"))?;

    let reader = BufReader::new(file);

    serde_json::from_reader(reader)
        .map_err(|err| format!("Unable to deserialize receipt: {err}"))
}