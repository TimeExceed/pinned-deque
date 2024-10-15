use crate::{chunk::Chunk, *};
use std::{collections::*, iter::*, ptr};

#[derive(Clone)]
pub struct Iter<'a, T: Sized> {
    size: usize,
    chunk_iter: vec_deque::Iter<'a, *mut Chunk<T>>,
    front_chunk: *const Chunk<T>,
    front_elem: *const T,
    back_chunk: *const Chunk<T>,
    back_elem: *const T,
}

pub struct IterMut<'a, T: Sized> {
    size: usize,
    chunk_iter: vec_deque::IterMut<'a, *mut Chunk<T>>,
    front_chunk: *mut Chunk<T>,
    front_elem: *mut T,
    back_chunk: *mut Chunk<T>,
    back_elem: *mut T,
}

pub struct IntoIter<T: Sized>(PinnedDeque<T>);

impl<'a, T> Iterator for Iter<'a, T>
where
    T: Sized,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.size == 0 {
            None
        } else {
            self.size -= 1;
            let res = unsafe { &*self.front_elem };
            self.front_elem = self.front_elem.wrapping_add(1);
            if self.front_elem > unsafe { &*self.front_chunk }.back() {
                self.front_chunk = if let Some(chunk) = self.chunk_iter.next() {
                    *chunk as *const Chunk<T>
                } else {
                    self.back_chunk
                };
                self.front_elem = unsafe {
                    let front_chunk: &Chunk<T> = &*self.front_chunk;
                    front_chunk.front()
                }
            }
            Some(res)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}

impl<'a, T> Iterator for IterMut<'a, T>
where
    T: Sized,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.size == 0 {
            None
        } else {
            self.size -= 1;
            let res = unsafe { &mut *self.front_elem };

            self.front_elem = self.front_elem.wrapping_add(1);
            if self.front_elem > unsafe { &mut *self.front_chunk }.back_mut() {
                self.front_chunk = if let Some(chunk) = self.chunk_iter.next() {
                    *chunk
                } else {
                    self.back_chunk
                };
                self.front_elem = unsafe {
                    let front_chunk: &mut Chunk<T> = &mut *self.front_chunk;
                    front_chunk.front_mut()
                }
            }

            Some(res)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}

impl<T: Sized> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.0.len();
        (size, Some(size))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
    T: Sized,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.size == 0 {
            None
        } else {
            self.size -= 1;
            let res = unsafe { &*self.back_elem };

            self.back_elem = self.back_elem.wrapping_sub(1);
            if self.back_elem < unsafe { &*self.back_chunk }.front() {
                self.back_chunk = if let Some(chunk) = self.chunk_iter.next_back() {
                    *chunk as *const Chunk<T>
                } else {
                    self.front_chunk
                };
                self.back_elem = unsafe {
                    let back_chunk: &Chunk<T> = &*self.back_chunk;
                    back_chunk.back()
                }
            }

            Some(res)
        }
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T>
where
    T: Sized,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.size == 0 {
            None
        } else {
            self.size -= 1;
            let res = unsafe { &mut *self.back_elem };

            self.back_elem = self.back_elem.wrapping_sub(1);
            if self.back_elem < unsafe { &mut *self.back_chunk }.front_mut() {
                self.back_chunk = if let Some(chunk) = self.chunk_iter.next_back() {
                    *chunk
                } else {
                    self.front_chunk
                };
                self.back_elem = unsafe {
                    let back_chunk: &mut Chunk<T> = &mut *self.back_chunk;
                    back_chunk.back_mut()
                }
            }

            Some(res)
        }
    }
}

impl<T: Sized> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop_back()
    }
}

impl<'a, T: Sized> ExactSizeIterator for Iter<'a, T> {}

impl<'a, T: Sized> ExactSizeIterator for IterMut<'a, T> {}

impl<T: Sized> ExactSizeIterator for IntoIter<T> {}

impl<'a, T> Iter<'a, T>
where
    T: Sized,
{
    pub(crate) fn new(deque: &'a PinnedDeque<T>) -> Self {
        let mut chunk_iter = deque.used.iter();
        if let Some(front_chunk) = chunk_iter.next() {
            let front_chunk = *front_chunk as *const Chunk<T>;
            let front_elem: *const _ = unsafe {
                let front_chunk: &_ = &*front_chunk;
                front_chunk.front()
            };
            let back_chunk: *const _ = if let Some(back_chunk) = chunk_iter.next_back() {
                *back_chunk as *const Chunk<T>
            } else {
                front_chunk
            };
            let back_elem: *const _ = unsafe {
                let back_chunk: &Chunk<T> = &*back_chunk;
                back_chunk.back()
            };
            Self {
                size: deque.len(),
                chunk_iter,
                front_chunk,
                front_elem,
                back_chunk,
                back_elem,
            }
        } else {
            assert_eq!(deque.len(), 0);
            Self {
                size: deque.len(),
                chunk_iter,
                front_chunk: ptr::null(),
                front_elem: ptr::null(),
                back_chunk: ptr::null(),
                back_elem: ptr::null(),
            }
        }
    }
}

impl<'a, T> IterMut<'a, T>
where
    T: Sized,
{
    pub(crate) fn new(deque: &'a mut PinnedDeque<T>) -> Self {
        let size = deque.len();
        let mut chunk_iter = deque.used.iter_mut();
        if let Some(front_chunk) = chunk_iter.next() {
            let front_chunk = *front_chunk;
            let front_elem: *mut _ = unsafe {
                let front_chunk = &mut *front_chunk;
                front_chunk.front_mut()
            };
            let back_chunk = if let Some(back_chunk) = chunk_iter.next_back() {
                *back_chunk
            } else {
                front_chunk
            };
            let back_elem: *mut _ = unsafe {
                let back_chunk = &mut *back_chunk;
                back_chunk.back_mut()
            };
            Self {
                size,
                chunk_iter,
                front_chunk,
                front_elem,
                back_chunk,
                back_elem,
            }
        } else {
            assert_eq!(size, 0);
            Self {
                size,
                chunk_iter,
                front_chunk: ptr::null_mut(),
                front_elem: ptr::null_mut(),
                back_chunk: ptr::null_mut(),
                back_elem: ptr::null_mut(),
            }
        }
    }
}

impl<T: Sized> IntoIterator for PinnedDeque<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

impl<'a, T: Sized> IntoIterator for &'a PinnedDeque<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

impl<'a, T: Sized> IntoIterator for &'a mut PinnedDeque<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut::new(self)
    }
}
