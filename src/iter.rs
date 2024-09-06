use crate::*;
use std::{iter::*, marker::PhantomData, pin::Pin, collections::*, mem::*, ptr};

#[derive(Clone)]
pub struct Iter<'a, T: Sized, const CAP_PER_PAGE: usize> {
    size: usize,
    page_iter: vec_deque::Iter<'a, Box<Page<T, CAP_PER_PAGE>>>,
    front_page: *const Page<T, CAP_PER_PAGE>,
    front_elem: *const MaybeUninit<T>,
    back_page: *const Page<T, CAP_PER_PAGE>,
    back_elem: *const MaybeUninit<T>,
}

pub struct IterMut<'a, T: Sized, const CAP_PER_PAGE: usize> {
    size: usize,
    page_iter: vec_deque::IterMut<'a, Box<Page<T, CAP_PER_PAGE>>>,
    front_page: *mut Page<T, CAP_PER_PAGE>,
    front_elem: *mut MaybeUninit<T>,
    back_page: *mut Page<T, CAP_PER_PAGE>,
    back_elem: *mut MaybeUninit<T>,
    _ph: PhantomData<&'a mut PinnedDeque<T, CAP_PER_PAGE>>,
}

impl<'a, T, const CAP_PER_PAGE: usize> Iterator for Iter<'a, T, CAP_PER_PAGE>
where
    T: Sized,
{
    type Item = Pin<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.size == 0 {
            None
        } else {
            self.size -= 1;
            if self.front_page.is_null() || self.front_elem == unsafe {&*self.front_page}.end as *const _ {
                self.front_page = if let Some(page) = self.page_iter.next() {
                    page.as_ref()
                } else {
                    self.back_page
                };
                debug_assert!(!self.front_page.is_null());
                self.front_elem = unsafe {
                    let front_page = &*self.front_page;
                    front_page.start as *const MaybeUninit<T>
                };
            }
            debug_assert!(self.front_page != self.back_page || self.front_elem < self.back_elem);
            let front_elem = self.front_elem;
            self.front_elem = front_elem.wrapping_add(1);
            unsafe {
                let res: &MaybeUninit<T> = &*front_elem;
                Some(Pin::new_unchecked(res.assume_init_ref()))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}

impl<'a, T, const CAP_PER_PAGE: usize> Iterator for IterMut<'a, T, CAP_PER_PAGE>
where
    T: Sized,
{
    type Item = Pin<&'a mut T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.size == 0 {
            None
        } else {
            self.size -= 1;
            if self.front_page.is_null() || self.front_elem == unsafe {&*self.front_page}.end {
                self.front_page = if let Some(page) = self.page_iter.next() {
                    page.as_mut()
                } else {
                    self.back_page
                };
                debug_assert!(!self.front_page.is_null());
                self.front_elem = unsafe {
                    let front_page = &*self.front_page;
                    front_page.start
                };
            }
            debug_assert!(self.front_page != self.back_page || self.front_elem < self.back_elem);
            let front_elem = self.front_elem;
            self.front_elem = front_elem.wrapping_add(1);
            unsafe {
                let res: &mut MaybeUninit<T> = &mut *front_elem;
                Some(Pin::new_unchecked(res.assume_init_mut()))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}

impl<'a, T, const CAP_PER_PAGE: usize> DoubleEndedIterator for Iter<'a, T, CAP_PER_PAGE>
where
    T: Sized,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.size == 0 {
            None
        } else {
            self.size -= 1;
            if self.back_page.is_null() || unsafe {&*self.back_page}.start as *const _ == self.back_elem {
                self.back_page = if let Some(page) = self.page_iter.next_back() {
                    page.as_ref()
                } else {
                    self.front_page
                };
                debug_assert!(!self.back_page.is_null());
                self.back_elem = unsafe {
                    let back_page = &*self.back_page;
                    back_page.end as *const _
                };
            }
            debug_assert!(self.front_page != self.back_page || self.front_elem < self.back_elem);
            self.back_elem = self.back_elem.wrapping_offset(-1);
            unsafe {
                let res: &MaybeUninit<T> = &*self.back_elem;
                Some(Pin::new_unchecked(res.assume_init_ref()))
            }
        }
    }
}

impl<'a, T, const CAP_PER_PAGE: usize> DoubleEndedIterator for IterMut<'a, T, CAP_PER_PAGE>
where
    T: Sized,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.size == 0 {
            None
        } else {
            self.size -= 1;
            if self.back_page.is_null() || unsafe {&*self.back_page}.start == self.back_elem {
                self.back_page = if let Some(page) = self.page_iter.next_back() {
                    page.as_mut()
                } else {
                    self.front_page
                };
                debug_assert!(!self.back_page.is_null());
                self.back_elem = unsafe {
                    let back_page: &mut _ = &mut *self.back_page;
                    back_page.end
                };
            }
            debug_assert!(self.front_page != self.back_page || self.front_elem < self.back_elem);
            self.back_elem = self.back_elem.wrapping_offset(-1);
            unsafe {
                let res: &mut _ = &mut *self.back_elem;
                Some(Pin::new_unchecked(res.assume_init_mut()))
            }
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
            size: deque.len(),
            page_iter: deque.used.iter(),
            front_page: ptr::null(),
            front_elem: ptr::null(),
            back_page: ptr::null(),
            back_elem: ptr::null(),
        }
    }
}

impl<'a, T, const CAP_PER_PAGE: usize> IterMut<'a, T, CAP_PER_PAGE>
where
    T: Sized,
{
    pub(crate) fn new(deque: &'a mut PinnedDeque<T, CAP_PER_PAGE>) -> Self {
        Self {
            size: deque.len(),
            page_iter: deque.used.iter_mut(),
            front_page: ptr::null_mut(),
            front_elem: ptr::null_mut(),
            back_page: ptr::null_mut(),
            back_elem: ptr::null_mut(),
            _ph: PhantomData{},
        }
    }
}

