use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
// Struct to hold financial data for credit scoring
pub struct FinancialData {
    pub monthly_income: u64,
    pub monthly_expenses: u64,
    pub total_debt: u64,
    pub monthly_debt_service: u64,
    pub requested_loan: u64,
    pub loan_duration_months: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
// Struct to hold calculated credit metrics
pub struct CreditMetrics {
    pub dti_bps: u64,
    pub dsti_bps: u64,
    pub lti_bps: u64,
    pub disposable_income: u64,
}

// Function to calculate credit metrics based on financial data
pub fn calculate_metrics(data: &FinancialData) -> CreditMetrics {
    let annual_income = data.monthly_income * 12;

    let dti_bps = data.total_debt * 10_000 / annual_income;

    let dsti_bps =
        data.monthly_debt_service * 10_000 / data.monthly_income;

    let lti_bps =
        data.requested_loan * 10_000 / annual_income;

    let disposable_income =
        data.monthly_income
            .saturating_sub(data.monthly_expenses)
            .saturating_sub(data.monthly_debt_service);

    CreditMetrics {
        dti_bps,
        dsti_bps,
        lti_bps,
        disposable_income,
    }
}


// Unit tests for the credit metrics calculation

#[cfg(test)]
mod tests {
    use super::*;

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
}