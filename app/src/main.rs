use common::{calculate_metrics, CreditMetrics, FinancialData};
use eframe::egui;
use host::generate_proof;
use std::fs;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Instant;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([980.0, 760.0]),
        ..Default::default()
    };

    eframe::run_native(
        "ZK Credit Score",
        options,
        Box::new(|_cc| Ok(Box::new(CreditScoreApp::default()))),
    )
}

struct ProofUiResult {
    score: u64,
    eligible: bool,
    input_hash: String,
    proving_time_secs: f64,
    receipt_size_kb: Option<f64>,
    metrics: CreditMetrics,
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
    verification_time: Option<f64>,
    receipt_size: Option<f64>,
    metrics: Option<CreditMetrics>,
    proof_generated: bool,
    verification_status: Option<bool>,
    verification_error: Option<String>,
    error: Option<String>,
    is_generating: bool,
    proof_receiver: Option<Receiver<Result<ProofUiResult, String>>>,
}

impl eframe::App for CreditScoreApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_proof_result(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(color::BACKGROUND))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.add_space(10.0);

                        self.render_header(ui);
                        ui.add_space(16.0);

                        ui.horizontal_wrapped(|ui| {
                            ui.set_width(ui.available_width());
                            ui.vertical(|ui| {
                                ui.set_min_width(420.0);
                                self.render_input_panel(ui, ctx);
                            });

                            ui.add_space(12.0);

                            ui.vertical(|ui| {
                                ui.set_min_width(360.0);
                                self.render_result_panel(ui);
                                ui.add_space(12.0);
                                self.render_metrics_panel(ui);
                            });
                        });

                        ui.add_space(12.0);
                        self.render_proof_panel(ui);
                        ui.add_space(16.0);
                    });
            });
    }
}

