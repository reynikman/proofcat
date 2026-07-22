# Contributing

ProofCat treats data-integrity changes as safety-critical.

## Before submitting a change

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
npm ci
npm run check
npm run lint
```

Changes to copying, checkpoint, verification, MHL or final verdicts must add a
fault test. A test must prove that a failure cannot produce a stronger verdict
than the evidence supports.

Do not add a new MHL hash element unless it is supported by the official ASC
MHL schema/reference implementation. Report-only hashes belong in JSON
evidence, not in the interoperability manifest.

Never commit client media, real client paths, signing keys, DSNs, tokens or
private infrastructure runbooks.
