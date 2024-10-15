use super::*;
use crate::*;
use quickcheck_macros::*;
use std::collections::VecDeque;

#[test]
fn itermut_mutability() {
    let mut trial = PinnedDeque::<usize>::with_capacity_per_chunk(2);
    trial.push_back(0);
    trial.push_back(1);
    trial.push_back(2);
    trial.iter_mut().for_each(|x| {
        *x *= 2;
    });
    let trial: Vec<_> = trial.iter().map(|x| *x).collect();
    assert_eq!(trial, vec![0, 2, 4]);
}

#[quickcheck]
fn iter_bothwards(ops: Vec<Op>) {
    let oracle = ops_to_oracle(&ops);
    let (oracle_front_half, oracle_back_half) = {
        let mut oracle_front_half = vec![];
        let mut oracle_back_half = vec![];
        let mut it = oracle.into_iter();
        loop {
            let Some(front) = it.next() else {
                break;
            };
            oracle_front_half.push(front);
            let Some(back) = it.next_back() else {
                break;
            };
            oracle_back_half.push(back);
        }
        (oracle_front_half, oracle_back_half)
    };
    let trial = ops_to_trial(&ops);
    let (trial_front_halt, trial_back_half) = {
        let mut trial_front_halt = vec![];
        let mut trial_back_half = vec![];
        let mut it = trial.iter();
        loop {
            let Some(front) = it.next() else {
                break;
            };
            trial_front_halt.push(*front);
            let Some(back) = it.next_back() else {
                break;
            };
            trial_back_half.push(*back);
        }
        (trial_front_halt, trial_back_half)
    };
    assert_eq!(trial_front_halt, oracle_front_half);
    assert_eq!(trial_back_half, oracle_back_half);
}

#[quickcheck]
fn iter_size_hint(ops: Vec<Op>) {
    let oracle = ops_to_oracle(&ops);
    let oracle_size_hints = {
        let mut res = vec![];
        let mut it = oracle.iter();
        loop {
            res.push(it.size_hint());
            let Some(_) = it.next() else {
                break;
            };
            res.push(it.size_hint());
            let Some(_) = it.next_back() else {
                break;
            };
        }
        res
    };
    let trial = ops_to_trial(&ops);
    let trial_size_hints = {
        let mut res = vec![];
        let mut it = trial.iter();
        loop {
            res.push(it.size_hint());
            let Some(_) = it.next() else {
                break;
            };
            res.push(it.size_hint());
            let Some(_) = it.next_back() else {
                break;
            };
        }
        res
    };
    assert_eq!(trial_size_hints, oracle_size_hints);
}

#[quickcheck]
fn itermut_bothwards(ops: Vec<Op>) {
    let oracle = ops_to_oracle(&ops);
    let (oracle_front_half, oracle_back_half) = {
        let mut oracle_front_half = vec![];
        let mut oracle_back_half = vec![];
        let mut it = oracle.into_iter();
        loop {
            let Some(front) = it.next() else {
                break;
            };
            oracle_front_half.push(front);
            let Some(back) = it.next_back() else {
                break;
            };
            oracle_back_half.push(back);
        }
        (oracle_front_half, oracle_back_half)
    };
    let mut trial = ops_to_trial(&ops);
    let (trial_front_halt, trial_back_half) = {
        let mut trial_front_halt = vec![];
        let mut trial_back_half = vec![];
        let mut it = trial.iter_mut();
        loop {
            let Some(front) = it.next() else {
                break;
            };
            trial_front_halt.push(*front);
            let Some(back) = it.next_back() else {
                break;
            };
            trial_back_half.push(*back);
        }
        (trial_front_halt, trial_back_half)
    };
    assert_eq!(trial_front_halt, oracle_front_half);
    assert_eq!(trial_back_half, oracle_back_half);
}

#[quickcheck]
fn itermut_size_hint(ops: Vec<Op>) {
    let oracle = ops_to_oracle(&ops);
    let oracle_size_hints = {
        let mut res = vec![];
        let mut it = oracle.iter();
        loop {
            res.push(it.size_hint());
            let Some(_) = it.next() else {
                break;
            };
            res.push(it.size_hint());
            let Some(_) = it.next_back() else {
                break;
            };
        }
        res
    };
    let mut trial = ops_to_trial(&ops);
    let trial_size_hints = {
        let mut res = vec![];
        let mut it = trial.iter_mut();
        loop {
            res.push(it.size_hint());
            let Some(_) = it.next() else {
                break;
            };
            res.push(it.size_hint());
            let Some(_) = it.next_back() else {
                break;
            };
        }
        res
    };
    assert_eq!(trial_size_hints, oracle_size_hints);
}

#[quickcheck]
fn intoiter(ops: Vec<Op>) {
    let oracle = ops_to_oracle(&ops);
    let trial: VecDeque<_> = {
        let trial = ops_to_trial(&ops);
        trial.into_iter().collect()
    };
    assert_eq!(trial, oracle);
}

fn ops_to_oracle(ops: &[Op]) -> VecDeque<usize> {
    let mut res = VecDeque::new();
    for op in ops.iter() {
        match op {
            Op::PopBack => {
                let _ = res.pop_back();
            }
            Op::PopFront => {
                let _ = res.pop_front();
            }
            Op::PushBack(x) => {
                res.push_back(*x);
            }
            Op::PushFront(x) => {
                res.push_front(*x);
            }
        }
    }
    res
}

fn ops_to_trial(ops: &[Op]) -> PinnedDeque<usize> {
    let mut res = PinnedDeque::with_capacity_per_chunk(2);
    for op in ops.iter() {
        match op {
            Op::PopBack => {
                let _ = res.pop_back();
            }
            Op::PopFront => {
                let _ = res.pop_front();
            }
            Op::PushBack(x) => {
                res.push_back(*x);
            }
            Op::PushFront(x) => {
                res.push_front(*x);
            }
        }
    }
    res
}
