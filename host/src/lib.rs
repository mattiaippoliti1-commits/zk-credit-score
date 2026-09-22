use common::{AssessmentResult, FinancialData};
use methods::{METHOD_ELF, METHOD_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};

pub struct ProofResult {
    pub assessment: AssessmentResult,
    pub receipt: risc0_zkvm::Receipt,
    pub proving_time: Duration,
}

pub fn generate_proof(input: FinancialData) -> Result<ProofResult, String> {
    let env = ExecutorEnv::builder()
        .write(&input)
        .map_err(|err| format!("Unable to write input to zkVM environment: {err}"))?
        .build()
        .map_err(|err| format!("Unable to build zkVM environment: {err}"))?;

    let prover = default_prover();

    let start = Instant::now();

    let prove_info = prover
        .prove(env, METHOD_ELF)
        .map_err(|err| format!("Proof generation failed: {err}"))?;

    let proving_time = start.elapsed();

    let receipt = prove_info.receipt;

    let assessment: AssessmentResult = receipt
        .journal
        .decode()
        .map_err(|err| format!("Unable to decode receipt journal: {err}"))?;

    // Save receipt to disk.
    let file = File::create("receipt.json")
        .map_err(|err| format!("Unable to create receipt.json: {err}"))?;

    serde_json::to_writer(file, &receipt)
        .map_err(|err| format!("Unable to serialize receipt: {err}"))?;

    Ok(ProofResult {
        assessment,
        receipt,
        proving_time,
    })
}

// Verify the receipt against the expected guest program.
pub fn verify_receipt(receipt: &risc0_zkvm::Receipt) -> Result<(), String> {
    receipt
        .verify(METHOD_ID)
        .map_err(|err| format!("Receipt verification failed: {err}"))
}

// Load the receipt from disk and deserialize it.
pub fn load_receipt() -> Result<risc0_zkvm::Receipt, String> {
    let file =
        File::open("receipt.json").map_err(|err| format!("Unable to open receipt.json: {err}"))?;

    let reader = BufReader::new(file);

    serde_json::from_reader(reader).map_err(|err| format!("Unable to deserialize receipt: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::{hash_financial_data, AssessmentResult};

    fn sample_input() -> FinancialData {
        FinancialData {
            monthly_income: 3500,
            monthly_expenses: 1500,
            total_debt: 8000,
            monthly_debt_service: 450,
            requested_loan: 10000,
            loan_duration_months: 36,
            interest_rate_bps: 350,
            dependents: 0,
            age_years: 30,
        }
    }

    #[test]
    fn valid_receipt_verifies_and_tampered_journal_fails_verification() {
        std::env::set_var("RISC0_DEV_MODE", "1");

        let input = sample_input();
        let proof = generate_proof(input.clone()).expect("proof generation should succeed");

        verify_receipt(&proof.receipt).expect("valid receipt should verify");
        assert!(
            proof.receipt.verify([0; 8]).is_err(),
            "receipt must not verify against a different image ID"
        );

        let assessment: AssessmentResult = proof
            .receipt
            .journal
            .decode()
            .expect("valid journal should decode");

        assert_eq!(assessment.credit_score, 95);
        assert!(assessment.eligible);
        assert_eq!(assessment.input_hash, hash_financial_data(&input));

        let mut receipt_json =
            serde_json::to_value(&proof.receipt).expect("receipt should serialize");
        let journal_bytes = receipt_json["journal"]["bytes"]
            .as_array_mut()
            .expect("serialized receipt should contain journal bytes");

        let first_byte = journal_bytes[0]
            .as_u64()
            .expect("journal byte should be an integer");
        journal_bytes[0] = serde_json::json!(first_byte ^ 1);

        let tampered_receipt: risc0_zkvm::Receipt =
            serde_json::from_value(receipt_json).expect("tampered receipt should deserialize");

        assert!(
            verify_receipt(&tampered_receipt).is_err(),
            "changing an authenticated journal byte must invalidate the receipt"
        );
    }
}
