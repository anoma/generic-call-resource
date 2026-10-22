use anoma_generic_call_library::GenericCallLogic;
use anoma_generic_call_witness::{
    GenericCall, GenericCallWitness, LogicCircuit, calculate_label_ref, calculate_value_ref,
    encode_generic_call_forwarder_input, encode_ordering_tags,
};
use anoma_rm_risc0::{
    Digest, nullifier_key::NullifierKey, resource::Resource, utils::words_to_bytes,
};
use anoma_rm_risc0_gadgets::evm::ForwarderCalldata;

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
            value: 1,
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
    let mut resource = ephemeral_resource();
    resource.is_ephemeral = false; // persistent — must be rejected
    let witness = GenericCallWitness {
        resource,
        is_consumed: false,
        action_tree_root: Digest::default(),
        nf_key: None,
        forwarder_addr: FORWARDER_ADDR.to_vec(),
        calls: calls(),
        ordering_info: None,
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
        ordering_info: None,
    };
    witness.constrain().unwrap_err();
}

#[test]
fn test_wrong_label_ref() {
    let mut resource = ephemeral_resource();

    // Use a different forwarder address in label and witness.
    resource.label_ref = calculate_label_ref(&[0xFFu8; 20]);
    let witness = GenericCallWitness {
        resource,
        is_consumed: false,
        action_tree_root: Digest::default(),
        nf_key: None,
        forwarder_addr: FORWARDER_ADDR.to_vec(),
        calls: calls(),
        ordering_info: None,
    };
    witness.constrain().unwrap_err();
}

#[test]
fn test_wrong_value_ref() {
    let mut resource = ephemeral_resource();

    // Use different calls in value and witness.
    resource.value_ref =
        calculate_value_ref(&encode_generic_call_forwarder_input(&other_calls()).unwrap());
    let witness = GenericCallWitness {
        resource,
        is_consumed: false,
        action_tree_root: Digest::default(),
        nf_key: None,
        forwarder_addr: FORWARDER_ADDR.to_vec(),
        calls: calls(),
        ordering_info: None,
    };
    witness.constrain().unwrap_err();
}

#[test]
fn test_ordering_tags_are_forwarded() {
    let after = Digest::new([3u32; 8]);
    let before = Digest::new([4u32; 8]);
    let logic = GenericCallLogic::consumed_ephemeral_resource_logic(
        ephemeral_resource(),
        Digest::default(),
        NullifierKey::from_bytes(NF_KEY_BYTES),
        FORWARDER_ADDR.to_vec(),
        calls(),
    )
    .with_ordering(vec![after], vec![before]);

    let instance = logic.witness.constrain().unwrap();
    let blob = &instance.app_data.external_payload[0];
    let calldata = ForwarderCalldata::decode(words_to_bytes(&blob.blob)).unwrap();

    assert_eq!(calldata.afterTags, encode_ordering_tags(&[after]));
    assert_eq!(calldata.beforeTags, encode_ordering_tags(&[before]));
}

#[test]
fn test_unordered_call_carries_no_tags() {
    let logic = GenericCallLogic::consumed_ephemeral_resource_logic(
        ephemeral_resource(),
        Digest::default(),
        NullifierKey::from_bytes(NF_KEY_BYTES),
        FORWARDER_ADDR.to_vec(),
        calls(),
    );
    assert!(logic.witness.ordering_info.is_none());

    let instance = logic.witness.constrain().unwrap();
    let blob = &instance.app_data.external_payload[0];
    let calldata = ForwarderCalldata::decode(words_to_bytes(&blob.blob)).unwrap();

    assert!(calldata.afterTags.is_empty());
    assert!(calldata.beforeTags.is_empty());
}
