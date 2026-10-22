//! Host-side prover for GenericCall resource logic.
//!
//! `GENERIC_CALL_ELF` and `GENERIC_CALL_ID` are the committed guest build. To
//! rotate them, rebuild the guest reproducibly
//! (`cargo risczero build --manifest-path generic_call_circuit/methods/guest/Cargo.toml`),
//! copy the output ELF to `elf/generic-call-guest.bin` and update
//! `GENERIC_CALL_ID` with the printed image ID. See
//! `generic_call_circuit/README.md`.

pub use anoma_generic_call_witness::GenericCall;
use anoma_generic_call_witness::GenericCallWitness;
use anoma_rm_risc0::{
    Digest, logic_proof::LogicProver, nullifier_key::NullifierKey, resource::Resource,
};
use hex::FromHex;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

pub const GENERIC_CALL_ELF: &[u8] = include_bytes!("../elf/generic-call-guest.bin");

lazy_static! {
    pub static ref GENERIC_CALL_ID: Digest =
        Digest::from_hex("dc64937e533f5eccf6041da5f49c905e6a28f76111ae2a5bc24160e67bd50dea")
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
                ordering_info: None,
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
                ordering_info: None,
            },
        }
    }

    /// Sets the `OrderingInfo` on the wrapped witness, constraining when this
    /// resource's forwarder call runs relative to the calls of other resources:
    /// a tag in `after_tags` names a resource whose calls must have run
    /// already, a tag in `before_tags` one whose calls must not have. Without
    /// it the call is unordered (`ordering_info: None`).
    pub fn with_ordering(mut self, after_tags: Vec<Digest>, before_tags: Vec<Digest>) -> Self {
        self.witness = self.witness.with_ordering(after_tags, before_tags);
        self
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
