//! smokesignal — the draft/verify handshake.
//!
//! Signals between two worlds. A small, resident draft model proposes
//! `k` tokens; a large model verifies all `k` proposals in a single
//! forward pass and commits only the longest matching prefix, plus the
//! verifier's own correction at the first divergence. Both sides then
//! rewind their KV frontiers over the discarded drafts and ingest the
//! same committed tokens, so the next round starts from an identical
//! sequence on both accelerators.
//!
//! Ported from Bad Apple's cross-silicon speculative decoding — a 0.6B
//! drafter resident on the Apple Neural Engine proposing to a 7B
//! verifier on the GPU, where the draft's residency is the point: it
//! costs the unified-memory budget ~nothing where a GPU-resident draft
//! model could not fit at all.
//!
//! **Token-selection semantics: this protocol is GREEDY.** Drafts and
//! verification are argmax-vs-argmax. Sampling on either side
//! invalidates the acceptance measurement.

use serde::{Deserialize, Serialize};

// ──────────────────────────── the sides ─────────────────────────────

/// The proposing mind — small, cheap, resident on a low-power
/// accelerator. Implementations must be deterministic argmax
/// (greedy) drafters.
pub trait Drafter {
    /// Propose up to `k` continuation tokens after `frontier`.
    fn draft(&mut self, frontier: &[u32], k: usize) -> Vec<u32>;
    /// Rewind the draft-side KV frontier over `n` discarded positions,
    /// then ingest `tokens` so the drafter's state matches the
    /// verifier's accepted sequence exactly.
    fn resync(&mut self, rewind: usize, ingest: &[u32]);
    /// Current position index of the draft-side frontier.
    fn position(&self) -> usize;
}

/// The verifying mind — large, authoritative. Greedy argmax only:
/// comparing a sampled draft to a sampled verify breaks the protocol's
/// acceptance math.
pub trait Verifier {
    /// One forward pass over `frontier + drafts`, returning the
    /// verifier's own argmax at each of the `drafts.len()` positions
    /// (i.e. what the verifier *would have said* at each draft slot).
    /// `return[i]` is the verifier token at frontier + i.
    fn verify(&mut self, frontier: &[u32], drafts: &[u32]) -> Vec<u32>;
    /// Rewind the verifier KV frontier over `n` rejected verify
    /// positions, then ingest `tokens` so verifier and drafter agree.
    fn resync(&mut self, rewind: usize, ingest: &[u32]);
    /// Current position index of the verifier-side frontier.
    fn position(&self) -> usize;
}

// ──────────────────────────── the round ─────────────────────────────

/// What one draft/verify round committed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Round {
    /// The drafter's proposals.
    pub drafts: Vec<u32>,
    /// How many draft tokens the verifier agreed with (longest
    /// matching prefix).
    pub accepted: usize,
    /// Every token committed this round: the accepted prefix plus the
    /// verifier's correction at the first divergence (or its bonus
    /// continuation when all drafts matched).
    pub committed: Vec<u32>,
}

/// The handshake. Runs rounds of propose → verify → commit → resync,
/// keeping both sides' frontiers identical throughout.
pub struct Session<'a, D: Drafter, V: Verifier> {
    pub drafter: &'a mut D,
    pub verifier: &'a mut V,
    /// Token ids both sides agree on — the shared sequence.
    pub frontier: Vec<u32>,
    stats: Stats,
}

/// Acceptance accounting across the session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stats {
    pub rounds: u64,
    pub proposed: u64,
    pub accepted: u64,
    /// Tokens committed from verifier authority (corrections + bonus).
    pub verifier_tokens: u64,
}

impl Stats {
    /// Fraction of draft tokens the verifier agreed with.
    pub fn acceptance_rate(&self) -> f64 {
        if self.proposed == 0 {
            0.0
        } else {
            self.accepted as f64 / self.proposed as f64
        }
    }
}

impl<'a, D: Drafter, V: Verifier> Session<'a, D, V> {
    /// A session over the shared `frontier` — both sides must already
    /// be ingested to this exact sequence.
    pub fn new(drafter: &'a mut D, verifier: &'a mut V, frontier: Vec<u32>) -> Self {
        Self {
            drafter,
            verifier,
            frontier,
            stats: Stats::default(),
        }
    }

