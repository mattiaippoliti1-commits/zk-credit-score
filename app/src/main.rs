// use common::{calculate_credit_score, calculate_metrics, FinancialData};
use common::FinancialData;
use host::generate_proof;
use eframe::egui;
use std::fs;

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
    input_hash: Option<String>,
    proving_time: Option<f64>,
    receipt_size: Option<f64>,
    proof_generated: bool,
    verification_status: Option<bool>,
    verification_error: Option<String>,
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

            if self.proof_generated {
                        ui.separator();

                        ui.heading("Zero-Knowledge Proof");

                        if let Some(hash) = &self.input_hash {
                            ui.label("Input SHA-256:");

                            ui.monospace(hash);
                        }

                        if let Some(time) = self.proving_time {
                            ui.label(format!(
                                "Proving time: {:.3} seconds",
                                time
                            ));
                        }

                        if let Some(size) = self.receipt_size {
                            ui.label(format!(
                                "Receipt size: {:.2} KB",
                                size
                            ));
                        }

                        ui.separator();

                        if ui.button("Verify Receipt").clicked() {
                            self.verify_receipt();
                        }

                        if let Some(status) = self.verification_status {
                            if status {
                                ui.colored_label(
                                    egui::Color32::GREEN,
                                    "Receipt verification: VALID",
                                );
                            } else {
                                ui.colored_label(
                                    egui::Color32::RED,
                                    "Receipt verification: INVALID",
                                );
                            }
                        }

                        if let Some(error) = &self.verification_error {
                            ui.colored_label(
                                egui::Color32::RED,
                                error,
                            );
                        }

                        ui.label("Proof status: GENERATED");
                    }
        });
    }
}

impl CreditScoreApp {
    fn calculate(&mut self) {
    self.error = None;
    self.score = None;
    self.eligible = None;
    self.input_hash = None;
    self.proving_time = None;
    self.receipt_size = None;
    self.proof_generated = false;
    self.verification_status = None;
    self.verification_error = None; 

    let result = self.parse_financial_data();

    match result {
        Ok(data) => {
            let result = generate_proof(data);

            self.score = Some(result.assessment.credit_score);
            self.eligible = Some(result.assessment.eligible);

            self.input_hash = Some(
                hex::encode(result.assessment.input_hash)
            );

            self.proving_time =
                Some(result.proving_time.as_secs_f64());

            self.receipt_size = fs::metadata("receipt.json")
                .ok()
                .map(|metadata| metadata.len() as f64 / 1024.0);

            self.proof_generated = true;
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

    fn verify_receipt(&mut self) {
    self.verification_status = None;
    self.verification_error = None;

    match host::load_receipt() {
        Ok(receipt) => {
            match host::verify_receipt(&receipt) {
                Ok(()) => {
                    self.verification_status = Some(true);
                }

                Err(error) => {
                    self.verification_status = Some(false);
                    self.verification_error = Some(error);
                }
            }
        }

        Err(error) => {
            self.verification_status = Some(false);
            self.verification_error = Some(error);
        }
    }
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