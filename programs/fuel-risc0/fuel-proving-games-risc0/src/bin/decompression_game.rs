//! An end-to-end example of using the RISC Zero ZKVM to generate and verify a proof of
//! decompression
//!
//! This program serializes the input blobs, and then verifies the decompression for correctness.
//!
//! You can run this script using the following command:
//! ```shell
//! RISC0_DEV_MODE=1 RUST_LOG=info cargo run --release --bin fuel-decompression-game-risc0 -- --help
//! ```
//!
//! The `RISC0_DEV_MODE=1` flag enables development mode, and `RUST_LOG=info` configures logging
//! for better visibility.
use clap::{Parser, Subcommand};
use fuel_proving_games_risc0::decompression_game::{execute_fixture, prove_fixture};
use fuel_zkvm_primitives_test_fixtures::decompression_fixtures::Fixture;
use risc0_zkvm::ExecutorEnv;

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
    ExecuteFixture {
        #[arg(value_enum)]
        fixture: Fixture,
    },
    ProveFixture {
        #[arg(value_enum)]
        fixture: Fixture,
    },
}

fn main() -> fuel_proving_games_risc0::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let env = ExecutorEnv::builder();

    match args.command {
        Command::ExecuteFixture { fixture } => {
            tracing::info!("Executing the fixture");
            let report = execute_fixture(fixture, env)?;
            tracing::info!("fixture executed successfully.");

            // Record the number of cycles executed.
            tracing::info!("Number of cycles: {}", report.cycles());
        }
        Command::ProveFixture { fixture } => {
            tracing::info!("Proving and verifying the fixture");
            let prove_info = prove_fixture(fixture, env)?;
            prove_info
                .receipt
                .verify(fuel_proving_games_risc0::FUEL_DECOMPRESSION_GAME_RISC0_ID)
                .map_err(|e| fuel_proving_games_risc0::Error::FailedToVerifyProof(e.to_string()))?;
            tracing::info!("Fixture proved and verified successfully.");
        }
    }

    Ok(())
}
