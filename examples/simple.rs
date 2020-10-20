/// This is a very naive example, that aims to see what's the cache hit/miss ratio.
/// Build a random weighted sequence, and feed it to the cache
/// With another random weigthed sequence, check how many hits there are.
use matecito::Matecito;

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

fn main() {
    let cache = Matecito::<usize, usize>::new(3000);
    let mut rng = rand::thread_rng();
    let choices: Vec<_> = (1..10_000).zip(1..10_000).collect();

    println!("Building scenarios...");
    let scenario_put = generate_distribution(&mut rng, &choices, 10_000);
    let scenario_get = generate_distribution(&mut rng, &choices, 20_000);

    println!("Starting with the benchmark.");
    for (key, value) in scenario_put.clone() {
        cache.put(key, value);
    }

    let mut final_result = vec![];

    let cache = cache.clone();
    let scenario_get = scenario_get.clone();
    let cache = cache.clone();
    let result = std::thread::spawn(move || {
        let mut v = vec![];
        for (key, _) in scenario_get {
            let r = cache.clone().get(key);
            let (key_put, value_put) = scenario_put[key];
            cache.put(key_put, value_put);
            v.push(r);
        }
        v
    })
    .join()
    .unwrap();
    let total = result.iter().count();
    let hits = result.iter().filter(|x| x.is_some()).count();
    let result = hits as f64 / total as f64;
    final_result.push(result);
    println!("Hit/Miss ratio is {:02}", final_result[0] * 100f64);
}
