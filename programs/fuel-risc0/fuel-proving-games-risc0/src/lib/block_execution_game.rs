use crate::FUEL_BLOCK_EXECUTION_GAME_RISC0_ELF;
use alloy_sol_types::SolType;
use fuel_zkvm_primitives_prover::games::block_execution_game::{Input, PublicValuesStruct};
use fuel_zkvm_primitives_test_fixtures::block_execution_fixtures::fixtures::Fixture;
use risc0_zkvm::{ExecutorEnvBuilder, ProveInfo};

pub fn run_fixture(fixture: Fixture, env: &mut ExecutorEnvBuilder<'_>) -> crate::Result<[u8; 32]> {
    let raw_input = Fixture::get_input_for_fixture(&fixture);
    let input: Input =
        bincode::deserialize(&raw_input).map_err(crate::Error::FailedToDeserializeInput)?;
    let block_id = input.block.header().id().into();
    env.write(&raw_input)
        .map_err(|e| crate::Error::FailedToWriteInputToProverEnv(e.to_string()))?;

    Ok(block_id)
}

pub fn execute_fixture(
    fixture: Fixture,
    mut env: ExecutorEnvBuilder<'_>,
) -> crate::Result<risc0_zkvm::SessionInfo> {
    let _ = run_fixture(fixture, &mut env)?;

    let executor = risc0_zkvm::default_executor();
    let env = env.build().map_err(|e| crate::Error::FailedToBuildProverEnv(e.to_string()))?;
    let executor_info = executor
        .execute(env, FUEL_BLOCK_EXECUTION_GAME_RISC0_ELF)
        .map_err(|e| crate::Error::FailedToExecuteProvingGame(e.to_string()))?;

    Ok(executor_info)
}

pub fn prove_fixture(
    fixture: Fixture,
    mut env: ExecutorEnvBuilder<'_>,
) -> crate::Result<ProveInfo> {
    let block_id = run_fixture(fixture, &mut env)?;

    let prover = risc0_zkvm::default_prover();
    let env = env.build().map_err(|e| crate::Error::FailedToBuildProverEnv(e.to_string()))?;
    let prove_info = prover
        .prove(env, FUEL_BLOCK_EXECUTION_GAME_RISC0_ELF)
        .map_err(|e| crate::Error::FailedToProveProvingGame(e.to_string()))?;
    let output: Vec<u8> = prove_info
        .receipt
        .journal
        .decode()
        .map_err(|e| crate::Error::FailedToDeserializePublicOutput(e.to_string()))?;

    let decoded_output = PublicValuesStruct::abi_decode(&output, true)
        .map_err(|e| crate::Error::FailedToDeserializePublicOutput(e.to_string()))?;

    let output_block_id = decoded_output.block_id.to_be_bytes();
    if output_block_id != block_id {
        return Err(crate::Error::Fault(format!(
            "Block ID mismatch: expected {:?}, got {:?}",
            block_id, output_block_id
        )));
    }

    Ok(prove_info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FUEL_BLOCK_EXECUTION_GAME_RISC0_ID;
    use csv::WriterBuilder;
    use fuel_zkvm_primitives_test_fixtures::block_execution_fixtures::fixtures::all_fixtures;
    use serde::Serialize;

    #[derive(Serialize)]
    struct ExecutionReport {
        fixture: Fixture,
        cycle_count: u64,
    }

    #[derive(Serialize)]
    struct ProvingReport {
        fixture: Fixture,
        proving_time: u128,
        verification_time: u128,
    }

    #[test]
    fn run_all_fixtures_and_collect_report() {
        let fixtures = all_fixtures();

        let file_path =
            std::env::var("FUEL_RISC0_REPORT").unwrap_or("fuel_risc0_report.csv".to_string());
        let mut wtr = WriterBuilder::new()
            .flexible(true)
            .from_path(file_path)
            .expect("Couldn't create CSV writer");

        for fixture in fixtures {
            let env = ExecutorEnvBuilder::default();
            let executor_info = execute_fixture(fixture.clone(), env).unwrap();

            let report =
                ExecutionReport { fixture: fixture.clone(), cycle_count: executor_info.cycles() };

            wtr.serialize(report).expect("Couldn't write report to CSV");

            // Flush the CSV writer to ensure the report is written to disk.
            wtr.flush().expect("Couldn't flush CSV writer");
        }
    }

    #[test]
    fn prove_all_fixtures_and_collect_report() {
        let fixtures = all_fixtures();

        let file_path =
            std::env::var("FUEL_RISC0_REPORT").unwrap_or("fuel_risc0_report.csv".to_string());
        let mut wtr = WriterBuilder::new()
            .flexible(true)
            .from_path(file_path)
            .expect("Couldn't create CSV writer");

        for fixture in fixtures {
            let env = ExecutorEnvBuilder::default();

            let start_time = std::time::Instant::now();
            let prove_info = prove_fixture(fixture.clone(), env).unwrap();
            let proving_time = start_time.elapsed().as_millis();

            let start_time = std::time::Instant::now();
            prove_info
                .receipt
                .verify(FUEL_BLOCK_EXECUTION_GAME_RISC0_ID)
                .expect("Proof verification failed.");
            let verification_time = start_time.elapsed().as_millis();

            let report =
                ProvingReport { fixture: fixture.clone(), proving_time, verification_time };

            wtr.serialize(report).expect("Couldn't write report to CSV");

            // Flush the CSV writer to ensure the report is written to disk.
            wtr.flush().expect("Couldn't flush CSV writer");
        }
    }
}
