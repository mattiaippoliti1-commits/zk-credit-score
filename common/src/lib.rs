use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialData {
    pub monthly_income: u64,
    pub monthly_expenses: u64,
    pub total_debt: u64,
    pub monthly_debt_service: u64,
    pub requested_loan: u64,
    pub loan_duration_months: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditMetrics {
    /// Debt-to-Income ratio in basis points.
    pub dti_bps: u64,

    /// Debt-Service-to-Income ratio in basis points.
    pub dsti_bps: u64,

    /// Loan-to-Income ratio in basis points.
    pub lti_bps: u64,

    /// Monthly income remaining after expenses and existing debt service.
    pub disposable_income: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditScore {
    pub dti_score: u64,
    pub dsti_score: u64,
    pub lti_score: u64,
    pub disposable_score: u64,
    pub score: u64,
    pub eligible: bool,
}

/// Represents the result of a creditworthiness assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentResult {
    pub credit_score: u64,
    pub eligible: bool,
    pub input_hash: [u8; 32],
}


pub fn hash_financial_data(data: &FinancialData) -> [u8; 32] {
    let mut hasher = Sha256::new();

    hasher.update(data.monthly_income.to_le_bytes());
    hasher.update(data.monthly_expenses.to_le_bytes());
    hasher.update(data.total_debt.to_le_bytes());
    hasher.update(data.monthly_debt_service.to_le_bytes());
    hasher.update(data.requested_loan.to_le_bytes());
    hasher.update(data.loan_duration_months.to_le_bytes());

    hasher.finalize().into()
}

pub fn build_assessment_result(
    data: &FinancialData,
    credit_score: &CreditScore,
) -> AssessmentResult {
    AssessmentResult {
        credit_score: credit_score.score,
        eligible: credit_score.eligible,
        input_hash: hash_financial_data(data),
    }
}

pub fn calculate_metrics(data: &FinancialData) -> CreditMetrics {
    let annual_income = data.monthly_income * 12;

    if data.monthly_income == 0 {
        return CreditMetrics {
            dti_bps: 10_000,
            dsti_bps: 10_000,
            lti_bps: 10_000,
            disposable_income: 0,
        };
    }

    let dti_bps = data.total_debt * 10_000 / annual_income;

    let dsti_bps =
        data.monthly_debt_service * 10_000 / data.monthly_income;

    let lti_bps =
        data.requested_loan * 10_000 / annual_income;

    let disposable_income = data
        .monthly_income
        .saturating_sub(data.monthly_expenses)
        .saturating_sub(data.monthly_debt_service);

    CreditMetrics {
        dti_bps,
        dsti_bps,
        lti_bps,
        disposable_income,
    }
}

/// Convert a ratio expressed in basis points into a score from 0 to 100.
///
/// Thresholds:
/// <= 20%  -> 100
/// <= 30%  -> 80
/// <= 40%  -> 60
/// <= 50%  -> 30
/// >  50%  -> 0
fn score_ratio(ratio_bps: u64) -> u64 {
    match ratio_bps {
        0..=2_000 => 100,
        2_001..=3_000 => 80,
        3_001..=4_000 => 60,
        4_001..=5_000 => 30,
        _ => 0,
    }
}

pub fn calculate_credit_score(
    data: &FinancialData,
    metrics: &CreditMetrics,
) -> CreditScore {
    let dti_score = score_ratio(metrics.dti_bps);

    let dsti_score = score_ratio(metrics.dsti_bps);

    let lti_score = match metrics.lti_bps {
        0..=3_000 => 100,
        3_001..=5_000 => 80,
        5_001..=7_000 => 60,
        7_001..=10_000 => 30,
        _ => 0,
    };

    let disposable_ratio_bps =
        if data.monthly_income == 0 {
            0
        } else {
            metrics.disposable_income * 10_000 / data.monthly_income
        };

    let disposable_score = match disposable_ratio_bps {
        5_000..=10_000 => 100,
        4_000..=4_999 => 80,
        3_000..=3_999 => 60,
        2_000..=2_999 => 30,
        _ => 0,
    };

    let score =
        (dti_score + dsti_score + lti_score + disposable_score) / 4;

    let eligible = score >= 60;

    CreditScore {
        dti_score,
        dsti_score,
        lti_score,
        disposable_score,
        score,
        eligible,
    }
}

// Unit tests for the creditworthiness metrics and score calculations.
#[cfg(test)]
mod tests {
    use super::*;
    // Test case for creditworthiness metrics calculation
    #[test]
    fn test_credit_metrics() {
        let data = FinancialData {
            monthly_income: 3500,
            monthly_expenses: 1500,
            total_debt: 8000,
            monthly_debt_service: 450,
            requested_loan: 10000,
            loan_duration_months: 36,
        };

        let metrics = calculate_metrics(&data);

        assert_eq!(metrics.dti_bps, 1904);
        assert_eq!(metrics.dsti_bps, 1285);
        assert_eq!(metrics.lti_bps, 2380);
        assert_eq!(metrics.disposable_income, 1550);
    }

    // Test case for credit score calculation
    #[test]
    fn test_credit_score_positive_case() {
        let data = FinancialData {
            monthly_income: 3500,
            monthly_expenses: 1500,
            total_debt: 8000,
            monthly_debt_service: 450,
            requested_loan: 10000,
            loan_duration_months: 36,
        };

        let metrics = calculate_metrics(&data);
        let score = calculate_credit_score(&data, &metrics);

        assert_eq!(score.dti_score, 100);
        assert_eq!(score.dsti_score, 100);
        assert_eq!(score.lti_score, 100);
        assert_eq!(score.disposable_score, 80);
        assert_eq!(score.score, 95);
        assert!(score.eligible);
    }

    #[test]
    fn test_score_ratio_boundaries() {
        assert_eq!(score_ratio(2_000), 100);
        assert_eq!(score_ratio(2_001), 80);

        assert_eq!(score_ratio(3_000), 80);
        assert_eq!(score_ratio(3_001), 60);

        assert_eq!(score_ratio(4_000), 60);
        assert_eq!(score_ratio(4_001), 30);

        assert_eq!(score_ratio(5_000), 30);
        assert_eq!(score_ratio(5_001), 0);
    }

    #[test]
    fn test_credit_score_negative_case() {
        let data = FinancialData {
            monthly_income: 2000,
            monthly_expenses: 1700,
            total_debt: 50000,
            monthly_debt_service: 1200,
            requested_loan: 50000,
            loan_duration_months: 36,
        };

        let metrics = calculate_metrics(&data);
        let score = calculate_credit_score(&data, &metrics);

        assert!(!score.eligible);
        assert!(score.score < 60);
    }
    // Test case for zero income
    #[test]
    fn test_zero_income() {
        let data = FinancialData {
            monthly_income: 0,
            monthly_expenses: 1000,
            total_debt: 5000,
            monthly_debt_service: 500,
            requested_loan: 10000,
            loan_duration_months: 36,
        };

        let metrics = calculate_metrics(&data);
        let score = calculate_credit_score(&data, &metrics);

        assert_eq!(metrics.disposable_income, 0);
        assert_eq!(metrics.dti_bps, 10_000);
        assert_eq!(metrics.dsti_bps, 10_000);
        assert_eq!(metrics.lti_bps, 10_000);
        assert!(!score.eligible);
    }

    // Test for hash function determinism
    #[test]
    fn test_financial_data_hash_is_deterministic() {
        let data = FinancialData {
            monthly_income: 3500,
            monthly_expenses: 1500,
            total_debt: 8000,
            monthly_debt_service: 450,
            requested_loan: 10000,
            loan_duration_months: 36,
        };

        let hash1 = hash_financial_data(&data);
        let hash2 = hash_financial_data(&data);

        assert_eq!(hash1, hash2);
    }

    // Test for hash function changes with different input
    #[test]
    fn test_financial_data_hash_changes_with_input() {
        let data1 = FinancialData {
            monthly_income: 3500,
            monthly_expenses: 1500,
            total_debt: 8000,
            monthly_debt_service: 450,
            requested_loan: 10000,
            loan_duration_months: 36,
        };

        let data2 = FinancialData {
            monthly_income: 3501,
            monthly_expenses: 1500,
            total_debt: 8000,
            monthly_debt_service: 450,
            requested_loan: 10000,
            loan_duration_months: 36,
        };

        let hash1 = hash_financial_data(&data1);
        let hash2 = hash_financial_data(&data2);

        assert_ne!(hash1, hash2);
    }
}