# fuel-risc0

## Install Dependencies

- Rust MSRV 1.82.0
   ```
    rustup toolchain install 1.82.0
    ```

- sp1 toolchain
    ```
    curl -L https://risczero.com/install | bash
    rzup install --version 1.2.5
    ```

## Repository structure

Since each proving game should have its own entrypoint, it lives as its own crate (currently colocated within the host crate).
We have `fuel-proving-games-risc0` which aims to aggregate these and provides the following features -

1. If imported as a library, you may access helpers to generate and verify proofs for the associated proving game.
2. If executed as a binary, you may run proof generation & verification via CLI (upcoming).

## Run proving tests

```
cargo test -p fuel-proving-games-risc0 prove_all_fixtures_and_collect_report
```

Make sure you use the correct env vars for the specific prover.

For CUDA proving, use the following feature flag:
```
cargo test -p fuel-proving-games-risc0 prove_all_fixtures_and_collect_report --features cuda
```

see [here](https://dev.risczero.com/api/generating-proofs/local-proving#nvidia-gpu) for installation instructions for the drivers.

## Run execution tests

```
cargo test -p fuel-proving-games-risc0 run_all_fixtures_and_collect_report
```
