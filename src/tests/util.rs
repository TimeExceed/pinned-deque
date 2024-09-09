#[derive(Debug, Clone)]
pub(super) enum Op {
    PushBack(usize),
    PushFront(usize),
    PopBack,
    PopFront,
}

impl quickcheck::Arbitrary for Op {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        match u8::arbitrary(g) % 4 {
            0 => Self::PushBack(usize::arbitrary(g)),
            1 => Self::PopBack,
            2 => Self::PushFront(usize::arbitrary(g)),
            3 => Self::PopFront,
            _ => unreachable!(),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            Self::PopBack | Self::PopFront => Box::new(std::iter::empty()),
            Op::PushBack(x) => Box::new(x.shrink().map(Self::PushBack)),
            Op::PushFront(x) => Box::new(x.shrink().map(Self::PushFront)),
        }
    }
}
