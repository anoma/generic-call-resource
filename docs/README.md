# Generic Call Resource

This folder specifies the generic call resource logic. The glossary of this repository, [`CONTEXT.md`](../CONTEXT.md#language), defines the terms.

## 1. Summary

A generic call resource lets an action make arbitrary calls on an EVM chain. The resource is ephemeral. Its label ref commits to the generic call forwarder, and its value ref commits to a list of calls. When the protocol adapter processes the resource, it calls the forwarder, and the forwarder makes the calls in their order.

The resource logic checks that the resource is ephemeral, that its label ref and its value ref match the forwarder and the calls, and that its external payload carries exactly these calls. It authorizes nothing: other resources of the same action must authorize what the calls do.

## 2. Documents

| Document                                         | Content                                                                        |
| ------------------------------------------------ | ------------------------------------------------------------------------------ |
| [`01-resource-logic.md`](./01-resource-logic.md) | What the generic call resource logic checks, what it leaves unchecked, and why |

## 3. Crates

- `generic_call_witness` holds the witness `GenericCallWitness` and its checks, which the circuit runs.
- `generic_call_library` holds the prover `GenericCallLogic`, the circuit binary, and its image ID `GENERIC_CALL_ID`, which is the logic ref.
- `generic_call_circuit` holds the source of the circuit binary and the tests of the circuit.
