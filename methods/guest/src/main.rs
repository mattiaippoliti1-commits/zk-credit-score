// DEFAULT MAIN

// use risc0_zkvm::guest::env;

// fn main() {
//     // TODO: Implement your guest code here

//     // read the input
//     let input: u32 = env::read();

//     // TODO: do something with the input

//     // write public output to the journal
//     env::commit(&input);
// }


use common::{calculate_metrics, FinancialData};
use risc0_zkvm::guest::env;

fn main() {
    // Read the financial data provided by the host.
    let data: FinancialData = env::read();

    // Calculate the creditworthiness metrics inside the zkVM.
    let metrics = calculate_metrics(&data);

    // Commit the computed metrics to the journal.
    env::commit(&metrics);
}