# constitute-build

Rust CLI and library for build contract posture.

`constitute-build` models a build as a swarm processor contract fulfilled by a
runner. A build run consumes source snapshot refs, recipe refs, runner/resource
grants, and secret posture, then emits storage-backed artifact refs and proof
refs. It does not own source/version truth, storage bytes, release selection, or
runner process truth.

## Commands

```powershell
cargo run -- fixture run
cargo run -- run --state succeeded
cargo run -- run --state blocked
cargo run -- status
```
