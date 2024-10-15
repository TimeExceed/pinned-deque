use crate::*;
use std::cmp::*;

impl<T> PartialOrd for PinnedDeque<T>
where
    T: Sized + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut it_a = self.iter();
        let mut it_b = other.iter();
        loop {
            let Some(a) = it_a.next() else {
                break;
            };
            let Some(b) = it_b.next() else {
                break;
            };
            let cmp = a.partial_cmp(&b);
            if !matches!(cmp, Some(Ordering::Equal)) {
                return cmp;
            }
        }
        match (it_a.next(), it_b.next()) {
            (Some(_), None) => Some(Ordering::Greater),
            (None, Some(_)) => Some(Ordering::Less),
            (None, None) => Some(Ordering::Equal),
            _ => unreachable!(),
        }
    }
}

impl<T> PartialEq for PinnedDeque<T>
where
    T: Sized + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        let mut it_a = self.iter();
        let mut it_b = other.iter();
        loop {
            let Some(a) = it_a.next() else {
                break;
            };
            let Some(b) = it_b.next() else {
                break;
            };
            let eq = a.eq(&b);
            if !eq {
                return false;
            }
        }
        true
    }
}

impl<T> Ord for PinnedDeque<T>
where
    T: Sized + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<T> Eq for PinnedDeque<T> where T: Sized + Eq {}
