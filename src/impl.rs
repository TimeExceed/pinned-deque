use crate::{chunk::Chunk, *};
use std::{alloc::Layout, collections::VecDeque};

pub struct PinnedDeque<T: Sized> {
    size: usize,
    cap_per_chunk: u32,
    layout: Layout,
    pub(crate) used: VecDeque<*mut Chunk<T>>,
    freed: Vec<*mut Chunk<T>>,
}

impl<T> PinnedDeque<T>
where
    T: Sized,
{
    /// Creates an empty deque with the adaptive capacity per chunk.
    ///
    /// Caveat:
    /// The default capacity per chunk intends to fit a chunk into a memory page.
    /// So, if the size of a single element plus the chunk overhead (8B) exceeds
    /// the size of a memory page, do not use this constructor.
    pub fn new() -> Self {
        let cap_per_chunk = Chunk::<T>::capacity_per_chunk();
        let layout = Chunk::<T>::layout(cap_per_chunk);
        Self {
            size: 0,
            cap_per_chunk,
            layout,
            used: VecDeque::new(),
            freed: Vec::new(),
        }
    }

    /// Creates an empty deque with the given capacity per chunk.
    pub fn with_capacity_per_chunk(cap_per_chunk: u32) -> Self {
        let layout = Chunk::<T>::layout(cap_per_chunk);
        Self {
            size: 0,
            cap_per_chunk,
            layout,
            used: VecDeque::new(),
            freed: Vec::new(),
        }
    }

    /// Reserves additional capacity in order to avoid memory allocations then.
    pub fn reserve(&mut self, additional: usize) {
        let cap_per_chunk = self.cap_per_chunk as usize;
        let n = additional.div_ceil(cap_per_chunk);
        if n > self.freed.len() {
            for _ in self.freed.len()..n {
                self.freed.push(Chunk::<T>::new(self.layout));
            }
        }
        debug_assert!(n <= self.freed.len());
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the total capacity of the deque.
    ///
    /// This deque do not guarantee that pushing elements will not cause memory allocations
    /// even if there is enough free capacity (i.e., `capacity() - len()`).
    pub fn capacity(&self) -> usize {
        (self.used.len() + self.freed.len()) * (self.cap_per_chunk as usize)
    }

    pub fn push_back(&mut self, elem: T) {
        self.size += 1;
        if let Some(back_chunk) = self.used.back() {
            let back_chunk = unsafe { &mut **back_chunk };
            if let Some(slot) = back_chunk.reserve_back(self.cap_per_chunk) {
                slot.write(elem);
                return;
            }
        }
        let new_chunk = self.fetch_a_freed_chunk();
        unsafe {
            let new_chunk = &mut *new_chunk;
            new_chunk.reset_for_back_insertion();
            new_chunk
                .reserve_back(self.cap_per_chunk)
                .unwrap_unchecked()
                .write(elem);
        }
        self.used.push_back(new_chunk);
    }

    pub fn push_front(&mut self, elem: T) {
        self.size += 1;
        if let Some(front_chunk) = self.used.front() {
            let front_chunk = unsafe { &mut **front_chunk };
            if let Some(slot) = front_chunk.reserve_front() {
                slot.write(elem);
                return;
            }
        }
        let new_chunk = self.fetch_a_freed_chunk();
        unsafe {
            let new_chunk = &mut *new_chunk;
            new_chunk.reset_for_front_insertion(self.cap_per_chunk);
            new_chunk.reserve_front().unwrap_unchecked().write(elem);
        }
        self.used.push_front(new_chunk);
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if let Some(back_chunk) = self.used.back() {
            let back_chunk = unsafe { &mut **back_chunk };
            let res = back_chunk.pop_back();
            if back_chunk.len() == 0 {
                let last_chunk = unsafe { self.used.pop_back().unwrap_unchecked() };
                self.recycle(last_chunk);
            }
            self.size -= 1;
            Some(res)
        } else {
            None
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if let Some(front_chunk) = self.used.front() {
            let front_chunk = unsafe { &mut **front_chunk };
            let res = front_chunk.pop_front();
            if front_chunk.len() == 0 {
                let first_chunk = unsafe { self.used.pop_front().unwrap_unchecked() };
                self.recycle(first_chunk);
            }
            self.size -= 1;
            Some(res)
        } else {
            None
        }
    }

    pub fn back(&self) -> Option<&T> {
        self.used.back().map(|back_chunk| {
            let back_chunk = unsafe { &*(*back_chunk as *const Chunk<T>) };
            back_chunk.back()
        })
    }

    pub fn front(&self) -> Option<&T> {
        self.used.front().map(|front_chunk| {
            let front_chunk = unsafe { &*(*front_chunk as *const Chunk<T>) };
            front_chunk.front()
        })
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.used.back().map(|back_chunk| {
            let back_chunk = unsafe { &mut **back_chunk };
            back_chunk.back_mut()
        })
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.used.front().map(|front_chunk| {
            let front_chunk = unsafe { &mut **front_chunk };
            front_chunk.front_mut()
        })
    }

    pub fn clear(&mut self) {
        while let Some(chunk_ptr) = self.used.pop_front() {
            let chunk = unsafe { &mut *chunk_ptr };
            chunk.drop_all();
            self.recycle(chunk_ptr);
        }
    }

    pub fn get(&self, mut idx: usize) -> Option<&T> {
        if idx >= self.len() {
            return None;
        }
        let first_chunk = unsafe { &*(*self.used.front().unwrap_unchecked() as *const Chunk<T>) };
        if idx < first_chunk.len() {
            return Some(first_chunk.get(idx));
        } else {
            idx -= first_chunk.len();
        }
        let n = idx / (self.cap_per_chunk as usize);
        let offset = idx % (self.cap_per_chunk as usize);
        let target_chunk = unsafe { &*(self.used[n + 1] as *const Chunk<T>) };
        Some(target_chunk.get(offset))
    }

    pub fn get_mut(&mut self, mut idx: usize) -> Option<&mut T> {
        if idx >= self.len() {
            return None;
        }
        let front_chunk = unsafe { &mut **self.used.front().unwrap_unchecked() };
        if idx < front_chunk.len() {
            return Some(front_chunk.get_mut(idx));
        } else {
            idx -= front_chunk.len();
        }
        let n = idx / (self.cap_per_chunk as usize);
        let offset = idx % (self.cap_per_chunk as usize);
        let target_chunk = unsafe { &mut *self.used[n + 1] };
        Some(target_chunk.get_mut(offset))
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(self)
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut::new(self)
    }

    fn fetch_a_freed_chunk(&mut self) -> *mut Chunk<T> {
        if let Some(chunk) = self.freed.pop() {
            chunk
        } else {
            Chunk::<T>::new(self.layout)
        }
    }

    fn recycle(&mut self, chunk: *mut Chunk<T>) {
        self.freed.push(chunk);
    }
}

impl<T> Drop for PinnedDeque<T>
where
    T: Sized,
{
    fn drop(&mut self) {
        self.clear();
        while let Some(chunk) = self.freed.pop() {
            Chunk::<T>::free(chunk, self.layout);
        }
    }
}
