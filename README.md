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
cargo run -- fulfill --projection-input target/build-materialization.json
cargo run -- init --state-file target/build-state.json
cargo run -- run --state succeeded
cargo run -- run --state blocked
cargo run -- run --state succeeded --state-file target/build-state.json
cargo run -- projection --input target/build-materialization.json
cargo run -- status --state-file target/build-state.json
```

Stateful runs persist build runs, artifact refs, build proofs, and runner
operation evidence together. The runner contribution is evidence for
fulfillment; it does not make the runner own build semantics.

`projection --input` validates a build materialization projection: source and
content-index refs mapped to filesystem-shaped build inputs, reverse mapping
refs, toolchain refs, dependency refs, storage object refs, and transition
conflict posture. It consumes refs, not raw source paths, as the build lane
boundary.

`fulfill --projection-input` uses that projection as the build input and emits
build run, artifact, proof, runner operation, release-candidate, log, metric,
and storage object refs. The projection ref is carried as input/evidence; Cargo
remains only a current adapter behind the projection.

When a generated manifest projection carries storage-backed dependency inputs,
`constitute-build` does not report the Cargo path residency fallback as a
compatibility adapter. Repo-local paths may still be tool materialization for
Cargo, but dependency selection comes from version/content/storage refs.

Build fixtures and run outcomes also emit a host-fabric build-processor member
contribution. That contribution lets fabric reduce host composition posture
without making fabric execute builds, select releases, or own source truth.
