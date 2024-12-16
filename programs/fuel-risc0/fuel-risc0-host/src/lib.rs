use alloy_sol_types::SolType;
use fuel_risc0_methods::FUEL_RISC0_PROVER_ELF;
use fuel_zkvm_primitives_prover::{Input, PublicValuesStruct};
use fuel_zkvm_primitives_test_fixtures::Fixture;
use risc0_zkvm::{ExecutorEnvBuilder, ProveInfo, SessionInfo};

pub fn run_fixture(fixture: Fixture, env: &mut ExecutorEnvBuilder<'_>) -> [u8; 32] {
    let raw_input = Fixture::get_input_for_fixture(&fixture);
    let input: Input = bincode::deserialize(&raw_input).unwrap();
    let block_id = input.block.header().id().into();
    env.write(&raw_input).expect("Failed to write input to environment");

    block_id
}

#[allow(unused)]
pub fn execute_program(fixture: Fixture, mut env: ExecutorEnvBuilder<'_>) -> SessionInfo {
    let _ = run_fixture(fixture, &mut env);

    let executor = risc0_zkvm::default_executor();
    let executor_info = executor.execute(env.build().unwrap(), FUEL_RISC0_PROVER_ELF).unwrap();

    executor_info
}

pub fn prove_program(fixture: Fixture, mut env: ExecutorEnvBuilder<'_>) -> ProveInfo {
    let block_id = run_fixture(fixture, &mut env);

    let prover = risc0_zkvm::default_prover();
    let prove_info = prover.prove(env.build().unwrap(), FUEL_RISC0_PROVER_ELF).unwrap();
    let output: Vec<u8> = prove_info.receipt.journal.decode().unwrap();

    let decoded_output = PublicValuesStruct::abi_decode(&output, true).unwrap();

    assert_eq!(decoded_output.block_id.to_be_bytes(), block_id);
    prove_info
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv::WriterBuilder;
    use fuel_risc0_methods::FUEL_RISC0_PROVER_ID;
    use fuel_zkvm_primitives_test_fixtures::all_fixtures;
    use risc0_zkvm::SegmentInfo;
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct SegmentInfoCrateLocal {
        pub po2: u32,
        pub cycles: u32,
    }

    impl From<SegmentInfo> for SegmentInfoCrateLocal {
        fn from(segment: SegmentInfo) -> Self {
            Self { po2: segment.po2, cycles: segment.cycles }
        }
    }

    impl From<Vec<SegmentInfo>> for Segments {
        fn from(value: Vec<SegmentInfo>) -> Self {
            Self(value.into_iter().map(Into::into).collect())
        }
    }

    #[derive(Serialize)]
    pub struct Segments(Vec<SegmentInfoCrateLocal>);

    #[derive(Serialize)]
    struct ExecutionReport {
        fixture: Fixture,
        cycle_count: u64,
        segments: Segments,
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
            let executor_info = execute_program(fixture.clone(), env);

            let report = ExecutionReport {
                fixture: fixture.clone(),
                cycle_count: executor_info.cycles(),
                segments: executor_info.segments.into(),
            };

            wtr.serialize(report).expect("Couldn't write report to CSV");

            // Flush the CSV writer to ensure the report is written to disk.
            wtr.flush().expect("Couldn't flush CSV writer");

            tracing::info!("Executed fixture: {:?}", fixture);
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
            let prove_info = prove_program(fixture.clone(), env);
            let proving_time = start_time.elapsed().as_millis();

            let start_time = std::time::Instant::now();
            prove_info.receipt.verify(FUEL_RISC0_PROVER_ID).expect("Proof verification failed.");
            let verification_time = start_time.elapsed().as_millis();

            let report =
                ProvingReport { fixture: fixture.clone(), proving_time, verification_time };

            wtr.serialize(report).expect("Couldn't write report to CSV");

            // Flush the CSV writer to ensure the report is written to disk.
            wtr.flush().expect("Couldn't flush CSV writer");

            tracing::info!("Proved fixture: {:?}", fixture);
        }
    }
}
