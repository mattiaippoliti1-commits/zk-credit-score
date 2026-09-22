# zk-credit-score

## Overview

`zk-credit-score` is an academic demo of verifiable credit scoring with the
RISC Zero zkVM. A host program provides private financial data to a zkVM guest;
the guest computes credit metrics and a final score; RISC Zero produces a
receipt proving that the expected guest program ran and committed the published
result.

The scoring model is intentionally simple. Its purpose is to demonstrate a
concrete zkVM workflow, not to implement a production banking model.

## Architecture

```text
FinancialData
  -> Host
  -> RISC Zero zkVM Guest
  -> Credit Scoring
  -> Journal
  -> Receipt
  -> Verification
```

- `common`: shared data structures, hashing, metric calculation, and scoring.
- `methods`: RISC Zero method build integration and generated guest image ID.
- `methods/guest`: guest program executed inside the zkVM.
- `host`: command-line prover/verifier and proof-generation helpers.
- `app`: small egui desktop GUI for the demo workflow.

## Private Inputs

The guest receives a `FinancialData` value containing:

- monthly income;
- monthly expenses;
- total debt;
- monthly debt service;
- requested loan;
- loan duration in months;
- interest rate in basis points;
- number of dependents;
- age in years.

These raw values are private inputs to the zkVM execution and are not directly
written to the journal.

## Public Outputs

The journal commits only:

- `credit_score`;
- `eligible`;
- `input_hash`.

`input_hash` is a deterministic SHA-256 digest of the `FinancialData` fields in
a fixed order. It is useful as a public identifier for the exact input used in a
proof, but it is not a complete privacy mechanism by itself.

## Credit Score

The guest computes:

- DTI: total debt divided by annual income;
- DSTI: monthly debt service divided by monthly income;
- LTI: requested loan divided by annual income;
- Disposable Income: monthly income minus expenses and existing debt service.

Each metric contributes to a simple score from 0 to 100. The final score is the
average of the component scores, and the applicant is eligible when the score is
at least 60.

## Zero-Knowledge Verification

The receipt proves that the RISC Zero guest image identified by `METHOD_ID`
executed successfully and produced the committed journal. A verifier can check
the receipt and trust that the published `credit_score`, `eligible`, and
`input_hash` came from the expected guest program, without receiving the raw
financial inputs.

The verifier still learns the public outputs. The score, eligibility decision,
and deterministic input hash can leak information about the private inputs.

## Tampering Detection

The journal is authenticated by the receipt. If a byte of the journal is changed
after proving, receipt verification fails. The host tests and the manual
tampering flow both check this property.

## Running the Project

Build/check the workspace:

```bash
cargo check --workspace
```

Run tests:

```bash
cargo test --workspace
```

Generate a real receipt:

```bash
cargo run --bin prover
```

Verify the saved receipt:

```bash
cargo run --bin verifier
```

Check the GUI crate:

```bash
cargo check -p app
```

## Example

The demo input currently used by the CLI has:

- monthly income: `3500`;
- monthly expenses: `1500`;
- total debt: `8000`;
- monthly debt service: `450`;
- requested loan: `10000`;
- loan duration: `36` months;
- interest rate: `350` basis points;
- dependents: `0`;
- age: `30`.

For this input, the verified result is:

```text
Credit score: 95/100
Eligible: true
```

## Security / Privacy Notes

The raw financial inputs are not published in the journal, but the journal does
publish a deterministic SHA-256 digest of them. This digest lets a party compare
or later recognize the exact same input, and it can support dictionary attacks
when input values are drawn from a small or predictable space.

For a stronger privacy-oriented protocol, the input digest could include a
salt/nonce or be replaced with a more explicit commitment scheme. This project
keeps the deterministic hash because it is simple and sufficient for the
academic demo: proving correct execution of a credit-scoring computation inside
a zkVM while keeping raw inputs out of the public journal.

## Academic Purpose

This repository is a university cryptography project. It demonstrates RISC Zero
zkVM proving, receipt verification, journal authentication, and careful handling
of public versus private data. It is not intended for real credit decisions.
