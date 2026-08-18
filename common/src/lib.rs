use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
// Define a struct to hold the financial data
pub struct FinancialData {
    pub monthly_income: u64,
    pub monthly_expenses: u64,
    pub total_debt: u64,
    pub monthly_debt_service: u64,
    pub requested_loan: u64,
    pub loan_duration_months: u32,
}