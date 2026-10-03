use smokesignal::{ScriptedDrafter, ScriptedVerifier, Session};

fn main() {
    // The drafter's guesses vs the verifier's ground truth.
    // Drafter proposes [1,2,3,4,5]; verifier agrees on [1,2] then
    // diverges at 3 (says 9), then tracks [4,5] — wait, it tracks its
    // own table from position: table[3]=9 means verifier's word wins.
    let mut drafter = ScriptedDrafter::new(vec![0, 1, 2, 3, 4, 5]);
    let mut verifier = ScriptedVerifier::new(vec![0, 1, 2, 9, 4, 5, 6]);

    let mut session = Session::new(&mut drafter, &mut verifier, vec![0]);

    for (i, r) in session.run(3, 4).iter().enumerate() {
        println!(
            "round {i}: drafts {:?} → accepted {} → committed {:?}",
            r.drafts, r.accepted, r.committed
        );
    }

    println!("\nfinal frontier: {:?}", session.frontier);
    let s = session.stats();
    println!(
        "stats: {} rounds, {}/{} accepted ({:.0}%), {} verifier tokens",
        s.rounds,
        s.accepted,
        s.proposed,
        s.acceptance_rate() * 100.0,
        s.verifier_tokens
    );
}
