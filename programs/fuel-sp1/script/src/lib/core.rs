use alloy_sol_types::SolType;
use fuel_zkvm_primitives_prover::{Input, PublicValuesStruct};
use fuel_zkvm_primitives_test_fixtures::{
    opcodes::start_node_with_transaction_and_produce_prover_input, Fixture,
};
use sp1_sdk::{ExecutionReport, ProverClient, SP1Stdin};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const FUEL_SP1_ELF: &[u8] = include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

pub async fn run_fixture(fixture: Fixture, stdin: &mut SP1Stdin) -> [u8; 32] {
    let block_id: [u8; 32];

    match fixture {
        Fixture::MainnetBlock(block) => {
            tracing::info!("Mainnet block: {:?}", block);
            let raw_input =
                fuel_zkvm_primitives_test_fixtures::mainnet_blocks::get_mainnet_block_input(block);
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

    block_id
}

pub async fn execute_program(
    fixture: Fixture,
    client: &ProverClient,
    mut stdin: SP1Stdin,
) -> ExecutionReport {
    let block_id = run_fixture(fixture, &mut stdin).await;

    // Execute the program
    let (output, report) = client.execute(FUEL_SP1_ELF, stdin).run().unwrap();
    tracing::info!("Program executed successfully.");

    let output = PublicValuesStruct::abi_decode(output.as_slice(), true).unwrap();

    assert_eq!(output.block_id.to_be_bytes(), block_id);

    report
}

pub async fn prove_and_verify_program(
    fixture: Fixture,
    client: &ProverClient,
    mut stdin: SP1Stdin,
) {
    let _ = run_fixture(fixture, &mut stdin).await;

    // Setup the program for proving.
    let (pk, vk) = client.setup(FUEL_SP1_ELF);

    // Generate the proof
    let proof = client.prove(&pk, stdin).run().expect("failed to generate proof");

    // Verify the proof
    client.verify(&proof, &vk).expect("failed to verify proof");
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv::Writer;
    use fuel_zkvm_primitives_test_fixtures::all_fixtures;
    use serde::Serialize;

    #[derive(Serialize)]
    struct ExecutionReport {
        fixture: Fixture,
        cycle_count: u64,
        memory_address_count: u64,
        syscall_count: u64,
    }

    #[tokio::test]
    async fn run_all_fixtures_and_collect_report() {
        let fixtures = all_fixtures();

        let file_path =
            std::env::var("FUEL_SP1_REPORT").unwrap_or("fuel_sp1_report.csv".to_string());
        let mut wtr = Writer::from_path(file_path).expect("Couldn't create CSV writer");

        for fixture in fixtures {
            let stdin = SP1Stdin::new();
            let report = execute_program(fixture.clone(), &ProverClient::new(), stdin).await;

            let perf_report = ExecutionReport {
                fixture: fixture.clone(),
                cycle_count: report.total_instruction_count(),
                memory_address_count: report.touched_memory_addresses,
                syscall_count: report.total_syscall_count(),
            };

            wtr.serialize(perf_report).expect("Couldn't write to CSV");

            // flush after each execution
            wtr.flush().expect("Couldn't flush CSV writer");

            tracing::info!("Executed fixture: {:?}", fixture);
        }
    }
}
