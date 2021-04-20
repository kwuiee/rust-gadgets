extern crate bio;

use std::env;

use bio::alignment::Alignment;
use bio::pattern_matching::myers::long;

fn main() {
    let rawp = env::args().nth(1).unwrap().to_string();
    let rawt = env::args().nth(2).unwrap().to_string();
    let pattern = rawp.as_bytes();
    let text = rawt.as_bytes();
    let mut myers = long::Myers::<u64>::new(pattern);
    let mut aln = Alignment::default();
    let mut matches = myers.find_all_lazy(text, 150);
    let (best_end, _) = matches.by_ref().min_by_key(|&(_, dist)| dist).unwrap();
    matches.alignment_at(best_end, &mut aln);
    println!(
        "Best alignment at {}..{} (distance: {})",
        aln.ystart, aln.yend, aln.score
    );
    println!("{}", aln.pretty(pattern, text));
}
