use crate::*;

impl<T> Default for PinnedDeque<T>
where
    T: Sized,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> std::fmt::Debug for PinnedDeque<T>
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

impl<T> std::iter::Extend<T> for PinnedDeque<T>
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

impl<'a, T> From<&'a [T]> for PinnedDeque<&'a T>
where
    T: Sized,
{
    fn from(value: &'a [T]) -> Self {
        value.iter().collect()
    }
}

impl<T, const N: usize> From<[T; N]> for PinnedDeque<T>
where
    T: Sized,
{
    fn from(value: [T; N]) -> Self {
        value.into_iter().collect()
    }
}

impl<T> From<Vec<T>> for PinnedDeque<T>
where
    T: Sized,
{
    fn from(value: Vec<T>) -> Self {
        value.into_iter().collect()
    }
}

impl<T> FromIterator<T> for PinnedDeque<T>
where
    T: Sized,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (size_hint, _) = iter.size_hint();
        let mut res = Self::new();
        res.reserve(size_hint);
        for x in iter {
            res.push_back(x);
        }
        res
    }
}

impl<T> Clone for PinnedDeque<T>
where
    T: Sized + Clone,
{
    fn clone(&self) -> Self {
        self.iter().cloned().collect()
    }

    fn clone_from(&mut self, source: &Self) {
        self.clear();
        self.extend(source.iter().cloned());
    }
}
