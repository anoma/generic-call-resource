# Resource Logic

This document specifies what the generic call resource logic checks, what it leaves unchecked, and the reason for each. The code is in [`generic_call_witness/src/lib.rs`](../generic_call_witness/src/lib.rs). The glossary [`CONTEXT.md`](../CONTEXT.md#language) defines the terms.

## 1. In-circuit checks

`GenericCallWitness::constrain` makes these in-circuit checks for each generic call resource, consumed or created:

1. The resource is ephemeral.
2. The label ref is the SHA-256 hash of the forwarder address of the witness.
3. The value ref is the SHA-256 hash of the ABI encoding of the calls, as `Call(address to, uint256 value, bytes data)[]`.
4. The tag is the nullifier of a consumed resource, which needs its nullifier key, or the commitment of a created resource.

The logic instance holds the tag, the side, the action tree root and the app data. The external payload has one blob: a call to the forwarder address, with the encoded calls as the input and an empty expected output. Its deletion criterion is `Immediately`. The resource payload, the discovery payload and the application payload are empty.

So the proof fixes the forwarder and the calls of the resource. The aggregation proof covers the app data, so the protocol adapter calls exactly this forwarder with exactly these calls.

## 2. What the logic leaves unchecked

| Not checked                          | Reason                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Who creates or consumes the resource | The logic checks no signature, so any prover can make a generic call resource with any calls. A transaction must leave no tokens in the forwarder. So the calls can use only what other resources of the same action give the forwarder, and these resources need their own authorization, for example the signature of the owner over the action tree root for an unwrap to the forwarder. The action tree root covers the tag of the generic call resource, so this signature also covers its calls. |
| The side                             | The resource is ephemeral, so it exists only in this transaction. Consumed or created, it makes the same calls once, and its tag only goes into the nullifier set or into the commitment tree.                                                                                                                                                                                                                                                                                                         |
| The calls                            | Any target, any amount of native tokens and any data. The forwarder makes them as they are.                                                                                                                                                                                                                                                                                                                                                                                                            |
| The quantity                         | The logic does not use the quantity. A generic call resource usually has the quantity 0, which adds nothing to the delta. Another quantity must balance with resources of the same kind point, like the quantity of any resource.                                                                                                                                                                                                                                                                      |
| The nullifier key                    | A consumed ephemeral resource does not exist before the transaction, so the prover chooses all its fields, also the commitment to its nullifier key.                                                                                                                                                                                                                                                                                                                                                   |

## 3. Checks outside the logic

- **The compliance proof** calculates the commitment, the nullifier and the delta of each resource, and it checks the nonces of the created resources. The delta proof shows that the transaction balances.
- **The protocol adapter** makes the external call, compares its output with the empty expected output, and rejects a repeated nullifier.
- **The generic call forwarder** accepts calls only from its protocol adapter and only with its logic ref, and it rejects a call without data and without value.

## 4. Trust assumptions

- **Other resources of the action authorize the calls.** An action that gives the forwarder tokens must also authorize the calls, for example through the signature of the owner over the action tree root.
- **A transaction leaves no tokens and no allowances in the forwarder.** The calls of any later transaction can take tokens that stay in the forwarder.

## 5. Parameters

| Name                           | Value                                                                                        |
| ------------------------------ | -------------------------------------------------------------------------------------------- |
| Label ref                      | `SHA-256(forwarder address)`                                                                 |
| Value ref                      | `SHA-256(abi.encode(Call[]))`                                                                |
| Logic ref                      | `GENERIC_CALL_ID` in [`generic_call_library/src/lib.rs`](../generic_call_library/src/lib.rs) |
| Deletion criterion of the call | `Immediately`                                                                                |
