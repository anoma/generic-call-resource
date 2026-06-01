//! Host-side prover for GenericCall resource logic.
//!
//! `GENERIC_CALL_ELF` and `GENERIC_CALL_ID` are placeholders. After building
//! the guest with the RISC0 toolchain (`cd crates/generic_call_circuit && cargo build`),
//! copy the output ELF to `elf/generic-call-guest.bin` and update `GENERIC_CALL_ID`
//! with the printed image ID.

use anoma_rm_risc0::{
    Digest, logic_proof::LogicProver, nullifier_key::NullifierKey, resource::Resource,
};
pub use generic_call_witness::GenericCall;
use generic_call_witness::GenericCallWitness;
use hex::FromHex;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

pub const GENERIC_CALL_ELF: &[u8] = include_bytes!("../elf/generic-call-guest.bin");

lazy_static! {
    pub static ref GENERIC_CALL_ID: Digest =
        Digest::from_hex("105d65173f4f961acc8be18ef3fee633fc0444832fc93c5fea439c118795e055")
            .unwrap();
}

/// Wraps a `GenericCallWitness` and implements `LogicProver` so it can be used
/// for proof generation inside the RISC0 zkVM.
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct GenericCallLogic {
    pub witness: GenericCallWitness,
}

impl GenericCallLogic {
    /// Logic for a consumed ephemeral GenericCall resource (e.g. the resource
    /// representing the intent to execute EVM calls).
    pub fn consumed_ephemeral_resource_logic(
        resource: Resource,
        action_tree_root: Digest,
        nf_key: NullifierKey,
        forwarder_addr: Vec<u8>,
        calls: Vec<GenericCall>,
    ) -> Self {
        Self {
            witness: GenericCallWitness {
                resource,
                is_consumed: true,
                action_tree_root,
                nf_key: Some(nf_key),
                forwarder_addr,
                calls,
            },
        }
    }

    /// Logic for a created ephemeral GenericCall resource (balancing resource).
    pub fn created_ephemeral_resource_logic(
        resource: Resource,
        action_tree_root: Digest,
        forwarder_addr: Vec<u8>,
        calls: Vec<GenericCall>,
    ) -> Self {
        Self {
            witness: GenericCallWitness {
                resource,
                is_consumed: false,
                action_tree_root,
                nf_key: None,
                forwarder_addr,
                calls,
            },
        }
    }
}

impl LogicProver for GenericCallLogic {
    type Witness = GenericCallWitness;

    fn proving_key() -> &'static [u8] {
        GENERIC_CALL_ELF
    }

    fn verifying_key() -> Digest {
        *GENERIC_CALL_ID
    }

    fn witness(&self) -> &Self::Witness {
        &self.witness
    }
}
