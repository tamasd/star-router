use criterion::BenchmarkId;
use criterion::Throughput;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use http::Method;
use rand::{distributions::Alphanumeric, Rng};
use star_router::{Route, Router};
use std::iter;
use url::Url;

pub fn empty_lookup_benchmark(c: &mut Criterion) {
    let router: Router<u64> = Router::new(Url::parse("http://example.com").unwrap());
    c.bench_function("not found", |b| {
        b.iter(|| black_box(router.resolve(&Method::GET, "/")))
    });
}

pub fn root_only_lookup_benchmark(c: &mut Criterion) {
    let root_item: u64 = 1;
    let mut router = Router::new(Url::parse("http://example.com").unwrap());
    router
        .add(Route::create("root", Method::GET, "/", root_item).unwrap())
        .unwrap();

    let router = router.optimize();

    c.bench_function("lookup /", |b| {
        b.iter(|| black_box(router.resolve(&Method::GET, "/").unwrap()))
    });
}

pub fn long_static_route_lookup_benchmark(c: &mut Criterion) {
    const ITEM_LEN: usize = 16;
    const MAX: usize = 1024;
    let mut rng = rand::thread_rng();
    let mut router = Router::new(Url::parse("http://example.com").unwrap());
    let items = (0..MAX)
        .map(|_| {
            iter::repeat(())
                .take(ITEM_LEN)
                .map(|_| rng.sample(Alphanumeric))
                .collect()
        })
        .collect::<Vec<String>>();
    for i in 0..MAX {
        let path = items
            .iter()
            .take(i)
            .map(String::from)
            .collect::<Vec<String>>()
            .join("/");

        router
            .add(Route::create(items.get(i).unwrap(), Method::GET, &path, i).unwrap())
            .unwrap();
    }
    let router = router.optimize();

    let mut group = c.benchmark_group("long static route");
    for size in 0..10 {
        let num = (2 << size) as usize;
        group.throughput(Throughput::Elements(num as u64));
        group.bench_with_input(BenchmarkId::from_parameter(num), &num, |b, &i| {
            let path = items
                .iter()
                .take(i)
                .map(String::from)
                .collect::<Vec<String>>()
                .join("/");
            b.iter(|| black_box(router.resolve(&Method::GET, &path)));
        });
    }
}

pub fn wide_static_route_lookup_benchmark(c: &mut Criterion) {
    const ITEM_LEN: usize = 16;
    const MAX: usize = 1024;

    let mut rng = rand::thread_rng();
    let mut router = Router::new(Url::parse("http://example.com").unwrap());

    let items = (0..MAX)
        .map(|_| {
            iter::repeat(())
                .take(ITEM_LEN)
                .map(|_| {
                    let mut path = String::from("/");
                    path.push(rng.sample(Alphanumeric));
                    path
                })
                .collect()
        })
        .collect::<Vec<String>>();

    let _ = items
        .iter()
        .map(|i| {
            router
                .add(Route::create(i, Method::GET, i, String::from(i)).unwrap())
                .unwrap();
        })
        .collect::<Vec<()>>();

    let router = router.optimize();
    let mut group = c.benchmark_group("wide static route");
    for size in 0..10 {
        let num = (2 << size) as usize;
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::from_parameter(num), &num, |b, &i| {
            let path = items.get(i - 1).unwrap();
            b.iter(|| black_box(router.resolve(&Method::GET, &path)));
        });
    }
}

pub fn dynamic_route_lookup_benchmark(c: &mut Criterion) {
    const ITEM_LEN: usize = 16;
    const MAX: usize = 1024;

    let mut rng = rand::thread_rng();
    let mut router = Router::new(Url::parse("http://example.com").unwrap());

    let path = (0..MAX)
        .map(|_| {
            let mut item = String::from(":");
            item.push_str(
                &iter::repeat(())
                    .take(ITEM_LEN)
                    .map(|_| rng.sample(Alphanumeric))
                    .collect::<String>(),
            );
            item
        })
        .collect::<Vec<String>>();

    for i in 1..MAX {
        router
            .add(
                Route::create(
                    &format!("route.{}", i),
                    Method::GET,
                    &(path
                        .iter()
                        .take(i)
                        .map(String::as_str)
                        .collect::<Vec<&str>>()
                        .join("/")),
                    i,
                )
                .unwrap(),
            )
            .unwrap();
    }

    let router = router.optimize();
    let mut group = c.benchmark_group("dynamic route lookup");
    for size in 0..10 {
        let num = (2 << size) as usize;
        let path = iter::repeat(())
            .take(num)
            .map(|_| String::from(rng.sample(Alphanumeric)))
            .collect::<Vec<String>>()
            .join("/");
        group.throughput(Throughput::Elements(num as u64));
        group.bench_with_input(BenchmarkId::from_parameter(num), &num, |b, _| {
            b.iter(|| black_box(router.resolve(&Method::GET, &path)));
        });
    }
}

pub fn wildcard_route_lookup_benchmark(c: &mut Criterion) {
    const ITEM_LEN: usize = 16;
    let mut rng = rand::thread_rng();
    let item: u64 = rng.gen();
    let mut router = Router::new(Url::parse("http://example.com").unwrap());

    router
        .add(Route::create("wildcard", Method::GET, "/*wildcard", item).unwrap())
        .unwrap();

    let router = router.optimize();
    let mut group = c.benchmark_group("wildcard route lookup");
    for size in 0..10 {
        let num = (2 << size) as usize;
        group.throughput(Throughput::Elements(num as u64));
        let path = (0..num)
            .map(|_| {
                iter::repeat(())
                    .take(ITEM_LEN)
                    .map(|_| rng.sample(Alphanumeric))
                    .collect()
            })
            .collect::<Vec<String>>()
            .join("/");
        group.bench_with_input(BenchmarkId::from_parameter(num), &path, |b, _| {
            b.iter(|| black_box(router.resolve(&Method::GET, &path)));
        });
    }
}

criterion_group!(
    benches,
    empty_lookup_benchmark,
    root_only_lookup_benchmark,
    long_static_route_lookup_benchmark,
    wide_static_route_lookup_benchmark,
    dynamic_route_lookup_benchmark,
    wildcard_route_lookup_benchmark,
);
criterion_main!(benches);
