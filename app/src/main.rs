use common::{calculate_credit_score, calculate_metrics, FinancialData};
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "ZK Credit Score",
        options,
        Box::new(|_cc| Ok(Box::new(CreditScoreApp::default()))),
    )
}

#[derive(Default)]
struct CreditScoreApp {
    monthly_income: String,
    monthly_expenses: String,
    total_debt: String,
    monthly_debt_service: String,
    requested_loan: String,
    loan_duration_months: String,
    interest_rate: String,
    dependents: String,
    age: String,

    score: Option<u64>,
    eligible: Option<bool>,
    error: Option<String>,
}

impl eframe::App for CreditScoreApp {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("ZK Credit Score");
            ui.separator();

            ui.heading("Financial Profile");

            ui.horizontal(|ui| {
                ui.label("Monthly income:");
                ui.text_edit_singleline(&mut self.monthly_income);
            });

            ui.horizontal(|ui| {
                ui.label("Monthly expenses:");
                ui.text_edit_singleline(&mut self.monthly_expenses);
            });

            ui.horizontal(|ui| {
                ui.label("Total debt:");
                ui.text_edit_singleline(&mut self.total_debt);
            });

            ui.horizontal(|ui| {
                ui.label("Monthly debt service:");
                ui.text_edit_singleline(&mut self.monthly_debt_service);
            });

            ui.horizontal(|ui| {
                ui.label("Requested loan:");
                ui.text_edit_singleline(&mut self.requested_loan);
            });

            ui.horizontal(|ui| {
                ui.label("Loan duration (months):");
                ui.text_edit_singleline(&mut self.loan_duration_months);
            });

            ui.horizontal(|ui| {
                ui.label("Interest rate (%):");
                ui.text_edit_singleline(&mut self.interest_rate);
            });

            ui.horizontal(|ui| {
                ui.label("Dependents:");
                ui.text_edit_singleline(&mut self.dependents);
            });

            ui.horizontal(|ui| {
                ui.label("Age:");
                ui.text_edit_singleline(&mut self.age);
            });

            ui.separator();

            if ui.button("Calculate Creditworthiness").clicked() {
                self.calculate();
            }

            if let Some(error) = &self.error {
                ui.separator();
                ui.colored_label(egui::Color32::RED, error);
            }

            if let Some(score) = self.score {
                ui.separator();
                ui.heading("Creditworthiness Assessment");

                ui.label(format!("Credit Score: {}/100", score));

                if let Some(eligible) = self.eligible {
                    if eligible {
                        ui.colored_label(
                            egui::Color32::GREEN,
                            "ELIGIBLE",
                        );
                    } else {
                        ui.colored_label(
                            egui::Color32::RED,
                            "NOT ELIGIBLE",
                        );
                    }
                }
            }
        });
    }
}

impl CreditScoreApp {
    fn calculate(&mut self) {
        self.error = None;
        self.score = None;
        self.eligible = None;

        let result = self.parse_financial_data();

        match result {
            Ok(data) => {
                let metrics = calculate_metrics(&data);
                let credit_score =
                    calculate_credit_score(&data, &metrics);

                self.score = Some(credit_score.score);
                self.eligible = Some(credit_score.eligible);
            }

            Err(error) => {
                self.error = Some(error);
            }
        }
    }

    fn parse_financial_data(&self) -> Result<FinancialData, String> {
        Ok(FinancialData {
            monthly_income: parse_u64(
                &self.monthly_income,
                "Monthly income",
            )?,

            monthly_expenses: parse_u64(
                &self.monthly_expenses,
                "Monthly expenses",
            )?,

            total_debt: parse_u64(
                &self.total_debt,
                "Total debt",
            )?,

            monthly_debt_service: parse_u64(
                &self.monthly_debt_service,
                "Monthly debt service",
            )?,

            requested_loan: parse_u64(
                &self.requested_loan,
                "Requested loan",
            )?,

            loan_duration_months: parse_u32(
                &self.loan_duration_months,
                "Loan duration",
            )?,

            interest_rate_bps: {
                let rate = parse_f64(
                    &self.interest_rate,
                    "Interest rate",
                )?;

                (rate * 100.0) as u64
            },

            dependents: parse_u32(
                &self.dependents,
                "Dependents",
            )?,

            age_years: parse_u32(
                &self.age,
                "Age",
            )?,
        })
    }
}

fn parse_u64(value: &str, field: &str) -> Result<u64, String> {
    value
        .trim()
        .parse::<u64>()
        .map_err(|_| format!("Invalid value for {}", field))
}

fn parse_u32(value: &str, field: &str) -> Result<u32, String> {
    value
        .trim()
        .parse::<u32>()
        .map_err(|_| format!("Invalid value for {}", field))
}

fn parse_f64(value: &str, field: &str) -> Result<f64, String> {
    value
        .trim()
        .replace(',', ".")
        .parse::<f64>()
        .map_err(|_| format!("Invalid value for {}", field))
}