    /// Session statistics.
    pub fn stats(&self) -> &Stats {
        &self.stats
    }

    /// One round: drafter proposes ≤k, verifier checks all proposals in
    /// a single pass, the longest matching prefix plus the verifier's
    /// correction commits, both sides resync identically.
    pub fn round(&mut self, k: usize) -> Round {
        let drafts = self.drafter.draft(&self.frontier, k);
        // A drafter with nothing to propose ends the round without a
        // verify pass — the frontier stays put.
        if drafts.is_empty() {
            let r = Round {
                drafts,
                accepted: 0,
                committed: Vec::new(),
            };
            self.stats.rounds += 1;
            return r;
        }
        let verdict = self.verifier.verify(&self.frontier, &drafts);

        // Longest matching prefix: accept[i] iff draft[i] ==
        // verifier's argmax at that position.
        let mut accepted = 0;
        for (i, d) in drafts.iter().enumerate() {
            match verdict.get(i) {
                Some(&v) if v == *d => accepted += 1,
                _ => break,
            }
        }

        // Committed = accepted prefix + verifier's own token at the
        // first divergence. When all k matched, the verifier's token
        // past the last draft (the "bonus" token) commits too — if it
        // reported one. A short verdict commits only the prefix.
        let mut committed: Vec<u32> = drafts[..accepted].to_vec();
        if let Some(&correction) = verdict.get(accepted) {
            committed.push(correction);
        }

        // Resync both sides: rewind discarded draft/verify positions,
        // then ingest the same committed sequence so frontiers agree.
        let drafter_discard = drafts.len() - accepted;
        self.drafter.resync(drafter_discard, &committed);
        let verifier_discard = drafts.len() - accepted;
        self.verifier.resync(verifier_discard, &committed);
        self.frontier.extend_from_slice(&committed);

        self.stats.rounds += 1;
        self.stats.proposed += drafts.len() as u64;
        self.stats.accepted += accepted as u64;
        self.stats.verifier_tokens += (committed.len() - accepted) as u64;

        Round {
            drafts,
            accepted,
            committed,
        }
    }

    /// Run `max_rounds` rounds (or until a round commits nothing or the
    /// drafter stops proposing). Returns all rounds in order.
    pub fn run(&mut self, k: usize, max_rounds: usize) -> Vec<Round> {
        let mut out = Vec::new();
        for _ in 0..max_rounds {
            let r = self.round(k);
            let done = r.committed.is_empty() || r.drafts.is_empty();
            out.push(r);
            if done {
                break;
            }
        }
        out
    }
}

// ─────────────────────── reference implementation ───────────────────

/// A scripted drafter for tests and examples: replays a fixed draft
/// table over the shared frontier. Greedy argmax by construction.
pub struct ScriptedDrafter {
    table: Vec<u32>,
    frontier_len: usize,
}

impl ScriptedDrafter {
    /// `table` is the token stream the drafter proposes; position is
    /// tracked against the frontier length.
    pub fn new(table: Vec<u32>) -> Self {
        Self {
            table,
            frontier_len: 0,
        }
    }
}

impl Drafter for ScriptedDrafter {
    fn draft(&mut self, frontier: &[u32], k: usize) -> Vec<u32> {
        let start = frontier.len().min(self.table.len());
        self.table[start..(start + k).min(self.table.len())].to_vec()
    }
    fn resync(&mut self, _rewind: usize, _ingest: &[u32]) {}
    fn position(&self) -> usize {
        self.frontier_len
    }
}

/// A scripted verifier: replies with its own argmax table.
pub struct ScriptedVerifier {
    table: Vec<u32>,
}

impl ScriptedVerifier {
    pub fn new(table: Vec<u32>) -> Self {
        Self { table }
    }
}

impl Verifier for ScriptedVerifier {
    fn verify(&mut self, _frontier: &[u32], drafts: &[u32]) -> Vec<u32> {
        // The verifier's own continuation starts right after frontier —
        // modeled as `table[frontier.len()..]`; for tests we track the
        // sequence in the session and index by position.
        let base = _frontier.len();
        (0..=drafts.len())
            .map(|i| *self.table.get(base + i).unwrap_or(&u32::MAX))
            .collect()
    }
    fn resync(&mut self, _rewind: usize, _ingest: &[u32]) {}
    fn position(&self) -> usize {
        0
    }
}
