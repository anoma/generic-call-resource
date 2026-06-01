use anoma_rm_risc0::{Digest, nullifier_key::NullifierKey, resource::Resource};
use generic_call_library::GenericCallLogic;
use generic_call_witness::{
    GenericCall, GenericCallWitness, LogicCircuit, calculate_label_ref, calculate_value_ref,
    encode_generic_call_forwarder_input,
};

const FORWARDER_ADDR: [u8; 20] = [0u8; 20];
const EVM_TARGET_ADDR: [u8; 20] = [1u8; 20];
const NF_KEY_BYTES: [u8; 32] = [2u8; 32];

fn calls() -> Vec<GenericCall> {
    vec![GenericCall {
        to: EVM_TARGET_ADDR.to_vec(),
        value: 0,
        data: vec![0xde, 0xad, 0xbe, 0xef],
    }]
}

fn other_calls() -> Vec<GenericCall> {
    vec![
        GenericCall {
            to: EVM_TARGET_ADDR.to_vec(),
            value: 0,
            data: vec![0xde, 0xad, 0xbe, 0xef],
        },
        GenericCall {
            to: EVM_TARGET_ADDR.to_vec(),
            value: 0,
            data: vec![0xca, 0xfe, 0xba, 0xbe],
        },
    ]
}

fn ephemeral_resource() -> Resource {
    let label_ref = calculate_label_ref(&FORWARDER_ADDR);
    let value_ref = calculate_value_ref(&encode_generic_call_forwarder_input(&calls()).unwrap());
    let nk_commitment = NullifierKey::from_bytes(NF_KEY_BYTES).commit();
    Resource {
        logic_ref: Digest::default(),
        label_ref,
        value_ref,
        nk_commitment,
        is_ephemeral: true,
        ..Default::default()
    }
}

#[test]
fn test_consumed_ephemeral() {
    let resource = ephemeral_resource();
    let witness = GenericCallLogic::consumed_ephemeral_resource_logic(
        resource,
        Digest::default(),
        NullifierKey::from_bytes(NF_KEY_BYTES),
        FORWARDER_ADDR.to_vec(),
        calls(),
    );
    witness.witness.constrain().unwrap();
}

#[test]
fn test_created_ephemeral() {
    let resource = ephemeral_resource();
    let witness = GenericCallLogic::created_ephemeral_resource_logic(
        resource,
        Digest::default(),
        FORWARDER_ADDR.to_vec(),
        calls(),
    );
    witness.witness.constrain().unwrap();
}

#[test]
fn test_non_ephemeral_rejected() {
    let label_ref = calculate_label_ref(&FORWARDER_ADDR);
    let value_ref = calculate_value_ref(&encode_generic_call_forwarder_input(&calls()).unwrap());
    let nk_commitment = NullifierKey::from_bytes(NF_KEY_BYTES).commit();
    let resource = Resource {
        logic_ref: Digest::default(),
        label_ref,
        value_ref,
        nk_commitment,
        is_ephemeral: false, // persistent — must be rejected
        ..Default::default()
    };
    let witness = GenericCallWitness {
        resource,
        is_consumed: false,
        action_tree_root: Digest::default(),
        nf_key: None,
        forwarder_addr: FORWARDER_ADDR.to_vec(),
        calls: calls(),
    };
    witness.constrain().unwrap_err();
}

#[test]
fn test_missing_nf_key() {
    let resource = ephemeral_resource();
    let witness = GenericCallWitness {
        resource,
        is_consumed: true,
        action_tree_root: Digest::default(),
        nf_key: None, // missing
        forwarder_addr: FORWARDER_ADDR.to_vec(),
        calls: calls(),
    };
    witness.constrain().unwrap_err();
}

#[test]
fn test_wrong_label_ref() {
    let forwarder_addr = FORWARDER_ADDR.to_vec();
    let wrong_label_ref = calculate_label_ref(&[0xFFu8; 20]);

    let calls = calls();
    let value_ref = calculate_value_ref(&encode_generic_call_forwarder_input(&calls).unwrap());

    let nk_commitment = NullifierKey::from_bytes(NF_KEY_BYTES).commit();
    let resource = Resource {
        logic_ref: Digest::default(),
        label_ref: wrong_label_ref,
        value_ref,
        nk_commitment,
        is_ephemeral: true,
        ..Default::default()
    };
    let witness = GenericCallWitness {
        resource,
        is_consumed: false,
        action_tree_root: Digest::default(),
        nf_key: None,
        forwarder_addr,
        calls,
    };
    witness.constrain().unwrap_err();
}

#[test]
fn test_wrong_value_ref() {
    let forwarder_addr = FORWARDER_ADDR.to_vec();
    let label_ref = calculate_label_ref(&forwarder_addr);

    let calls = calls();
    let other_calls = other_calls();
    let wrong_value_ref =
        calculate_value_ref(&encode_generic_call_forwarder_input(&other_calls).unwrap());

    let nk_commitment = NullifierKey::from_bytes(NF_KEY_BYTES).commit();
    let resource = Resource {
        logic_ref: Digest::default(),
        label_ref,
        value_ref: wrong_value_ref,
        nk_commitment,
        is_ephemeral: true,
        ..Default::default()
    };
    let witness = GenericCallWitness {
        resource,
        is_consumed: false,
        action_tree_root: Digest::default(),
        nf_key: None,
        forwarder_addr,
        calls,
    };
    witness.constrain().unwrap_err();
}
