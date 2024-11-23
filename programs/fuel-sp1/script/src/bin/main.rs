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

use alloy_sol_types::SolType;
use clap::{Parser, Subcommand};
use fuel_zkvm_primitives_prover::{Input, PublicValuesStruct};
use fuel_zkvm_primitives_test_fixtures::{
    opcodes::start_node_with_transaction_and_produce_prover_input, Fixture,
};
use sp1_sdk::{ProverClient, SP1Stdin};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const FUEL_SP1_ELF: &[u8] = include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

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
    let mut stdin = SP1Stdin::new();

    match args.command {
        Command::Execute { fixture } => {
            tracing::info!("Executing the program.");

            let block_id: [u8; 32];
            match fixture {
                Fixture::MainnetBlock(block) => {
                    tracing::info!("Mainnet block: {:?}", block);
                    let raw_input =
                        fuel_zkvm_primitives_test_fixtures::mainnet_blocks::get_mainnet_block_input(
                            block,
                        );
                    let input: Input = bincode::deserialize(&raw_input).unwrap();

                    block_id = input.block.header().id().into();
                    stdin.write(&input);
                }
                Fixture::Opcode(opcode) => {
                    tracing::info!("Opcode args: {:?}", opcode);

                    let service =
                        start_node_with_transaction_and_produce_prover_input(opcode).await.unwrap();

                    block_id = service.input.block.header().id().into();
                    stdin.write(&service.input);
                }
            }

            // Execute the program
            let (output, report) = client.execute(FUEL_SP1_ELF, stdin).run().unwrap();
            tracing::info!("Program executed successfully.");

            // Read the output.
            let proof = PublicValuesStruct::abi_decode(output.as_slice(), true).unwrap();

            assert_eq!(proof.block_id.to_be_bytes(), block_id);

            tracing::info!("Proof block id: {:?}", proof.block_id);
            tracing::info!("Proof input hash: {:?}", proof.input_hash);

            // Record the number of cycles executed.
            tracing::info!("Number of cycles: {}", report.total_instruction_count());
        }
        Command::Prove { fixture } => {
            tracing::info!("Proving the program.");

            match fixture {
                Fixture::MainnetBlock(block) => {
                    tracing::info!("Mainnet block: {:?}", block);
                    let raw_input =
                        fuel_zkvm_primitives_test_fixtures::mainnet_blocks::get_mainnet_block_input(
                            block,
                        );
                    let input: Input = bincode::deserialize(&raw_input).unwrap();

                    stdin.write(&input);
                }
                Fixture::Opcode(opcode) => {
                    tracing::info!("Opcode args: {:?}", opcode);

                    let service =
                        start_node_with_transaction_and_produce_prover_input(opcode).await.unwrap();

                    stdin.write(&service.input);
                }
            }

            // Setup the program for proving.
            let (pk, vk) = client.setup(FUEL_SP1_ELF);

            // Generate the proof
            let proof = client.prove(&pk, stdin).run().expect("failed to generate proof");

            tracing::info!("Successfully generated proof!");

            // Verify the proof.
            client.verify(&proof, &vk).expect("failed to verify proof");
            tracing::info!("Successfully verified proof!");
        }
    }
}