impl CreditScoreApp {
    fn render_header(&self, ui: &mut egui::Ui) {
        panel(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("ZK Credit Score")
                            .size(30.0)
                            .strong()
                            .color(color::TEXT),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("Verifiable credit scoring powered by RISC Zero zkVM")
                            .size(14.0)
                            .color(color::MUTED),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    badge(ui, "Zero-Knowledge Verified");
                    badge(ui, "Powered by RISC Zero");
                });
            });
        });
    }

    fn render_input_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        panel(ui, |ui| {
            section_title(ui, "Financial Data", "Private input sent to the zkVM guest");

            ui.columns(2, |columns| {
                field(
                    &mut columns[0],
                    "Income",
                    "EUR / month",
                    &mut self.monthly_income,
                );
                field(
                    &mut columns[1],
                    "Expenses",
                    "EUR / month",
                    &mut self.monthly_expenses,
                );
                field(&mut columns[0], "Total Debt", "EUR", &mut self.total_debt);
                field(
                    &mut columns[1],
                    "Debt Service",
                    "EUR / month",
                    &mut self.monthly_debt_service,
                );
                field(
                    &mut columns[0],
                    "Requested Loan",
                    "EUR",
                    &mut self.requested_loan,
                );
                field(
                    &mut columns[1],
                    "Duration",
                    "months",
                    &mut self.loan_duration_months,
                );
                field(
                    &mut columns[0],
                    "Interest Rate",
                    "%",
                    &mut self.interest_rate,
                );
                field(&mut columns[1], "Dependents", "count", &mut self.dependents);
                field(&mut columns[0], "Age", "years", &mut self.age);
            });

            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled(!self.is_generating, egui::Button::new("Load demo data"))
                    .clicked()
                {
                    self.load_demo_data();
                }

                let primary = egui::Button::new(
                    egui::RichText::new(if self.is_generating {
                        "Generating proof..."
                    } else {
                        "Generate ZK Proof"
                    })
                    .strong(),
                )
                .fill(color::ACCENT);

                if ui.add_enabled(!self.is_generating, primary).clicked() {
                    self.start_calculation(ctx);
                }
            });

            if self.is_generating {
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("Generating Zero-Knowledge Proof...")
                        .strong()
                        .color(color::ACCENT),
                );
                ui.add(
                    egui::ProgressBar::new(0.45)
                        .animate(true)
                        .desired_width(ui.available_width())
                        .text("This can take a few seconds"),
                );
            }

            if let Some(error) = &self.error {
                ui.add_space(12.0);
                error_box(ui, error);
            }
        });
    }

    fn render_result_panel(&self, ui: &mut egui::Ui) {
        panel(ui, |ui| {
            section_title(ui, "Credit Score", "Result committed in the journal");

            match self.score {
                Some(score) => {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(score.to_string())
                                .size(48.0)
                                .strong()
                                .color(color::TEXT),
                        );
                        ui.label(egui::RichText::new("/ 100").size(22.0).color(color::MUTED));
                    });

                    ui.add(
                        egui::ProgressBar::new(score as f32 / 100.0)
                            .desired_width(ui.available_width())
                            .text(format!("{score}%")),
                    );

                    ui.add_space(10.0);
                    if self.eligible == Some(true) {
                        status_pill(ui, "Eligible", color::SUCCESS);
                    } else {
                        status_pill(ui, "Not Eligible", color::DANGER);
                    }
                }
                None => {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("No proof generated yet").color(color::MUTED));
                    ui.add_space(34.0);
                }
            }
        });
    }

    fn render_metrics_panel(&self, ui: &mut egui::Ui) {
        panel(ui, |ui| {
            section_title(
                ui,
                "Metrics",
                "Computed by the same scoring logic used by the guest",
            );

            if let Some(metrics) = &self.metrics {
                ui.columns(2, |columns| {
                    metric_card(&mut columns[0], "DTI", format_bps(metrics.dti_bps));
                    metric_card(&mut columns[1], "DSTI", format_bps(metrics.dsti_bps));
                    metric_card(&mut columns[0], "LTI", format_bps(metrics.lti_bps));
                    metric_card(
                        &mut columns[1],
                        "Disposable Income",
                        format!("EUR {}", metrics.disposable_income),
                    );
                });
            } else {
                ui.label(
                    egui::RichText::new("Metrics will appear after proof generation")
                        .color(color::MUTED),
                );
            }
        });
    }

    fn render_proof_panel(&mut self, ui: &mut egui::Ui) {
        panel(ui, |ui| {
            section_title(
                ui,
                "Zero-Knowledge Proof",
                "Receipt generation and cryptographic verification",
            );

            ui.columns(3, |columns| {
                proof_status(
                    &mut columns[0],
                    "Proof generated",
                    self.proof_generated,
                    self.is_generating,
                );
                proof_status(
                    &mut columns[1],
                    "Receipt verified",
                    self.verification_status == Some(true),
                    false,
                );
                proof_status(
                    &mut columns[2],
                    "Verification successful",
                    self.verification_status == Some(true),
                    false,
                );
            });

            ui.add_space(12.0);

            ui.columns(3, |columns| {
                detail(
                    &mut columns[0],
                    "Proving time",
                    optional_secs(self.proving_time),
                );
                detail(
                    &mut columns[1],
                    "Verification time",
                    optional_secs(self.verification_time),
                );
                detail(
                    &mut columns[2],
                    "Receipt size",
                    optional_kb(self.receipt_size),
                );
            });

            if let Some(hash) = &self.input_hash {
                ui.add_space(12.0);
                ui.label(egui::RichText::new("Input SHA-256").color(color::MUTED));
                let mut hash_text = hash.clone();
                ui.add(
                    egui::TextEdit::singleline(&mut hash_text)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(ui.available_width())
                        .interactive(false),
                );
            }

            ui.add_space(12.0);
            if ui
                .add_enabled(
                    self.proof_generated && !self.is_generating,
                    egui::Button::new("Verify Receipt"),
                )
                .clicked()
            {
                self.verify_receipt();
            }

            if let Some(status) = self.verification_status {
                ui.add_space(8.0);
                if status {
                    status_pill(ui, "Receipt verification: VALID", color::SUCCESS);
                } else {
                    status_pill(ui, "Receipt verification: INVALID", color::DANGER);
                }
            }

            if let Some(error) = &self.verification_error {
                ui.add_space(8.0);
                error_box(ui, error);
            }
        });
    }

    fn poll_proof_result(&mut self, ctx: &egui::Context) {
        if let Some(receiver) = &self.proof_receiver {
            match receiver.try_recv() {
                Ok(Ok(result)) => {
                    self.score = Some(result.score);
                    self.eligible = Some(result.eligible);
                    self.input_hash = Some(result.input_hash);
                    self.proving_time = Some(result.proving_time_secs);
                    self.receipt_size = result.receipt_size_kb;
                    self.metrics = Some(result.metrics);
                    self.proof_generated = true;
                    self.is_generating = false;
                    self.proof_receiver = None;
                }
                Ok(Err(error)) => {
                    self.error = Some(error);
                    self.is_generating = false;
                    self.proof_receiver = None;
                }
                Err(TryRecvError::Empty) => {
                    ctx.request_repaint();
                }
                Err(TryRecvError::Disconnected) => {
                    self.error = Some("Proof generation worker stopped unexpectedly".to_string());
                    self.is_generating = false;
                    self.proof_receiver = None;
                }
            }
        }
    }

    fn load_demo_data(&mut self) {
        self.monthly_income = "3500".to_string();
        self.monthly_expenses = "1500".to_string();
        self.total_debt = "8000".to_string();
        self.monthly_debt_service = "450".to_string();
        self.requested_loan = "10000".to_string();
        self.loan_duration_months = "36".to_string();
        self.interest_rate = "3.50".to_string();
        self.dependents = "0".to_string();
        self.age = "30".to_string();
    }

    fn start_calculation(&mut self, ctx: &egui::Context) {
        self.reset_result_state();

        let data = match self.parse_financial_data() {
            Ok(data) => data,
            Err(error) => {
                self.error = Some(error);
                return;
            }
        };

        let (sender, receiver) = mpsc::channel();
        let repaint_ctx = ctx.clone();

        self.is_generating = true;
        self.proof_receiver = Some(receiver);

        std::thread::spawn(move || {
            let metrics = calculate_metrics(&data);
            let result = generate_proof(data).map(|proof| ProofUiResult {
                score: proof.assessment.credit_score,
                eligible: proof.assessment.eligible,
                input_hash: hex::encode(proof.assessment.input_hash),
                proving_time_secs: proof.proving_time.as_secs_f64(),
                receipt_size_kb: fs::metadata("receipt.json")
                    .ok()
                    .map(|metadata| metadata.len() as f64 / 1024.0),
                metrics,
            });

            let _ = sender.send(result);
            repaint_ctx.request_repaint();
        });
    }

    fn reset_result_state(&mut self) {
        self.error = None;
        self.score = None;
        self.eligible = None;
        self.input_hash = None;
        self.proving_time = None;
        self.verification_time = None;
        self.receipt_size = None;
        self.metrics = None;
        self.proof_generated = false;
        self.verification_status = None;
        self.verification_error = None;
    }

    fn parse_financial_data(&self) -> Result<FinancialData, String> {
        Ok(FinancialData {
            monthly_income: parse_u64(&self.monthly_income, "Income")?,
            monthly_expenses: parse_u64(&self.monthly_expenses, "Expenses")?,
            total_debt: parse_u64(&self.total_debt, "Total debt")?,
            monthly_debt_service: parse_u64(&self.monthly_debt_service, "Debt service")?,
            requested_loan: parse_u64(&self.requested_loan, "Requested loan")?,
            loan_duration_months: parse_u32(&self.loan_duration_months, "Duration")?,
            interest_rate_bps: {
                let rate = parse_f64(&self.interest_rate, "Interest rate")?;

                if !rate.is_finite() || rate < 0.0 {
                    return Err("Interest rate must be a non-negative finite number".to_string());
                }

                (rate * 100.0) as u64
            },
            dependents: parse_u32(&self.dependents, "Dependents")?,
            age_years: parse_u32(&self.age, "Age")?,
        })
    }

    fn verify_receipt(&mut self) {
        self.verification_status = None;
        self.verification_error = None;
        self.verification_time = None;

        let start = Instant::now();

        match host::load_receipt() {
            Ok(receipt) => match host::verify_receipt(&receipt) {
                Ok(()) => {
                    self.verification_time = Some(start.elapsed().as_secs_f64());
                    self.verification_status = Some(true);
                }

                Err(error) => {
                    self.verification_time = Some(start.elapsed().as_secs_f64());
                    self.verification_status = Some(false);
                    self.verification_error = Some(error);
                }
            },

            Err(error) => {
                self.verification_status = Some(false);
                self.verification_error = Some(error);
            }
        }
    }
}

