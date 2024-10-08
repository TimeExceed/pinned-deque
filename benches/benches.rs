use criterion::*;
use pinned_deque::*;
use std::{collections::*, time::*};

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const CAP_PER_PAGE: usize = 1022; // For a 4KB page, 510 u64's are allowed.

fn push_back(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let mut group = c.benchmark_group("PushBack");
    group.plot_config(plot_config);
    const SIZES: &[usize] = &[
        100usize,
        1_000usize,
        10_000usize,
        100_000usize,
        1_000_000usize,
        10_000_000usize,
    ];
    for n in SIZES.iter() {
        group.bench_with_input(BenchmarkId::new("PinnedDeque", n), n, |b, i| {
            b.iter_custom(|iters| {
                let mut res = Duration::ZERO;
                for _ in 0..iters {
                    let mut trial = PinnedDeque::<u64, CAP_PER_PAGE>::new();
                    let i = *i as u64;
                    let start = Instant::now();
                    for x in 0..i {
                        trial.push_back(x);
                    }
                    res += start.elapsed();
                }
                res
            })
        });
        group.bench_with_input(BenchmarkId::new("VecDeque", n), n, |b, i| {
            b.iter_custom(|iters| {
                let mut res = Duration::ZERO;
                for _ in 0..iters {
                    let mut trial = VecDeque::<u64>::new();
                    let i = *i as u64;
                    let start = Instant::now();
                    for x in 0..i {
                        trial.push_back(x);
                    }
                    res += start.elapsed();
                }
                res
            })
        });
        group.bench_with_input(BenchmarkId::new("Vec", n), n, |b, i| {
            b.iter_custom(|iters| {
                let mut res = Duration::ZERO;
                for _ in 0..iters {
                    let mut trial = Vec::<u64>::new();
                    let i = *i as u64;
                    let start = Instant::now();
                    for x in 0..i {
                        trial.push(x);
                    }
                    res += start.elapsed();
                }
                res
            })
        });
    }
    group.finish();
}

fn push_front(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let mut group = c.benchmark_group("PushFront");
    group.plot_config(plot_config);
    const SIZES: &[usize] = &[
        100usize,
        1_000usize,
        10_000usize,
        100_000usize,
        1_000_000usize,
        10_000_000usize,
    ];
    for n in SIZES.iter() {
        group.bench_with_input(BenchmarkId::new("PinnedDeque", n), n, |b, i| {
            b.iter_custom(|iters| {
                let mut res = Duration::ZERO;
                for _ in 0..iters {
                    let mut trial = PinnedDeque::<u64, CAP_PER_PAGE>::new();
                    let start = Instant::now();
                    for _ in 0..*i {
                        trial.push_front(0);
                    }
                    res += start.elapsed();
                }
                res
            })
        });
        group.bench_with_input(BenchmarkId::new("VecDeque", n), n, |b, i| {
            b.iter_custom(|iters| {
                let mut res = Duration::ZERO;
                for _ in 0..iters {
                    let mut trial = VecDeque::<u64>::new();
                    let start = Instant::now();
                    for _ in 0..*i {
                        trial.push_front(0);
                    }
                    res += start.elapsed();
                }
                res
            })
        });
    }
    group.finish();
}

fn get_mid(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let mut group = c.benchmark_group("GetMiddle");
    group.plot_config(plot_config);
    for n in [
        100usize,
        1_000usize,
        10_000usize,
        100_000usize,
        1_000_000usize,
    ]
    .iter()
    {
        let pinned: PinnedDeque<u64, CAP_PER_PAGE> = (0..*n).map(|x| x as u64).collect();
        let vecdeque: VecDeque<u64> = (0..*n).map(|x| x as u64).collect();
        let vec: Vec<u64> = (0..*n).map(|x| x as u64).collect();
        let mid_idx = *n / 2;
        group.bench_with_input(BenchmarkId::new("PinnedDeque", n), n, |b, _| {
            b.iter(|| {
                black_box(pinned.get(mid_idx));
            })
        });
        group.bench_with_input(BenchmarkId::new("VecDeque", n), n, |b, _| {
            b.iter(|| {
                black_box(vecdeque.get(mid_idx));
            })
        });
        group.bench_with_input(BenchmarkId::new("Vec", n), n, |b, _| {
            b.iter(|| {
                black_box(vec.get(mid_idx));
            })
        });
    }
    group.finish();
}

fn iter(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let mut group = c.benchmark_group("Iter");
    group.plot_config(plot_config);
    for n in [
        100usize,
        1_000usize,
        10_000usize,
        100_000usize,
        1_000_000usize,
    ]
    .iter()
    {
        let pinned: PinnedDeque<u64, CAP_PER_PAGE> = (0..*n).map(|x| x as u64).collect();
        let vecdeque: VecDeque<u64> = (0..*n).map(|x| x as u64).collect();
        let vec: Vec<u64> = (0..*n).map(|x| x as u64).collect();
        group.bench_with_input(BenchmarkId::new("PinnedDeque", n), n, |b, _| {
            b.iter(|| {
                for x in pinned.iter() {
                    black_box(x);
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("VecDeque", n), n, |b, _| {
            b.iter(|| {
                for x in vecdeque.iter() {
                    black_box(x);
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("Vec", n), n, |b, _| {
            b.iter(|| {
                for x in vec.iter() {
                    black_box(x);
                }
            })
        });
    }
    group.finish();
}

fn iter_backwards(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let mut group = c.benchmark_group("IterBack");
    group.plot_config(plot_config);
    for n in [
        100usize,
        1_000usize,
        10_000usize,
        100_000usize,
        1_000_000usize,
    ]
    .iter()
    {
        let pinned: PinnedDeque<u64, CAP_PER_PAGE> = (0..*n).map(|x| x as u64).collect();
        let vecdeque: VecDeque<u64> = (0..*n).map(|x| x as u64).collect();
        let vec: Vec<u64> = (0..*n).map(|x| x as u64).collect();
        group.bench_with_input(BenchmarkId::new("PinnedDeque", n), n, |b, _| {
            b.iter(|| {
                for x in pinned.iter() {
                    black_box(x);
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("VecDeque", n), n, |b, _| {
            b.iter(|| {
                for x in vecdeque.iter() {
                    black_box(x);
                }
            })
        });
        group.bench_with_input(BenchmarkId::new("Vec", n), n, |b, _| {
            b.iter(|| {
                for x in vec.iter() {
                    black_box(x);
                }
            })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    push_back,
    push_front,
    get_mid,
    iter,
    iter_backwards
);
criterion_main!(benches);
