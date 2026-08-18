# 1. Generate valid receipt
cargo run --bin prover

# 2. Verify valid receipt
cargo run --bin verifier

# 3. Create test copies
cp receipt.json receipt-valid.json
cp receipt.json receipt-tampered.json

# 4. Tamper with the journal
python3 -c 'import json; f="receipt-tampered.json"; r=json.load(open(f)); print("Original first byte:", r["journal"]["bytes"][0]); r["journal"]["bytes"][0]=100; print("Modified first byte:", r["journal"]["bytes"][0]); json.dump(r, open(f,"w"))'

# 5. Replace the valid receipt with the tampered one
cp receipt.json receipt-original.json
cp receipt-tampered.json receipt.json

# 6. Verification must fail
cargo run --bin verifier

# 7. Restore valid receipt
mv receipt-original.json receipt.json

# 8. Verification must succeed again
cargo run --bin verifier

# 9. Cleanup
rm -f receipt-valid.json receipt-tampered.json