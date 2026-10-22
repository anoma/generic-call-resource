# generic_call_witness

Witness library for the `GenericCall` resource kind in the
[Generic Call Resource](https://github.com/anoma/generic-call-resource)
application.

This crate defines the data carried by a `GenericCall` resource and the
constraint logic (`LogicCircuit::constrain`) that runs inside the RISC0 zkVM
guest.

## What it provides

- **`GenericCall`** — a single EVM call: `to` (contract address), `value`
  (native token amount in wei), and `data` (ABI-encoded selector and calldata).
- **`GenericCallWitness`** — the full witness for one resource: the `Resource`,
  whether it is consumed, the action tree root, the (optional) nullifier key,
  the forwarder address, the list of calls, and the optional `ordering_info`
  (set with `GenericCallWitness::with_ordering`).
- **`OrderingInfo`** — the `after_tags` / `before_tags` call-ordering tags; see
  [Call ordering](#call-ordering).
- **`encode_generic_call_forwarder_input`** — ABI-encodes `&[GenericCall]` as
  `Call[]`, matching `abi.decode(input, (Call[]))` in the
  `GenericCallForwarder` contract.
- **`calculate_label_ref`** / **`calculate_value_ref`** — compute
  `hash(forwarder_addr)` and `hash(abi_encode(calls))` respectively.
- **`encode_ordering_tags`** — encodes the `after_tags` / `before_tags` of
  `OrderingInfo` as the `bytes32` ordering tags carried by `ForwarderCalldata`.

## Constraint logic

`GenericCallWitness::constrain` enforces that:

1. The resource is **ephemeral** (a `GenericCall` resource must never be
   persistent).
2. `resource.label_ref == hash(forwarder_addr)`.
3. `resource.value_ref == hash(abi_encode(calls))`.
4. A nullifier key is present when the resource is consumed.

On success it returns a `LogicInstance` carrying the resource's nullifier
(consumed) or commitment (created), and the ABI-encoded forwarder calldata as an
external payload blob for the protocol adapter to deliver.

## Call ordering

`GenericCallWitness` carries an optional `OrderingInfo`:

```rust
pub struct OrderingInfo {
    pub after_tags: Vec<Digest>,  // calls that must run before this one
    pub before_tags: Vec<Digest>, // calls that must not run before this one
}
```

A tag in `after_tags` names a resource whose forwarder calls must have run
before this resource's calls, one in `before_tags` a resource whose calls must
not have. `ordering_info: None` — the default for every constructor — is an
unordered call, and `GenericCallWitness::with_ordering` (or
`GenericCallLogic::with_ordering` on the host side) sets it.

When it is present, `constrain` copies the tags onto the forwarder calldata's
`afterTags` / `beforeTags` fields; the protocol adapter enforces the ordering
when it delivers the calls. They are *not* part of `label_ref` or `value_ref`,
so changing them does not change the resource.

## Usage

This crate is consumed by:

- [`generic_call_library`](../generic_call_library/) — the host-side prover.
- [`generic_call_circuit`](../generic_call_circuit/) — the zkVM guest that calls
  `constrain()`.

## License

GPL-3.0
