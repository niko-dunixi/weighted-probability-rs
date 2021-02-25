extern crate weighted_probability_rs;
use weighted_probability_rs::*;
use rand::{thread_rng};

#[derive(Debug, Clone, Copy)]
enum Marble {
    Red,
    Blue,
}

fn main() {
    let marble_bag = Alias::from_weighted_tuples(&[
        WeightedTuple::new(1, Marble::Red),
        WeightedTuple::new(2, Marble::Blue),
    ]);
    let mut rng = thread_rng();
    println!("{:?}", marble_bag.select(&mut rng));
}