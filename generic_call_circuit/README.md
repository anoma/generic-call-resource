# Reproducibly generate proving and verifying keys (ELF and ImageID)

You may generate different ELFs and ImageIDs on different machines and environments. To reproduce the same output and publicly verify that the ELF and ImageID correspond to the generic call circuit source code, use the following tool and command.

## Reproducing the `GENERIC_CALL_ID` committed in `generic_call_library`

The `GENERIC_CALL_ID` constant in [`generic_call_library/src/lib.rs`](../generic_call_library/src/lib.rs) is the `ImageID` of the guest ELF built from this repo at the commit it was last rotated at. To reproduce it, run:

```bash
cargo risczero build --manifest-path generic_call_circuit/methods/guest/Cargo.toml
```

which reproduces:

```bash
ELFs ready at:
ImageID: dc64937e533f5eccf6041da5f49c905e6a28f76111ae2a5bc24160e67bd50dea
generic_call_circuit/methods/guest/target/riscv32im-risc0-zkvm-elf/docker/generic-call-guest.bin
```

The same build is also embedded as the committed guest binary at `generic_call_library/elf/generic-call-guest.bin` in this repo.

It was last rotated for the `anoma-rm-risc0` / `anoma-rm-risc0-gadgets` bump to
`v2.0.0-rc.7.1`, which added the `afterTags` / `beforeTags` ordering fields to
`ForwarderCalldata`, and for the witness-side `OrderingInfo` (`after_tags` /
`before_tags`) that now populates them — both change the ABI-encoded forwarder
calldata the guest commits to.

## ImageID drift after future changes

Running `cargo risczero build` against a later commit may produce an `ImageID` that no longer matches the committed `GENERIC_CALL_ID`. That's expected whenever a transitive dependency changes without the committed ID being rotated — see below.

### Why the values drift

The `ImageID` is a digest over the entire compiled guest binary, which transitively pins:

- the circuit source in this crate (`anoma_generic_call_circuit`),
- `anoma_generic_call_witness` (the resource-logic/constraint code the guest actually links against — this is a guest dependency; `anoma_generic_call_library` is host-only and is **not** compiled into the guest),
- the ARM libraries (`anoma_rm_risc0`, `anoma_rm_risc0_gadgets`),
- the toolchain and any other transitive dependency.

*Any* change to `anoma_generic_call_witness` — including a version bump that does not touch resource-logic behavior — re-links the guest and produces a new `ImageID`. The same is true for ARM library upgrades. The circuit's own source code can be byte-for-byte identical and the `ImageID` will still change.

### Why we do not bump `GENERIC_CALL_ID` on every rebuild

`GENERIC_CALL_ID` is consumed off-chain (app-level checks) and on-chain (the deployed protocol adapter / verifier contracts). Rotating it requires:

1. redeploying the protocol-adapter contracts on every supported chain, and
2. updating the `ID in PA` reference used by downstream integrations.

We only pay that cost when the **circuit logic itself** changes — i.e., when the proof semantics change. Dependency or toolchain bumps that leave the logic unchanged are intentionally allowed to leave `GENERIC_CALL_ID` and the committed `generic-call-guest.bin` on the older build, even though `cargo risczero build` would now produce a different `ImageID`.

If you need the two values to agree (e.g., before a release that does change circuit logic), update both together:

1. Rebuild with the command above.
2. Replace `generic_call_library/elf/generic-call-guest.bin` with the freshly built ELF.
3. Update `GENERIC_CALL_ID` in `generic_call_library/src/lib.rs` to the new `ImageID`.
4. Coordinate the corresponding protocol-adapter redeployment and PA update.

Note: The unstable feature of risc0-zkvm currently causes issues in circuits. This can be temporarily fixed by manually updating the tool. The problem will be fully resolved in the next release of RISC Zero.

```bash
cargo install --force --git https://github.com/risc0/risc0 --tag v3.0.3 -Fexperimental cargo-risczero
```
