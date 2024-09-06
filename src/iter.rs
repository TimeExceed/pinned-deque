use crate::*;
use std::{iter::*, marker::PhantomData, pin::Pin};

#[derive(Clone)]
pub struct Iter<'a, T: Sized, const CAP_PER_PAGE: usize> {
    underlying: &'a PinnedDeque<T, CAP_PER_PAGE>,
    front_idx: usize,
    back_idx: usize,
}

pub struct IterMut<'a, T: Sized, const CAP_PER_PAGE: usize> {
    underlying: *mut PinnedDeque<T, CAP_PER_PAGE>,
    front_idx: usize,
    back_idx: usize,
    _ph: PhantomData<&'a mut PinnedDeque<T, CAP_PER_PAGE>>,
}

impl<'a, T, const CAP_PER_PAGE: usize> Iterator for Iter<'a, T, CAP_PER_PAGE>
where
    T: Sized,
{
    type Item = Pin<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        let cur_idx = self.front_idx;
        if cur_idx < self.back_idx {
            self.front_idx += 1;
            self.underlying.get(cur_idx)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.back_idx - self.front_idx;
        (remaining, Some(remaining))
    }
}

impl<'a, T, const CAP_PER_PAGE: usize> Iterator for IterMut<'a, T, CAP_PER_PAGE>
where
    T: Sized,
{
    type Item = Pin<&'a mut T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.front_idx < self.back_idx {
            let underlying: &'a mut _ = self.as_mut();
            let cur_idx = self.front_idx;
            self.front_idx += 1;
            underlying.get_mut(cur_idx)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.back_idx - self.front_idx;
        (remaining, Some(remaining))
    }
}

impl<'a, T, const CAP_PER_PAGE: usize> DoubleEndedIterator for Iter<'a, T, CAP_PER_PAGE>
where
    T: Sized,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front_idx < self.back_idx {
            self.back_idx -= 1;
            self.underlying.get(self.back_idx)
        } else {
            None
        }
    }
}

impl<'a, T, const CAP_PER_PAGE: usize> DoubleEndedIterator for IterMut<'a, T, CAP_PER_PAGE>
where
    T: Sized,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front_idx < self.back_idx {
            let underlying: &'a mut _ = self.as_mut();
            self.back_idx -= 1;
            underlying.get_mut(self.back_idx)
        } else {
            None
        }
    }
}

impl<'a, T, const CAP_PER_PAGE: usize> ExactSizeIterator for Iter<'a, T, CAP_PER_PAGE>
where
    T: Sized,
{}

impl<'a, T, const CAP_PER_PAGE: usize> ExactSizeIterator for IterMut<'a, T, CAP_PER_PAGE>
where
    T: Sized,
{}

impl<'a, T, const CAP_PER_PAGE: usize> Iter<'a, T, CAP_PER_PAGE>
where
    T: Sized,
{
    pub(crate) fn new(deque: &'a PinnedDeque<T, CAP_PER_PAGE>) -> Self {
        Self {
            underlying: deque,
            front_idx: 0,
            back_idx: deque.len(),
        }
    }
}

impl<'a, T, const CAP_PER_PAGE: usize> IterMut<'a, T, CAP_PER_PAGE>
where
    T: Sized,
{
    pub(crate) fn new(deque: &'a mut PinnedDeque<T, CAP_PER_PAGE>) -> Self {
        Self {
            underlying: deque,
            front_idx: 0,
            back_idx: deque.len(),
            _ph: PhantomData{},
        }
    }

    fn as_mut(&self) -> &'a mut PinnedDeque<T, CAP_PER_PAGE> {
        unsafe {
            &mut *self.underlying
        }
    }
}

