use smokesignal::{ScriptedDrafter, ScriptedVerifier, Session};
use std::hint::black_box;
use std::time::Instant;

fn main() {
    let mut d = ScriptedDrafter::new((0..1_000_000u32).collect());
    let mut v = ScriptedVerifier::new((0..1_000_100u32).collect());
    let mut s = Session::new(&mut d, &mut v, vec![0]);
    let t = Instant::now();
    let rounds = s.run(black_box(4), black_box(10_000));
    println!(
        "{} rounds committing {} tokens in {:?}",
        rounds.len(),
        s.frontier.len() - 1,
        t.elapsed()
    );
}
