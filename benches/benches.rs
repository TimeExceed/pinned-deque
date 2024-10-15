// Because blist fails to build since rust 1.79.0,
// benchmarks about it is removed.
use criterion::*;
use pinned_deque::*;
use std::{collections::*, time::*};

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

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
        let n = *n as u64;
        group.bench_function(BenchmarkId::new("PinnedDeque", n), |b| {
            b.iter_custom(|iters| {
                let mut res = Duration::ZERO;
                for _ in 0..iters {
                    let mut trial = PinnedDeque::<u64>::new();
                    let start = Instant::now();
                    for x in 0..n {
                        trial.push_back(x);
                    }
                    res += start.elapsed();
                }
                res
            })
        });
        group.bench_function(BenchmarkId::new("VecDeque", n), |b| {
            b.iter_custom(|iters| {
                let mut res = Duration::ZERO;
                for _ in 0..iters {
                    let mut trial = VecDeque::<u64>::new();
                    let start = Instant::now();
                    for x in 0..n {
                        trial.push_back(x);
                    }
                    res += start.elapsed();
                }
                res
            })
        });
        group.bench_function(BenchmarkId::new("Vec", n), |b| {
            b.iter_custom(|iters| {
                let mut res = Duration::ZERO;
                for _ in 0..iters {
                    let mut trial = Vec::<u64>::new();
                    let start = Instant::now();
                    for x in 0..n {
                        trial.push(x);
                    }
                    res += start.elapsed();
                }
                res
            })
        });
        if n <= 1_000_000 {
            group.bench_function(BenchmarkId::new("rblist", n), |b| {
                b.iter_custom(|iters| {
                    let mut res = Duration::ZERO;
                    for _ in 0..iters {
                        let mut trial = rblist::BList::<u64>::new(rblist::Scale::Huge);
                        let start = Instant::now();
                        for x in 0..n {
                            trial.push_back(x).unwrap();
                        }
                        res += start.elapsed();
                    }
                    res
                })
            });
        }
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
        let n = *n as u64;
        group.bench_function(BenchmarkId::new("PinnedDeque", n), |b| {
            b.iter_custom(|iters| {
                let mut res = Duration::ZERO;
                for _ in 0..iters {
                    let mut trial = PinnedDeque::<u64>::new();
                    let start = Instant::now();
                    for _ in 0..n {
                        trial.push_front(0);
                    }
                    res += start.elapsed();
                }
                res
            })
        });
        group.bench_function(BenchmarkId::new("VecDeque", n), |b| {
            b.iter_custom(|iters| {
                let mut res = Duration::ZERO;
                for _ in 0..iters {
                    let mut trial = VecDeque::<u64>::new();
                    let start = Instant::now();
                    for _ in 0..n {
                        trial.push_front(0);
                    }
                    res += start.elapsed();
                }
                res
            })
        });
        if n <= 1_000_000 {
            group.bench_function(BenchmarkId::new("rblist", n), |b| {
                b.iter_custom(|iters| {
                    let mut res = Duration::ZERO;
                    for _ in 0..iters {
                        let mut trial = rblist::BList::<u64>::new(rblist::Scale::Huge);
                        let start = Instant::now();
                        for _ in 0..n {
                            trial.push_front(0).unwrap();
                        }
                        res += start.elapsed();
                    }
                    res
                })
            });
        }
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
        let pinned: PinnedDeque<u64> = (0..*n).map(|x| x as u64).collect();
        let vecdeque: VecDeque<u64> = (0..*n).map(|x| x as u64).collect();
        let vec: Vec<u64> = (0..*n).map(|x| x as u64).collect();
        let mid_idx = *n / 2;
        group.bench_function(BenchmarkId::new("PinnedDeque", n), |b| {
            b.iter(|| {
                black_box(pinned.get(mid_idx));
            })
        });
        group.bench_function(BenchmarkId::new("VecDeque", n), |b| {
            b.iter(|| {
                black_box(vecdeque.get(mid_idx));
            })
        });
        group.bench_function(BenchmarkId::new("Vec", n), |b| {
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
        let pinned: PinnedDeque<u64> = (0..*n).map(|x| x as u64).collect();
        let vecdeque: VecDeque<u64> = (0..*n).map(|x| x as u64).collect();
        let vec: Vec<u64> = (0..*n).map(|x| x as u64).collect();
        let rblist = {
            let mut rblist = rblist::BList::<u64>::new(rblist::Scale::Huge);
            for x in 0..*n {
                rblist.push_back(x as u64).unwrap();
            }
            rblist
        };
        group.bench_function(BenchmarkId::new("PinnedDeque", n), |b| {
            b.iter(|| {
                for x in pinned.iter() {
                    black_box(x);
                }
            })
        });
        group.bench_function(BenchmarkId::new("VecDeque", n), |b| {
            b.iter(|| {
                for x in vecdeque.iter() {
                    black_box(x);
                }
            })
        });
        group.bench_function(BenchmarkId::new("Vec", n), |b| {
            b.iter(|| {
                for x in vec.iter() {
                    black_box(x);
                }
            })
        });
        group.bench_function(BenchmarkId::new("rblist", n), |b| {
            b.iter(|| {
                for x in rblist.iter() {
                    black_box(x.value().unwrap());
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
        let pinned: PinnedDeque<u64> = (0..*n).map(|x| x as u64).collect();
        let vecdeque: VecDeque<u64> = (0..*n).map(|x| x as u64).collect();
        let vec: Vec<u64> = (0..*n).map(|x| x as u64).collect();
        group.bench_function(BenchmarkId::new("PinnedDeque", n), |b| {
            b.iter(|| {
                for x in pinned.iter().rev() {
                    black_box(x);
                }
            })
        });
        group.bench_function(BenchmarkId::new("VecDeque", n), |b| {
            b.iter(|| {
                for x in vecdeque.iter().rev() {
                    black_box(x);
                }
            })
        });
        group.bench_function(BenchmarkId::new("Vec", n), |b| {
            b.iter(|| {
                for x in vec.iter().rev() {
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
