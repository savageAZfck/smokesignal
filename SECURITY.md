# Security policy

## Reporting

Open a private security advisory on the GitHub repository, or email the
maintainer (see `Cargo.toml` authors).

## Scope

In scope:

- Frontier desynchronization — a round path where drafter and verifier
  ingest different sequences or rewind different counts.
- Acceptance of a draft the verifier did not argmax-match.
- Committed sequences missing the correction/bonus token.
- Stats that misreport acceptance (e.g. counting the correction as an
  accepted draft).

Out of scope:

- Verifier correctness — the large model is authoritative by design.
- Driver-side KV management — `resync` semantics are implemented by
  each runtime; the crate only specifies the contract.
- Throughput/regression concerns — acceptance rate is reported, not
  guaranteed.

## Guidance

- Assert tokenizer parity between the two sides before the first
  round — a vocab mismatch is silent garbage.
- Keep both runtimes greedy on this path; route sampled generation
  through a non-speculative path.
- Log `Stats` per session — a sudden acceptance collapse is a health
  signal (drafter drift, checkpoint mismatch, or a subtle resync bug).