fn panel(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::group(ui.style())
        .fill(color::PANEL)
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add_contents(ui);
        });
}

fn section_title(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    ui.label(
        egui::RichText::new(title)
            .size(18.0)
            .strong()
            .color(color::TEXT),
    );
    ui.label(egui::RichText::new(subtitle).size(12.0).color(color::MUTED));
    ui.add_space(12.0);
}

fn field(ui: &mut egui::Ui, label: &str, unit: &str, value: &mut String) {
    ui.add_space(6.0);
    ui.label(egui::RichText::new(label).strong().color(color::TEXT));
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(value)
                .desired_width(130.0)
                .hint_text("0"),
        );
        ui.label(egui::RichText::new(unit).size(12.0).color(color::MUTED));
    });
}

fn badge(ui: &mut egui::Ui, text: &str) {
    egui::Frame::default()
        .fill(color::BADGE)
        .inner_margin(egui::Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).size(12.0).color(color::TEXT));
        });
}

fn metric_card(ui: &mut egui::Ui, label: &str, value: String) {
    egui::Frame::group(ui.style())
        .fill(color::CARD)
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.set_min_width(130.0);
            ui.label(egui::RichText::new(label).size(12.0).color(color::MUTED));
            ui.label(
                egui::RichText::new(value)
                    .size(18.0)
                    .strong()
                    .color(color::TEXT),
            );
        });
    ui.add_space(8.0);
}

