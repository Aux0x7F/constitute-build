# constitute-build

Rust CLI and library for build contract posture.

`constitute-build` models a build as a swarm processor contract fulfilled by a
runner. A build run consumes source operation refs, source snapshot refs,
content-index refs, recipe refs, runner/resource grants, and secret posture,
then emits storage-backed artifact refs, proof refs, and release candidate
refs. It does not own source/version truth, storage bytes, release selection,
or runner process truth.

Build run state is reduced from contract readiness, source operation/snapshot
refs, recipe refs, runner availability, action/resource grants, secret posture,
compatibility posture, and release-candidate output. A caller cannot make a
failed prerequisite look like an artifact-producing build by passing a state
flag.

## Commands

```powershell
cargo run -- fixture run
cargo run -- init --state-file target/build-state.json
cargo run -- run --state succeeded
cargo run -- run --state blocked
cargo run -- run --state succeeded --state-file target/build-state.json
cargo run -- status --state-file target/build-state.json
```

Stateful runs persist build runs, artifact refs, build proofs, and runner
operation evidence together. The runner contribution is evidence for
fulfillment; it does not make the runner own build semantics.

Build fixtures and run outcomes also emit a host-fabric build-processor member
contribution. That contribution lets fabric reduce host composition posture
without making fabric execute builds, select releases, or own source truth.
