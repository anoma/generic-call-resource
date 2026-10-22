//! Witness library for the GenericCall resource kind.
//!
//! A GenericCall resource can only be ephemeral. It encodes one or more EVM
//! calls that will be forwarded by the protocol adapter to the
//! `GenericCallForwarder` contract.

use alloy_primitives::{Address, B256, Bytes, U256};
use alloy_sol_types::{SolValue, sol};
pub use anoma_rm_risc0::resource_logic::LogicCircuit;
use anoma_rm_risc0::{
    Digest,
    error::ArmError,
    logic_instance::{AppData, ExpirableBlob, LogicInstance},
    nullifier_key::NullifierKey,
    resource::Resource,
    utils::{bytes_to_words, hash_bytes},
};
use anoma_rm_risc0_gadgets::evm::ForwarderCalldata;
use serde::{Deserialize, Serialize};

pub enum DeletionCriterion {
    Immediately = 0,
    Never = 1,
}

sol! {
    struct Call {
        address to;
        uint256 value;
        bytes data;
    }
}

/// A single EVM call to forward.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenericCall {
    /// Address of the EVM contract to call.
    pub to: Vec<u8>,
    /// Native token value (wei) to send with the call.
    pub value: u128,
    /// ABI-encoded function selector and calldata.
    pub data: Vec<u8>,
}

/// ABI-encodes `calls` as `Call[]` matching `abi.decode(input, (Call[]))` in
/// the `GenericCallForwarder` contract.
pub fn encode_generic_call_forwarder_input(calls: &[GenericCall]) -> Result<Vec<u8>, ArmError> {
    let sol_calls: Vec<Call> = calls
        .iter()
        .map(|c| {
            let to: Address =
                c.to.as_slice()
                    .try_into()
                    .map_err(|_| ArmError::ProveFailed("Invalid call `to` bytes".to_string()))?;
            Ok(Call {
                to,
                value: U256::from(c.value),
                data: Bytes::from(c.data.clone()),
            })
        })
        .collect::<Result<_, ArmError>>()?;

    Ok(sol_calls.abi_encode())
}

/// Encodes the `after_tags` / `before_tags` of `OrderingInfo` as the `bytes32`
/// ordering tags carried by `ForwarderCalldata`.
pub fn encode_ordering_tags(tags: &[Digest]) -> Vec<B256> {
    tags.iter()
        .map(|tag| B256::from_slice(tag.as_bytes()))
        .collect()
}

/// Computes `label_ref = hash(forwarder_addr)`.
pub fn calculate_label_ref(forwarder_addr: &[u8]) -> Digest {
    hash_bytes(forwarder_addr)
}

/// Computes `value_ref = hash(abi_encode(calls))`.
pub fn calculate_value_ref(encoded_calls: &[u8]) -> Digest {
    hash_bytes(encoded_calls)
}

/// OrderingInfo constrains when a forwarder call runs relative to the calls of
/// other resources. Absent ordering info means an unordered call.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OrderingInfo {
    /// Tags of the resources whose forwarder calls must run before this call.
    pub after_tags: Vec<Digest>,
    /// Tags of the resources whose forwarder calls must not run before this call.
    pub before_tags: Vec<Digest>,
}

/// Witness for a single GenericCall resource (consumed or created ephemeral).
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct GenericCallWitness {
    pub resource: Resource,
    pub is_consumed: bool,
    pub action_tree_root: Digest,
    /// Required when `is_consumed == true`.
    pub nf_key: Option<NullifierKey>,
    /// Address of the `GenericCallForwarder` contract.
    pub forwarder_addr: Vec<u8>,
    /// The EVM calls to execute.
    pub calls: Vec<GenericCall>,
    /// See OrderingInfo struct. None is an unordered call.
    pub ordering_info: Option<OrderingInfo>,
}

impl GenericCallWitness {
    /// Constrains when this forwarder call runs relative to the calls of other
    /// resources. A tag in `after_tags` names a resource whose calls must have
    /// run already; a tag in `before_tags` names one whose calls must not have.
    pub fn with_ordering(mut self, after_tags: Vec<Digest>, before_tags: Vec<Digest>) -> Self {
        self.ordering_info = Some(OrderingInfo {
            after_tags,
            before_tags,
        });
        self
    }
}

