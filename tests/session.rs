use smokesignal::{ScriptedDrafter, ScriptedVerifier, Session};

#[test]
fn full_match_commits_prefix_plus_bonus() {
    // Drafter says [1,2,3]; verifier agrees with all three and adds its own.
    let mut d = ScriptedDrafter::new(vec![0, 1, 2, 3]);
    let mut v = ScriptedVerifier::new(vec![0, 1, 2, 3, 4]);
    let mut s = Session::new(&mut d, &mut v, vec![0]);

    let r = s.round(3);
    assert_eq!(r.drafts, vec![1, 2, 3]);
    assert_eq!(r.accepted, 3);
    // Bonus verifier token commits after the full match.
    assert_eq!(r.committed, vec![1, 2, 3, 4]);
    assert_eq!(s.frontier, vec![0, 1, 2, 3, 4]);
}

#[test]
fn divergence_commits_verifier_correction() {
    // Drafter [1,2,3]; verifier agrees on 1,2 but says 9 at position 3.
    let mut d = ScriptedDrafter::new(vec![0, 1, 2, 3]);
    let mut v = ScriptedVerifier::new(vec![0, 1, 2, 9, 4]);
    let mut s = Session::new(&mut d, &mut v, vec![0]);

    let r = s.round(3);
    assert_eq!(r.accepted, 2);
    assert_eq!(r.committed, vec![1, 2, 9], "correction, not the draft");
    assert_eq!(s.frontier, vec![0, 1, 2, 9]);
}

#[test]
fn total_rejection_still_advances() {
    // Verifier disagrees at the very first draft position.
    let mut d = ScriptedDrafter::new(vec![0, 7, 7, 7]);
    let mut v = ScriptedVerifier::new(vec![0, 9, 4, 5]);
    let mut s = Session::new(&mut d, &mut v, vec![0]);

    let r = s.round(3);
    assert_eq!(r.accepted, 0);
    assert_eq!(r.committed, vec![9], "the verifier's word commits");
    assert_eq!(s.frontier, vec![0, 9]);
}

#[test]
fn stats_track_acceptance() {
    let mut d = ScriptedDrafter::new(vec![0, 1, 2, 3]);
    let mut v = ScriptedVerifier::new(vec![0, 1, 2, 9, 4]);
    let mut s = Session::new(&mut d, &mut v, vec![0]);
    s.round(3);
    let st = s.stats();
    assert_eq!(st.rounds, 1);
    assert_eq!(st.proposed, 3);
    assert_eq!(st.accepted, 2);
    assert!((st.acceptance_rate() - 2.0 / 3.0).abs() < 1e-9);
    assert_eq!(st.verifier_tokens, 1);
}

#[test]
fn run_stops_when_drafter_runs_out() {
    let mut d = ScriptedDrafter::new(vec![0, 1]);
    let mut v = ScriptedVerifier::new(vec![0, 1, 2]);
    let mut s = Session::new(&mut d, &mut v, vec![0]);
    let rounds = s.run(4, 10);
    // Round 1 commits [1, 2]; round 2 drafts nothing → empty, stop.
    assert_eq!(rounds.len(), 2);
    assert!(rounds[1].drafts.is_empty());
    assert_eq!(s.frontier, vec![0, 1, 2]);
}
