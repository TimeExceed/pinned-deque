use crate::*;
use quickcheck_macros::quickcheck;
use std::collections::VecDeque;

#[quickcheck]
fn cmp(a: Vec<usize>, b: Vec<usize>) {
    let oracle_a: VecDeque<_> = a.iter().copied().collect();
    let oracle_b: VecDeque<_> = b.iter().copied().collect();
    let trial_a: PinnedDeque<usize, 2> = a.iter().copied().collect();
    let trial_b: PinnedDeque<usize, 2> = b.iter().copied().collect();
    let oracle_cmp = oracle_a.cmp(&oracle_b);
    let trial_cmp = trial_a.cmp(&trial_b);
    assert_eq!(trial_cmp, oracle_cmp);
}

#[quickcheck]
fn eq(a: Vec<usize>, b: Vec<usize>) {
    let oracle_a: VecDeque<_> = a.iter().copied().collect();
    let oracle_b: VecDeque<_> = b.iter().copied().collect();
    let trial_a: PinnedDeque<usize, 2> = a.iter().copied().collect();
    let trial_b: PinnedDeque<usize, 2> = b.iter().copied().collect();
    let oracle_eq = oracle_a.eq(&oracle_b);
    let trial_eq = trial_a.eq(&trial_b);
    assert_eq!(oracle_eq, trial_eq);
}
