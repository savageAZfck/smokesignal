# Threat model

smokesignal is a correctness protocol, not a security boundary — its
threats are desynchronization and invalid acceptance, not adversaries.

## What it defends against

- **Divergent frontiers.** After every round both sides rewind over the
  same discarded positions and ingest the same committed tokens —
  frontier drift is the failure mode this exists to prevent.
- **False acceptance.** Commits require argmax equality at every
  prefix position; a draft token the verifier wouldn't have produced
  can never commit.
- **Lost verifier words.** The correction/bonus token always commits —
  the verifier is never left a token behind the accepted prefix.
- **Stale drafting.** Empty drafts end the round without a verify
  pass — a dead drafter can't stall the verifier.

## What it does not defend against

- **A dishonest verifier.** The verifier is authoritative by
  definition; if it lies, the committed sequence is its lie. Keep the
  verifier in the trust boundary.
- **Sampling drift.** Any caller that samples on either side voids the
  correctness contract — the protocol documents greedy-only but cannot
  enforce it.
- **Vocab mismatches.** Drafts in one tokenizer's space verified in
  another produce garbage, not errors — assert tokenizer parity at
  setup.
- **Side-channel across accelerators.** The draft's residency on a
  second silicon is the perf feature, not a security feature.

## Design posture

Both sides must be able to prove they hold the same sequence after
every exchange — the round isn't over until the frontiers match.
