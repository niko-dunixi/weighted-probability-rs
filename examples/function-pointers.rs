extern crate weighted_probability_rs;
use rand::thread_rng;
use std::collections::HashMap;
use std::error::Error;
use std::hash::Hash;
use weighted_probability_rs::*;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
enum Marble {
    Red,
    Blue,
    Green,
}

type MarbleProducer = fn() -> Marble;

fn produce_red() -> Marble {
    Marble::Red
}

fn main() -> Result<(), Box<dyn Error>> {
    let marble_bag: Alias<MarbleProducer> = Alias::from_weighted_tuples(&[
        WeightedTuple::new(1, produce_red as fn() -> Marble),
        WeightedTuple::new(2, || Marble::Blue),
        WeightedTuple::new(3, (|| Marble::Green) as MarbleProducer),
    ])?;
    let mut rng = thread_rng();
    let mut marble_selection_counts = HashMap::new();
    let max_iterations = 1_000_000;
    for _ in 0..max_iterations {
        let marble_producing_function = marble_bag.select(&mut rng);
        let chosen_marble = marble_producing_function();
        marble_selection_counts.entry(chosen_marble).or_insert(0u64);
        marble_selection_counts.insert(chosen_marble, marble_selection_counts[&chosen_marble] + 1);
    }
    println!(
        "Making {} selections resulted in {:?}",
        max_iterations, marble_selection_counts
    );
    Ok(())
}
