# Generic Call Resource

The resource logic that lets an action of the Anoma Resource Machine (ARM) make arbitrary calls on an EVM chain through the generic call forwarder.

## Language

**Generic call resource**:
An ephemeral resource whose resource logic is the generic call logic. Its label ref commits to the generic call forwarder, and its value ref commits to a list of generic calls.

**Generic call**:
One EVM call `(to, value, data)` that the generic call forwarder makes: a target address, an amount of native tokens and the call data.

**Generic call forwarder**:
The contract that makes the generic calls of a resource when the protocol adapter calls it.

**Resource**:
A unit of state with a quantity. A **persistent** resource stays until a later transaction consumes it. An **ephemeral** resource exists only in one transaction.

**Tag / Commitment / Nullifier**:
The tag of a resource in a transaction is its nullifier if the transaction consumes it, else its commitment. Only a holder of the nullifier key of a resource can calculate its nullifier.

**Resource logic / Logic ref**:
The circuit that decides when a transaction can create or consume a resource. Its logic ref identifies its code.

**Label ref / Value ref**:
Two hashes in a resource. The label ref separates the resources of one resource logic, and the value ref commits to the data of one resource.

**Action tree root**:
The root of a Merkle tree over the tags of an action, in the order of its list.

**External call**:
A call that the protocol adapter makes to a forwarder when it processes a resource. The external payload of the resource carries it.

**Protocol adapter**:
The contract that verifies transactions on an EVM chain and makes the external calls of their resources.

**In-circuit check**:
A check that a circuit makes: a proof of the circuit is valid only if the check holds.

**Out-of-circuit check**:
A check outside the circuits, for example a check that the protocol adapter or the forwarder makes on chain.
