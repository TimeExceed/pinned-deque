use super::*;
use crate::*;
use quickcheck_macros::quickcheck;
use std::collections::VecDeque;

#[quickcheck]
fn basic_ops(ops: Vec<Op>) {
    let mut oracle = VecDeque::new();
    let mut trial = PinnedDeque::<usize, 2>::new();
    for op in ops.into_iter() {
        match op {
            Op::PopBack => {
                let oracle_back = oracle.pop_back();
                let trial_back = trial.back().map(|x| *x);
                assert_eq!(oracle_back, trial_back);
                let r = trial.pop_back();
                assert_eq!(r.is_some(), oracle_back.is_some());
            }
            Op::PopFront => {
                let oracle_front = oracle.pop_front();
                let trial_front = trial.front().map(|x| *x);
                assert_eq!(oracle_front, trial_front);
                let r = trial.pop_front();
                assert_eq!(r.is_some(), oracle_front.is_some());
            }
            Op::PushBack(n) => {
                oracle.push_back(n);
                trial.push_back(n);
            }
            Op::PushFront(n) => {
                oracle.push_front(n);
                trial.push_front(n);
            }
        }
    }
    assert_eq!(trial.len(), oracle.len());
    {
        let trial: VecDeque<_> = trial.iter().map(|x| *x.get_ref()).collect();
        assert_eq!(trial, oracle);
    }
    {
        let trial: VecDeque<_> = trial.iter_mut().map(|x| *x.get_mut()).collect();
        assert_eq!(trial, oracle);
    }
    {
        let trial: VecDeque<_> = (0..trial.len())
            .map(|idx| *trial.get(idx).unwrap().get_ref())
            .collect();
        assert_eq!(trial, oracle);
    }
    {
        let trial: VecDeque<_> = (0..trial.len())
            .map(|idx| *trial.get_mut(idx).unwrap().get_mut())
            .collect();
        assert_eq!(trial, oracle);
    }
}

#[test]
fn drop_elems_0() {
    let mut buf = String::new();
    {
        use std::fmt::Write;
        let mut trial = PinnedDeque::<A<String>, 2>::new();
        trial.push_back(A {
            buf: &mut buf,
            id: "0".to_owned(),
        });
        trial.pop_back();
        writeln!(&mut buf, "popped back.").unwrap();
    }
    assert_eq!(
        buf,
        "0 is dropped.\
        \npopped back.\n"
    );
}

#[test]
fn drop_elems_1() {
    let mut buf = String::new();
    {
        use std::fmt::Write;
        let mut trial = PinnedDeque::<A<String>, 2>::new();
        trial.push_back(A {
            buf: &mut buf,
            id: "0".to_owned(),
        });
        trial.push_back(A {
            buf: &mut buf,
            id: "1".to_owned(),
        });
        writeln!(&mut buf, "deque is dropping.").unwrap();
    }
    assert_eq!(
        buf,
        "deque is dropping.\
        \n0 is dropped.\
        \n1 is dropped.\n"
    );
}

#[test]
fn clear() {
    let mut buf = String::new();
    {
        use std::fmt::Write;
        let mut trial = PinnedDeque::<A<String>, 2>::new();
        trial.push_back(A {
            buf: &mut buf,
            id: "0".to_owned(),
        });
        trial.clear();
        writeln!(&mut buf, "cleared.").unwrap();
    }
    assert_eq!(
        buf,
        "0 is dropped.\
        \ncleared.\n"
    );
}

struct A<W: std::fmt::Write> {
    buf: *mut W,
    id: String,
}

impl<W: std::fmt::Write> Drop for A<W> {
    fn drop(&mut self) {
        let w = unsafe { &mut *self.buf };
        writeln!(w, "{} is dropped.", self.id).unwrap();
    }
}

#[test]
fn get_mut() {
    let mut trial = PinnedDeque::<usize, 2>::new();
    trial.push_back(0);
    *trial.get_mut(0).unwrap() = 1;
    let trial: Vec<_> = trial.iter().map(|x| *x).collect();
    assert_eq!(trial, vec![1]);
}

#[test]
fn back_mut() {
    let mut trial = PinnedDeque::<usize, 2>::new();
    trial.push_back(0);
    *trial.back_mut().unwrap() = 1;
    let trial: Vec<_> = trial.iter().map(|x| *x).collect();
    assert_eq!(trial, vec![1]);
}

#[test]
fn front_mut() {
    let mut trial = PinnedDeque::<usize, 2>::new();
    trial.push_back(0);
    *trial.front_mut().unwrap() = 1;
    let trial: Vec<_> = trial.iter().map(|x| *x).collect();
    assert_eq!(trial, vec![1]);
}
