use crate::*;

impl<T, const CAP_PER_PAGE: usize> Default for PinnedDeque<T, CAP_PER_PAGE>
where
    T: Sized,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const CAP_PER_PAGE: usize> std::fmt::Debug for PinnedDeque<T, CAP_PER_PAGE>
where
    T: Sized + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        let mut it = self.iter();
        if let Some(first) = it.next() {
            write!(f, "{first:?}")?;
            for x in it {
                write!(f, ", {x:?}")?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl<T, const CAP_PER_PAGE: usize> std::iter::Extend<T> for PinnedDeque<T, CAP_PER_PAGE>
where
    T: Sized,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();
        let (size_hint, _) = iter.size_hint();
        self.reserve(size_hint);
        for x in iter {
            self.push_back(x);
        }
    }
}

impl<T, const CAP_PER_PAGE: usize> From<&[T]> for PinnedDeque<T, CAP_PER_PAGE>
where
    T: Sized + Clone,
{
    fn from(value: &[T]) -> Self {
        value.iter().cloned().collect()
    }
}

impl<T, const CAP_PER_PAGE: usize, const N: usize> From<[T; N]> for PinnedDeque<T, CAP_PER_PAGE>
where
    T: Sized,
{
    fn from(value: [T; N]) -> Self {
        value.into_iter().collect()
    }
}

impl<T, const CAP_PER_PAGE: usize> From<Vec<T>> for PinnedDeque<T, CAP_PER_PAGE>
where
    T: Sized,
{
    fn from(value: Vec<T>) -> Self {
        value.into_iter().collect()
    }
}

impl<T, const CAP_PER_PAGE: usize> FromIterator<T> for PinnedDeque<T, CAP_PER_PAGE>
where
    T: Sized,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (size_hint, _) = iter.size_hint();
        let mut res = Self::with_capacity(size_hint);
        for x in iter {
            res.push_back(x);
        }
        res
    }
}

impl<T, const CAP_PER_PAGE: usize> Clone for PinnedDeque<T, CAP_PER_PAGE>
where
    T: Sized + Clone,
{
    fn clone(&self) -> Self {
        self.iter().map(|x| x.get_ref().clone()).collect()
    }

    fn clone_from(&mut self, source: &Self) {
        self.clear();
        self.extend(source.iter().map(|x| x.get_ref().clone()));
    }
}