impl LogicCircuit for GenericCallWitness {
    fn constrain(&self) -> Result<LogicInstance, ArmError> {
        if !self.resource.is_ephemeral {
            return Err(ArmError::ProveFailed(
                "GenericCall resource must be ephemeral".to_string(),
            ));
        }

        let expected_label_ref = calculate_label_ref(&self.forwarder_addr);
        if self.resource.label_ref != expected_label_ref {
            return Err(ArmError::ProveFailed(
                "Invalid resource label_ref".to_string(),
            ));
        }

        let encoded_calls = encode_generic_call_forwarder_input(&self.calls)?;

        let expected_value_ref = calculate_value_ref(&encoded_calls);
        if self.resource.value_ref != expected_value_ref {
            return Err(ArmError::ProveFailed(
                "Invalid resource value_ref".to_string(),
            ));
        }

        let tag = if self.is_consumed {
            let nf_key = self
                .nf_key
                .as_ref()
                .ok_or(ArmError::MissingField("Nullifier key"))?;
            self.resource.nullifier(nf_key)?
        } else {
            self.resource.commitment()
        };

        let forwarder_calldata =
            ForwarderCalldata::from_bytes(&self.forwarder_addr, encoded_calls, vec![]);
        let forwarder_calldata = match self.ordering_info.as_ref() {
            Some(ordering_info) => forwarder_calldata.with_ordering(
                encode_ordering_tags(&ordering_info.after_tags),
                encode_ordering_tags(&ordering_info.before_tags),
            ),
            None => forwarder_calldata,
        };
        let external_blob = ExpirableBlob {
            blob: bytes_to_words(&forwarder_calldata.encode()),
            deletion_criterion: DeletionCriterion::Immediately as u32,
        };

        let app_data = AppData {
            resource_payload: vec![],
            discovery_payload: vec![],
            external_payload: vec![external_blob],
            application_payload: vec![],
        };

        Ok(LogicInstance {
            tag,
            is_consumed: self.is_consumed,
            root: self.action_tree_root,
            app_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anoma_rm_risc0::utils::words_to_bytes;

    #[test]
    fn ordering_tags_reach_the_forwarder_calldata() {
        let after = Digest::new([1u32; 8]);
        let before = Digest::new([2u32; 8]);

        let calldata = ForwarderCalldata::from_bytes(&[0xffu8; 20], vec![0xab], vec![])
            .with_ordering(
                encode_ordering_tags(&[after]),
                encode_ordering_tags(&[before]),
            );

        let decoded = ForwarderCalldata::decode(&calldata.encode()).unwrap();

        assert_eq!(decoded.afterTags, vec![B256::from_slice(after.as_bytes())]);
        assert_eq!(
            decoded.beforeTags,
            vec![B256::from_slice(before.as_bytes())]
        );
    }

    #[test]
    fn constrain_emits_the_ordering_tags_in_the_external_payload() {
        let after = Digest::new([3u32; 8]);
        let forwarder_addr = vec![0xffu8; 20];
        let calls = vec![GenericCall {
            to: vec![0x11u8; 20],
            value: 0,
            data: vec![0xab],
        }];
        let encoded_calls = encode_generic_call_forwarder_input(&calls).unwrap();

        let witness = GenericCallWitness {
            resource: Resource {
                is_ephemeral: true,
                label_ref: calculate_label_ref(&forwarder_addr),
                value_ref: calculate_value_ref(&encoded_calls),
                ..Resource::default()
            },
            is_consumed: false,
            action_tree_root: Digest::default(),
            nf_key: None,
            forwarder_addr,
            calls,
            ordering_info: None,
        }
        .with_ordering(vec![after], vec![]);

        let instance = witness.constrain().unwrap();
        let blob = &instance.app_data.external_payload[0];
        let decoded = ForwarderCalldata::decode(words_to_bytes(&blob.blob)).unwrap();

        assert_eq!(decoded.afterTags, vec![B256::from_slice(after.as_bytes())]);
        assert!(decoded.beforeTags.is_empty());
    }
}
