# smokesignal specification

Draft/verify speculative decoding across heterogeneous accelerators.

## Roles

- **Drafter** — small, cheap, resident on a low-power accelerator.
  `draft(frontier, k)` proposes ≤k continuations; `resync(rewind,
  ingest)` rewinds its KV frontier over discarded positions and ingests
  the committed tokens.
- **Verifier** — large, authoritative. `verify(frontier, drafts)`
  returns its argmax at each draft position in one forward pass;
  `resync` mirrors the drafter's state sync.

## Round semantics

`accept[i] = drafts[i] == verdict[i]` for the longest prefix.
`committed = drafts[..accepted] + verdict[accepted]` — the verifier's
own token at the first divergence, or the bonus continuation on full
match. A short verdict commits only the prefix. Empty drafts commit
nothing — no verify pass is made.

Both sides then `resync(drafts.len() - accepted, committed)` — the
discarded positions are rewound and the identical committed sequence
is ingested, so frontiers agree exactly before the next round.

## Token-selection contract

**Greedy only.** The protocol is argmax-vs-argmax by construction:
sampling on either side makes the acceptance measurement meaningless
and the resync ambiguous.

## Stats

`Stats` tracks rounds, proposed, accepted, verifier-sourced tokens,
and `acceptance_rate()` = accepted/proposed — the number that decides
whether speculation is paying.

## Non-goals

- smokesignal is the handshake, not the runtime — actual CoreML/MLX/CUDA
  drivers implement the traits.
- It does not handle vocab mismatches between drafter and verifier —
  both sides must share a tokenizer.
- Batched/parallel speculation is out of scope; the protocol is a
  single draft chain per session.
