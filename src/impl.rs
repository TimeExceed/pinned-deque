use crate::*;
use std::{pin::Pin, collections::VecDeque, mem::*};

pub struct PinnedDeque<T: Sized, const CAP_PER_PAGE: usize> {
    size: usize,
    used: VecDeque<Box<Page<T, CAP_PER_PAGE>>>,
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

    pub fn pop_back(&mut self) -> bool {
        if let Some(last_page) = self.used.back_mut() {
            last_page.pop_back();
            if last_page.len() == 0 {
                let last_page = self.used.pop_back().unwrap();
                self.recycle(last_page);
            }
            self.size -= 1;
            true
        } else {
            false
        }
    }

    pub fn pop_front(&mut self) -> bool {
        if let Some(first_page) = self.used.front_mut() {
            first_page.pop_front();
            if first_page.len() == 0 {
                let first_page = self.used.pop_front().unwrap();
                self.recycle(first_page);
            }
            self.size -= 1;
            true
        } else {
            false
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
        self.used.front_mut().map(|first_page| first_page.front_mut())
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
            let used = unsafe {
                &mut *used
            };
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
            let used = unsafe {
                &mut *used
            };
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
    T: Sized
{
    fn drop(&mut self) {
        self.clear();
    }
}

struct Page<T: Sized, const CAP_PER_PAGE: usize> {
    start: *mut MaybeUninit<T>,
    end: *mut MaybeUninit<T>,
    elems: [MaybeUninit<T>; CAP_PER_PAGE],
}

impl<T, const CAP_PER_PAGE: usize> Page<T, CAP_PER_PAGE>
where
    T: Sized,
{
    fn new() -> Box<Self> {
        let mut res = Box::new(Self {
            start: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
            elems: unsafe { MaybeUninit::uninit().assume_init() },
        });
        res.reset_for_back_insertion();
        res
    }

    fn reset_for_front_insertion(&mut self) {
        let last_ptr = self.last_ptr() as *mut _;
        self.start = last_ptr;
        self.end = last_ptr;
    }

    fn reset_for_back_insertion(&mut self) {
        let first_ptr = self.first_ptr() as *mut _;
        self.start = first_ptr;
        self.end = first_ptr;
    }

    fn reserve_front(&mut self) -> Option<&mut MaybeUninit<T>> {
        if self.start > self.first_ptr() as *mut _ {
            unsafe {
                let res = self.start.wrapping_offset(-1);
                self.start = res;
                Some(&mut *res)
            }
        } else {
            None
        }
    }

    fn reserve_back(&mut self) -> Option<&mut MaybeUninit<T>> {
        if self.end < self.last_ptr() as *mut _ {
            unsafe {
                let res = self.end;
                self.end = res.wrapping_add(1);
                Some(&mut *res)
            }
        } else {
            None
        }
    }

    fn first_ptr(&self) -> *const MaybeUninit<T> {
        &self.elems[0]
    }

    fn last_ptr(&self) -> *const MaybeUninit<T> {
        self.first_ptr().wrapping_add(CAP_PER_PAGE)
    }

    fn len(&self) -> usize {
        assert!(self.start <= self.end);
        let res = unsafe {
            self.end.offset_from(self.start)
        };
        res as usize
    }

    fn get(&self, idx: usize) -> Pin<&T> {
        unsafe {
            let res = self.start.wrapping_add(idx);
            assert!(res < self.end);
            let res: &MaybeUninit<T> = &*(res as *const _);
            Pin::new_unchecked(res.assume_init_ref())
        }
    }

    fn get_mut(&mut self, idx: usize) -> Pin<&mut T> {
        unsafe {
            let res = self.start.wrapping_add(idx);
            assert!(res < self.end);
            let res: &mut MaybeUninit<T> = &mut *res;
            Pin::new_unchecked(res.assume_init_mut())
        }
    }

    fn front(&self) -> Pin<&T> {
        assert!(self.end > self.start);
        unsafe {
            let res: &MaybeUninit<T> = &*(self.start as *const _);
            Pin::new_unchecked(res.assume_init_ref())
        }
    }

    fn back(&self) -> Pin<&T> {
        assert!(self.end > self.start);
        unsafe {
            let res = self.end.wrapping_offset(-1);
            let res: &MaybeUninit<T> = &*(res as *const _);
            Pin::new_unchecked(res.assume_init_ref())
        }
    }

    fn front_mut(&mut self) -> Pin<&mut T> {
        assert!(self.end > self.start);
        unsafe {
            let res = &mut *self.start;
            Pin::new_unchecked(res.assume_init_mut())
        }
    }

    fn back_mut(&mut self) -> Pin<&mut T> {
        assert!(self.end > self.start);
        unsafe {
            let res = self.end.wrapping_offset(-1);
            let res = &mut *res;
            Pin::new_unchecked(res.assume_init_mut())
        }
    }

    fn pop_front(&mut self) {
        assert!(self.end > self.start);
        unsafe {
            let res = &mut *self.start;
            self.start = self.start.wrapping_add(1);
            res.assume_init_drop();
        }
    }

    fn pop_back(&mut self) {
        assert!(self.end > self.start);
        unsafe {
            self.end = self.end.wrapping_offset(-1);
            let res = &mut *self.end;
            res.assume_init_drop();
        }
    }

    fn front_capacity(&self) -> usize {
        let first_ptr = self.first_ptr() as *mut _;
        assert!(first_ptr <= self.start);
        unsafe {
            self.start.offset_from(first_ptr) as usize
        }
    }

    fn back_capacity(&self) -> usize {
        let last_ptr = self.last_ptr() as *mut _;
        assert!(self.end <= last_ptr);
        unsafe {
            last_ptr.offset_from(self.end) as usize
        }
    }

    fn drop_all(&mut self) {
        assert!(self.start <= self.end);
        let mut ptr = self.start;
        while ptr < self.end {
            unsafe {
                let obj: &mut MaybeUninit<T> = &mut *ptr;
                obj.assume_init_drop();
            }
            ptr = ptr.wrapping_add(1);
        }
    }
}