fn proof_status(ui: &mut egui::Ui, label: &str, complete: bool, loading: bool) {
    let (text, color) = if complete {
        ("OK", color::SUCCESS)
    } else if loading {
        ("...", color::ACCENT)
    } else {
        ("Pending", color::MUTED)
    };

    ui.label(egui::RichText::new(label).size(12.0).color(color::MUTED));
    ui.label(egui::RichText::new(text).strong().color(color));
}

fn detail(ui: &mut egui::Ui, label: &str, value: String) {
    ui.label(egui::RichText::new(label).size(12.0).color(color::MUTED));
    ui.label(egui::RichText::new(value).strong().color(color::TEXT));
}

fn status_pill(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    egui::Frame::default()
        .fill(color.linear_multiply(0.18))
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).strong().color(color));
        });
}

fn error_box(ui: &mut egui::Ui, error: &str) {
    egui::Frame::group(ui.style())
        .fill(color::DANGER.linear_multiply(0.12))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(error).color(color::DANGER));
        });
}

fn format_bps(value: u64) -> String {
    format!("{:.2}%", value as f64 / 100.0)
}

fn optional_secs(value: Option<f64>) -> String {
    value
        .map(|seconds| format!("{seconds:.3} s"))
        .unwrap_or_else(|| "-".to_string())
}

fn optional_kb(value: Option<f64>) -> String {
    value
        .map(|kb| format!("{kb:.2} KB"))
        .unwrap_or_else(|| "-".to_string())
}

fn parse_u64(value: &str, field: &str) -> Result<u64, String> {
    value
        .trim()
        .parse::<u64>()
        .map_err(|_| format!("Invalid value for {field}"))
}

fn parse_u32(value: &str, field: &str) -> Result<u32, String> {
    value
        .trim()
        .parse::<u32>()
        .map_err(|_| format!("Invalid value for {field}"))
}

fn parse_f64(value: &str, field: &str) -> Result<f64, String> {
    value
        .trim()
        .replace(',', ".")
        .parse::<f64>()
        .map_err(|_| format!("Invalid value for {field}"))
}

mod color {
    use eframe::egui::Color32;

    pub const BACKGROUND: Color32 = Color32::from_rgb(12, 18, 24);
    pub const PANEL: Color32 = Color32::from_rgb(20, 29, 38);
    pub const CARD: Color32 = Color32::from_rgb(25, 37, 48);
    pub const BADGE: Color32 = Color32::from_rgb(32, 52, 62);
    pub const TEXT: Color32 = Color32::from_rgb(232, 238, 242);
    pub const MUTED: Color32 = Color32::from_rgb(146, 160, 172);
    pub const ACCENT: Color32 = Color32::from_rgb(28, 145, 135);
    pub const SUCCESS: Color32 = Color32::from_rgb(62, 201, 133);
    pub const DANGER: Color32 = Color32::from_rgb(237, 91, 98);
}
