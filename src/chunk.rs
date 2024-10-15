use std::{
    alloc::{alloc_zeroed, Layout},
    mem::{size_of, MaybeUninit},
};

pub(crate) struct Chunk<T: Sized> {
    pub(crate) start: u32,
    pub(crate) end: u32,
    pub(crate) _elems: [MaybeUninit<T>; 0],
}

impl<T: Sized> Chunk<T> {
    pub(crate) fn capacity_per_chunk() -> u32 {
        let page_size = page_size::get();
        assert!(
            size_of::<Chunk<T>>() + size_of::<T>() <= page_size,
            "size of a single element: {}, size of the overhead: {}",
            size_of::<T>(),
            size_of::<Chunk<T>>(),
        );
        let res = (page_size - size_of::<Chunk<T>>()) / size_of::<T>();
        res as u32
    }

    pub(crate) fn layout(cap_per_chunk: u32) -> Layout {
        assert!(cap_per_chunk < u32::MAX);
        let page_size = page_size::get();
        let chunk_size = size_of::<Chunk<T>>() + size_of::<T>() * (cap_per_chunk as usize);
        Layout::from_size_align(chunk_size, page_size).unwrap()
    }

    pub(crate) fn new(layout: Layout) -> *mut Self {
        unsafe { alloc_zeroed(layout) as *mut Self }
    }

    pub(crate) fn free(ptr: *mut Self, layout: Layout) {
        unsafe {
            std::alloc::dealloc(ptr as *mut u8, layout);
        }
    }

    pub(crate) fn reset_for_front_insertion(&mut self, cap_per_chunk: u32) {
        self.end = cap_per_chunk;
        self.start = cap_per_chunk;
    }

    pub(crate) fn reset_for_back_insertion(&mut self) {
        self.start = 0;
        self.end = 0;
    }

    pub(crate) fn reserve_front(&mut self) -> Option<&mut MaybeUninit<T>> {
        if self.start > 0 {
            let new_start = self.start.wrapping_sub(1);
            self.start = new_start;
            Some(self.inner_get_mut(new_start))
        } else {
            None
        }
    }

    pub(crate) fn reserve_back(&mut self, cap_per_chunk: u32) -> Option<&mut MaybeUninit<T>> {
        if self.end < cap_per_chunk {
            let old_end = self.end;
            self.end = self.end.wrapping_add(1);
            Some(self.inner_get_mut(old_end))
        } else {
            None
        }
    }

    pub(crate) fn len(&self) -> usize {
        let res = self.end - self.start;
        res as usize
    }

    pub(crate) fn get(&self, idx: usize) -> &T {
        debug_assert!(idx < self.len());
        let res = self.inner_get(self.start.wrapping_add(idx as u32));
        unsafe { res.assume_init_ref() }
    }

    pub(crate) fn get_mut(&mut self, idx: usize) -> &mut T {
        debug_assert!(idx < self.len());
        let res = self.inner_get_mut(self.start.wrapping_add(idx as u32));
        unsafe { res.assume_init_mut() }
    }

    pub(crate) fn front(&self) -> &T {
        let res = self.inner_get(self.start);
        unsafe { res.assume_init_ref() }
    }

    pub(crate) fn front_mut(&mut self) -> &mut T {
        let res = self.inner_get_mut(self.start);
        unsafe { res.assume_init_mut() }
    }

    pub(crate) fn back(&self) -> &T {
        debug_assert!(self.end > self.start);
        let res = self.inner_get(self.end.wrapping_sub(1));
        unsafe { res.assume_init_ref() }
    }

    pub(crate) fn back_mut(&mut self) -> &mut T {
        debug_assert!(self.end > self.start);
        let res = self.inner_get_mut(self.end.wrapping_sub(1));
        unsafe { res.assume_init_mut() }
    }

    pub(crate) fn pop_front(&mut self) -> T {
        debug_assert!(self.start < self.end);
        let old_start = self.start;
        self.start = self.start.wrapping_add(1);
        let res = self.inner_get(old_start);
        unsafe { res.assume_init_read() }
    }

    pub(crate) fn pop_back(&mut self) -> T {
        debug_assert!(self.start < self.end);
        let new_end = self.end.wrapping_sub(1);
        self.end = new_end;
        let res = self.inner_get(new_end);
        unsafe { res.assume_init_read() }
    }

    pub(crate) fn drop_all(&mut self) {
        debug_assert!(self.start <= self.end);
        let mut ptr: *mut _ = self.inner_get_mut(self.start);
        let end = ptr.wrapping_add(self.end as usize);
        while ptr < end {
            unsafe {
                let obj: &mut _ = &mut *ptr;
                obj.assume_init_drop();
            }
            ptr = ptr.wrapping_add(1);
        }
    }

    fn inner_get(&self, idx: u32) -> &MaybeUninit<T> {
        let idx = idx as usize;
        let self_ptr_in_u8 = self as *const _ as *const u8;
        let start = {
            let start = self_ptr_in_u8.wrapping_add(size_of::<Self>());
            start as *const MaybeUninit<T>
        };
        unsafe {
            let res = start.add(idx);
            res.as_ref().unwrap_unchecked()
        }
    }

    fn inner_get_mut(&mut self, idx: u32) -> &mut MaybeUninit<T> {
        let idx = idx as usize;
        let self_ptr_in_u8 = self as *mut _ as *mut u8;
        let start = {
            let start = self_ptr_in_u8.wrapping_add(size_of::<Self>());
            start as *mut MaybeUninit<T>
        };
        unsafe {
            let res = start.add(idx);
            res.as_mut().unwrap_unchecked()
        }
    }
}
