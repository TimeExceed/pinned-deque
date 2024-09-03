use crate::*;
use quickcheck_macros::quickcheck;
use std::collections::*;

#[quickcheck]
fn debug(a: Vec<usize>) {
    let oracle: VecDeque<_> = a.iter().copied().collect();
    let trial: Deque<_, 2> = a.iter().copied().collect();
    assert_eq!(format!("{trial:?}"), format!("{oracle:?}"));
}

#[quickcheck]
fn extend(a: Vec<usize>, b: Vec<usize>) {
    let oracle = {
        let mut oracle: VecDeque<_> = a.iter().copied().collect();
        oracle.extend(b.iter().copied());
        oracle
    };
    let trial = {
        let mut trial: Deque<_, 2> = a.iter().copied().collect();
        trial.extend(b.iter().copied());
        trial
    };
    assert_eq!(format!("{trial:?}"), format!("{oracle:?}"));
}

#[quickcheck]
fn from_slice_ref(a: Vec<usize>) {
    let trial: Deque<usize, 2> = a.as_slice().into();
    assert_eq!(format!("{trial:?}"), format!("{a:?}"));
}

#[test]
fn from_slice() {
    let trial: Deque<_, 2> = [1, 2, 3].into();
    assert_eq!(format!("{trial:?}"), "[1, 2, 3]");
}

#[quickcheck]
fn from_vec(a: Vec<usize>) {
    let trial: Deque<_, 2> = a.clone().into();
    assert_eq!(format!("{trial:?}"), format!("{a:?}"));
}

#[test]
fn clone() {
    let origin: Deque<A, 2> = [A("origin".to_owned())].into();
    let cloned = origin.clone();
    assert_eq!(format!("{cloned:?}"), "[cloned_origin]");
}

struct A(String);

impl Clone for A {
    fn clone(&self) -> Self {
        A(format!("cloned_{}", self.0))
    }
}

impl std::fmt::Debug for A {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
