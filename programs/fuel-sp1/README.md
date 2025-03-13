# fuel-sp1

## Install Dependencies

- Rust MSRV 1.85.0
   ```
    rustup toolchain install 1.85.0
    ```

- sp1 toolchain
    ```
    curl -L https://sp1up.succinct.xyz | bash
    sp1up --version 4.1.3
    ```

## Repository structure

Since each proving game should have its own entrypoint, it lives as its own crate.
We have `fuel-proving-games-sp1` which aims to aggregate these and provides the following features -

1. If imported as a library, you may access helpers to generate and verify proofs for the associated proving game.
2. If executed as a binary, you may run proof generation & verification via CLI (upcoming).

## Run proving tests

TBD

## Run execution tests

TBD
