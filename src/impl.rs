use crate::*;
use std::{collections::VecDeque, mem::*, pin::Pin};

pub struct PinnedDeque<T: Sized, const CAP_PER_PAGE: usize> {
    size: usize,
    pub(crate) used: VecDeque<Box<Page<T, CAP_PER_PAGE>>>,
    freed: Vec<Box<Page<T, CAP_PER_PAGE>>>,
}

impl<T, const CAP_PER_PAGE: usize> PinnedDeque<T, CAP_PER_PAGE>
where
    T: Sized,
{
    pub const PAGE_SIZE: usize = size_of::<*mut T>() * 2 + size_of::<T>() * CAP_PER_PAGE;

    pub const fn capacity_per_page(page_size: usize) -> usize {
        assert!(page_size >= size_of::<T>() + size_of::<*mut T>() * 2);
        (page_size - size_of::<*mut T>() * 2) / size_of::<T>()
    }

    pub fn new() -> Self {
        Self {
            size: 0,
            used: VecDeque::new(),
            freed: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut res = Self::new();
        res.reserve(capacity);
        res
    }

    pub fn reserve(&mut self, additional: usize) {
        let n = (additional + CAP_PER_PAGE - 1) / CAP_PER_PAGE;
        if n > self.freed.len() {
            for _ in self.freed.len()..n {
                self.freed.push(Page::new());
            }
        }
        assert!(n <= self.freed.len());
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {
        let mut res = self.freed.len() * CAP_PER_PAGE;
        if let Some(first_page) = self.used.front().as_ref() {
            res += first_page.front_capacity();
        }
        if let Some(last_page) = self.used.back().as_ref() {
            res += last_page.back_capacity();
        }
        res
    }

    pub fn push_back(&mut self, elem: T) {
        self.size += 1;
        if let Some(last_page) = self.used.back_mut() {
            if let Some(slot) = last_page.reserve_back() {
                slot.write(elem);
                return;
            }
        }
        let mut new_page = self.fetch_a_freed_page();
        new_page.reset_for_back_insertion();
        new_page.reserve_back().unwrap().write(elem);
        self.used.push_back(new_page);
    }

    pub fn push_front(&mut self, elem: T) {
        self.size += 1;
        if let Some(first_page) = self.used.front_mut() {
            if let Some(slot) = first_page.reserve_front() {
                slot.write(elem);
                return;
            }
        }
        let mut new_page = self.fetch_a_freed_page();
        new_page.reset_for_front_insertion();
        new_page.reserve_front().unwrap().write(elem);
        self.used.push_front(new_page);
    }

    pub fn pop_back(&mut self) -> Option<()> {
        if let Some(last_page) = self.used.back_mut() {
            last_page.pop_back();
            if last_page.len() == 0 {
                let last_page = self.used.pop_back().unwrap();
                self.recycle(last_page);
            }
            self.size -= 1;
            Some(())
        } else {
            None
        }
    }

    pub fn pop_front(&mut self) -> Option<()> {
        if let Some(first_page) = self.used.front_mut() {
            first_page.pop_front();
            if first_page.len() == 0 {
                let first_page = self.used.pop_front().unwrap();
                self.recycle(first_page);
            }
            self.size -= 1;
            Some(())
        } else {
            None
        }
    }

    pub fn back(&self) -> Option<Pin<&T>> {
        self.used.back().map(|last_page| last_page.back())
    }

    pub fn front(&self) -> Option<Pin<&T>> {
        self.used.front().map(|first_page| first_page.front())
    }

    pub fn back_mut(&mut self) -> Option<Pin<&mut T>> {
        self.used.back_mut().map(|last_page| last_page.back_mut())
    }

    pub fn front_mut(&mut self) -> Option<Pin<&mut T>> {
        self.used
            .front_mut()
            .map(|first_page| first_page.front_mut())
    }

    pub fn clear(&mut self) {
        while let Some(mut page) = self.used.pop_front() {
            page.drop_all();
            self.recycle(page);
        }
    }

    pub fn get(&self, mut idx: usize) -> Option<Pin<&T>> {
        if idx >= self.len() {
            return None;
        }
        let first_page = self.used.front().unwrap();
        if idx < first_page.len() {
            return Some(first_page.get(idx));
        } else {
            idx -= first_page.len();
        }
        let n = idx / CAP_PER_PAGE;
        let offset = idx % CAP_PER_PAGE;
        let target_page = &self.used[n + 1];
        Some(target_page.get(offset))
    }

    pub fn get_mut(&mut self, mut idx: usize) -> Option<Pin<&mut T>> {
        if idx >= self.len() {
            return None;
        }
        let used: *mut _ = &mut self.used;
        {
            let used = unsafe { &mut *used };
            let first_page = used.front_mut().unwrap();
            if idx < first_page.len() {
                return Some(first_page.get_mut(idx));
            } else {
                idx -= first_page.len();
            }
        }
        let n = idx / CAP_PER_PAGE;
        let offset = idx % CAP_PER_PAGE;
        let target_page = {
            let used = unsafe { &mut *used };
            &mut used[n + 1]
        };
        Some(target_page.get_mut(offset))
    }

    pub fn iter(&self) -> Iter<'_, T, CAP_PER_PAGE> {
        Iter::new(self)
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T, CAP_PER_PAGE> {
        IterMut::new(self)
    }

    fn fetch_a_freed_page(&mut self) -> Box<Page<T, CAP_PER_PAGE>> {
        if let Some(page) = self.freed.pop() {
            page
        } else {
            Page::new()
        }
    }

    fn recycle(&mut self, page: Box<Page<T, CAP_PER_PAGE>>) {
        self.freed.push(page);
    }
}

impl<T, const CAP_PER_PAGE: usize> Drop for PinnedDeque<T, CAP_PER_PAGE>
where
    T: Sized,
{
    fn drop(&mut self) {
        self.clear();
    }
}
