//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- execute add
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- prove add
//! ```

use clap::{Parser, Subcommand};
use fuel_script::core::{execute_program, prove_and_verify_program};
use fuel_zkvm_primitives_test_fixtures::Fixture;
use sp1_sdk::{ProverClient, SP1Stdin};

/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
#[clap(name = "command", about = "The command to execute", rename_all = "snake_case")]
enum Command {
    Execute {
        #[arg(value_enum)]
        fixture: Fixture,
    },
    Prove {
        #[arg(value_enum)]
        fixture: Fixture,
    },
}

#[tokio::main]
async fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Parse the command line arguments.
    let args = Args::parse();

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the inputs.
    let stdin = SP1Stdin::new();

    match args.command {
        Command::Execute { fixture } => {
            tracing::info!("Executing the program.");
            // Execute the program.
            let report = execute_program(fixture, &client, stdin).await;
            tracing::info!("Program executed successfully.");

            // Record the number of cycles executed.
            tracing::info!("Number of cycles: {}", report.total_instruction_count());
        }
        Command::Prove { fixture } => {
            tracing::info!("Proving and verifying the program.");
            // Generate and verify the proof.
            prove_and_verify_program(fixture, &client, stdin).await;
            tracing::info!("Successfully generated and verified proof!");
        }
    }
}
