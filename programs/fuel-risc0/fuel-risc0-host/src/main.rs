//! An end-to-end example of using the RISC Zero ZKVM to generate and verify a proof of execution
//! of the FuelVM.
//!
//! This program starts a node with a transaction, serializes the input, and passes it to the ZKVM.
//! It then verifies the generated proof to ensure correctness.
//!
//! You can run this script using the following command:
//! ```shell
//! RISC0_DEV_MODE=1 RUST_LOG=info cargo run --release -- --help
//! ```
//!
//! The `RISC0_DEV_MODE=1` flag enables development mode, and `RUST_LOG=info` configures logging
//! for better visibility.
use clap::Parser;
use fuel_risc0_host::prove_program;
use fuel_zkvm_primitives_test_fixtures::Fixture;
use risc0_zkvm::ExecutorEnv;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[arg(value_enum)]
    fixture: Fixture,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let env = ExecutorEnv::builder();

    let output = prove_program(args.fixture, env).await;

    tracing::info!("Proof block id: {:?}", output.block_id);
    tracing::info!("Proof input hash: {:?}", output.input_hash);
}
