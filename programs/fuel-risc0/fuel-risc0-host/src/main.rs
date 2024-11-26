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
use alloy_sol_types::SolType;
use clap::Parser;
use fuel_risc0_methods::{FUEL_RISC0_PROVER_ELF, FUEL_RISC0_PROVER_ID};
use fuel_zkvm_primitives_prover::{Input, PublicValuesStruct};
use fuel_zkvm_primitives_test_fixtures::{
    opcodes::start_node_with_transaction_and_produce_prover_input, Fixture,
};
use risc0_zkvm::{default_prover, ExecutorEnv};

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

    let mut env = ExecutorEnv::builder();

    let block_id: [u8; 32];

    match args.fixture {
        Fixture::MainnetBlock(block) => {
            let raw_input =
                fuel_zkvm_primitives_test_fixtures::mainnet_blocks::get_mainnet_block_input(block);
            let input: Input = bincode::deserialize(&raw_input).unwrap();

            block_id = input.block.header().id().into();
            env.write(&input).expect("Failed to write input to environment");
        }
        Fixture::Opcode(instruction) => {
            let service =
                start_node_with_transaction_and_produce_prover_input(instruction).await.unwrap();

            block_id = service.input.block.header().id().into();

            let input: Vec<u8> =
                bincode::serialize(&service.input).expect("Failed to serialize service input");

            env.write(&input).expect("Failed to write input to environment");
        }
    }

    let prover = default_prover();
    let prove_info = prover.prove(env.build().unwrap(), FUEL_RISC0_PROVER_ELF).unwrap();
    let output: Vec<u8> = prove_info.receipt.journal.decode().unwrap();

    let decoded_output = PublicValuesStruct::abi_decode(&output, true).unwrap();

    assert_eq!(decoded_output.block_id.to_be_bytes(), block_id);

    println!("Proof block id: {:?}", decoded_output.block_id);
    println!("Proof input hash: {:?}", decoded_output.input_hash);

    prove_info.receipt.verify(FUEL_RISC0_PROVER_ID).expect("Proof verification failed.");

    println!("Successfully verified proof!");
}
