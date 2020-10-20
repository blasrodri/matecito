use matecito::Matecito;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::prelude::*;

fn generate_distribution(
    mut rng: &mut ThreadRng,
    choices: &[(usize, usize)],
    num_elements: usize,
) -> Vec<(usize, usize)> {
    let mut v = Vec::with_capacity(num_elements);
    for _ in 0..num_elements {
        let asd = choices.choose_weighted(&mut rng, |item| item.1).unwrap();
        let (a, b) = asd;
        v.push((*a, *b));
    }
    v
}

fn criterion_benchmark(c: &mut Criterion) {
    let cache = Matecito::<usize, usize>::new(1000);
    let mut rng = rand::thread_rng();
    let choices: Vec<_> = (1..10_000).zip(1..10_000).collect();

    let mut scenario_put = generate_distribution(&mut rng, &choices, 100_000);
    let mut scenario_get = generate_distribution(&mut rng, &choices, 100_000);

    for (key, value) in scenario_put {
        cache.put(key, value);
    }

    // let mut scenario1 = generate_distribution(&mut rng, &choices, 100_000);
    let mut final_result = vec![];

    c.bench_function("asd", |b| {
        let cache = cache.clone();
        b.iter(|| {
            let scenario_get = scenario_get.clone();
            let cache = cache.clone();
            let result = std::thread::spawn(move || {
                let mut v = vec![];
                for (key, _) in scenario_get {
                    v.push(cache.clone().get(key));
                }
                v
            })
            .join()
            .unwrap();
            let total = result.iter().count();
            let positives = result.iter().filter(|x| x.is_some()).count();
            let result = positives as f64 / total as f64;
            final_result.push(result);
        });
        println!("{:?}", final_result);
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
