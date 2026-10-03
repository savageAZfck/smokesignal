# smokesignal

The draft/verify handshake for speculative decoding across heterogeneous accelerators. Signals between two worlds — proposals out, verified answers back.

Extracted from Bad Apple's cross-silicon speculative decoding: a 0.6B drafter resident on the Apple Neural Engine proposes to a 7B verifier on the GPU — where **the draft's residency is the point**: it costs the unified-memory budget ~nothing where a GPU-resident draft model could not fit at all.

## The protocol

Each round:

1. **Propose** — the small model drafts up to `k` continuation tokens.
2. **Verify** — the large model evaluates all `k` positions in a *single* forward pass, returning its own argmax at each.
3. **Commit** — the longest matching prefix plus the verifier's correction at the first divergence (or a bonus token on full match).
4. **Resync** — both sides rewind their KV frontiers over discarded positions and ingest the same committed tokens, so the next round starts identical on both substrates.

## Greedy only

Drafts and verification are **argmax-vs-argmax**. Sampling on either side invalidates the acceptance measurement — the crate documents this contract rather than pretending otherwise.

## Usage

Implement `Drafter` and `Verifier` for your runtimes, then:

```rust
use smokesignal::Session;

let mut session = Session::new(&mut drafter, &mut verifier, frontier);
for round in session.run(k, max_rounds) {
    // round.committed is the sequence both sides agreed on
}
println!("acceptance: {:.0}%", session.stats().acceptance_rate() * 100.0);
```

`ScriptedDrafter`/`ScriptedVerifier` ship as reference implementations for tests.

## License

MIT
