use anoma_generic_call_witness::LogicCircuit;
use anoma_generic_call_witness::GenericCallWitness;
use risc0_zkvm::guest::env;

fn main() {
    let witness: GenericCallWitness = env::read();

    let instance = witness.constrain().unwrap();

    env::commit(&instance);
}
