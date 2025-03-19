use crate::FUEL_DECOMPRESSION_GAME_RISC0_ELF;
use alloy_sol_types::SolType;
use fuel_zkvm_primitives_prover::games::decompression_game::PublicValuesStruct;
use fuel_zkvm_primitives_test_fixtures::decompression_fixtures::Fixture;
use risc0_zkvm::{ExecutorEnvBuilder, ProveInfo};

pub fn run_fixture(fixture: Fixture, env: &mut ExecutorEnvBuilder<'_>) -> crate::Result<()> {
    let raw_input = Fixture::get_input_for_fixture(&fixture);
    env.write(&raw_input)
        .map_err(|e| crate::Error::FailedToWriteInputToProverEnv(e.to_string()))?;

    Ok(())
}

pub fn execute_fixture(
    fixture: Fixture,
    mut env: ExecutorEnvBuilder<'_>,
) -> crate::Result<risc0_zkvm::SessionInfo> {
    run_fixture(fixture, &mut env)?;

    let executor = risc0_zkvm::default_executor();
    let env = env.build().map_err(|e| crate::Error::FailedToBuildProverEnv(e.to_string()))?;

    let executor_info = executor
        .execute(env, FUEL_DECOMPRESSION_GAME_RISC0_ELF)
        .map_err(|e| crate::Error::FailedToExecuteProvingGame(e.to_string()))?;

    Ok(executor_info)
}

pub fn prove_fixture(
    fixture: Fixture,
    mut env: ExecutorEnvBuilder<'_>,
) -> crate::Result<ProveInfo> {
    run_fixture(fixture, &mut env)?;

    let prover = risc0_zkvm::default_prover();
    let env = env.build().map_err(|e| crate::Error::FailedToBuildProverEnv(e.to_string()))?;

    let prove_info = prover
        .prove(env, FUEL_DECOMPRESSION_GAME_RISC0_ELF)
        .map_err(|e| crate::Error::FailedToProveProvingGame(e.to_string()))?;
    let output: Vec<u8> = prove_info
        .receipt
        .journal
        .decode()
        .map_err(|e| crate::Error::FailedToDeserializePublicOutput(e.to_string()))?;

    let decoded_output = PublicValuesStruct::abi_decode(&output, true)
        .map_err(|e| crate::Error::FailedToDeserializePublicOutput(e.to_string()))?;

    if decoded_output.first_block_height > decoded_output.last_block_height {
        return Err(crate::Error::Fault(format!(
            "First block height is greater than last block height: {} > {}",
            decoded_output.first_block_height, decoded_output.last_block_height
        )));
    }

    Ok(prove_info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FUEL_DECOMPRESSION_GAME_RISC0_ID;
    use csv::WriterBuilder;
    use fuel_zkvm_primitives_test_fixtures::decompression_fixtures::all_fixtures;
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
                .verify(FUEL_DECOMPRESSION_GAME_RISC0_ID)
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
