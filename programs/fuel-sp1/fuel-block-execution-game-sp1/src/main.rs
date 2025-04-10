//! contains the sp1 hook for running the block execution game

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]

sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolType;
use fuel_zkvm_primitives_prover::games::block_execution_game::{prove, PublicValuesStruct};

pub fn main() {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a custom system call which handles reading inputs
    // from the prover.
    let bytes = sp1_zkvm::io::read_vec();

    let proof = prove(&bytes).expect("Proof generation failed");

    // Encode the public values of the program.
    let bytes = PublicValuesStruct::abi_encode(&proof);

    // Commit to the public values of the program. The final proof will have a commitment to all the
    // bytes that were committed to.
    sp1_zkvm::io::commit_slice(&bytes);
}
