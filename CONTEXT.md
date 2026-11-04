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

**App data / Deletion criterion**:
The app data of a resource holds four lists of blobs: the resource payload, the discovery payload, the external payload and the application payload. The deletion criterion of a blob is `Never` or `Immediately`. The protocol adapter emits each blob with `Never` as an event. Each blob of the external payload is an external call, whatever its deletion criterion.

**External call**:
A call that the protocol adapter makes to a forwarder when it processes a resource. The external payload of the resource carries it.

**Protocol adapter**:
The contract that verifies transactions on an EVM chain and makes the external calls of their resources.

**Kind / Kind base**:
The kind of a resource is the hash to curve of its logic ref and its label ref. Its kind base is the point that the compliance circuit multiplies by its quantity. Resources balance against each other only if they have the same kind base.
_Avoid_: kind point

**Compliance unit / Aggregation proof / Delta proof**:
Each action has one compliance unit. Its compliance proof shows the **compliance checks**: the in-circuit checks that the action follows the rules of the ARM. The aggregation proof aggregates the compliance proofs and the resource logic proofs of a transaction into one proof, so the protocol adapter verifies only the aggregation proof and the delta proof. The delta proof shows that a transaction balances: for each kind base, the consumed and the created quantities are equal.

**In-circuit check**:
A check that a circuit makes: a proof of the circuit is valid only if the check holds.

**Out-of-circuit check**:
A check outside the circuits, for example a check that the protocol adapter or the forwarder makes on chain.
