use fraction::Fraction;
use std::error::Error;
use std::fmt;

// use rand::Rng;
// https://docs.rs/rand/0.8.3/rand/rngs/index.html

/// Represents an item that we want to chose with a given weight. The
/// weight provided can be arbitrary, needless to say larger means more
/// likely. This value will be normalized in relation to the other weights
/// when provided to the `Alias::from_weighted_tuples`.
#[derive(Debug)]
pub struct WeightedTuple<T: Copy> {
    weight: u64,
    value: T,
}

impl<T: Copy> WeightedTuple<T> {
    /// Initializes a new and immutable `WeightedTuple`
    pub fn new(weight: u64, value: T) -> WeightedTuple<T> {
        WeightedTuple {
            weight: weight,
            value: value,
        }
    }
}

#[derive(Debug)]
struct NormalizedWeightTuple<T> {
    fractional_weight: Fraction,
    value: T,
}

/*
https://www.keithschwarz.com/darts-dice-coins/

Algorithm: Vose's Alias Method

Initialization:
    Create arrays Alias and Prob, each of size n.
    Create two worklists, Small and Large.
    Multiply each probability by n.
    For each scaled probability pi:
        If pi<1, add i to Small.
        Otherwise (pi≥1), add i to Large.
    While Small and Large are not empty: (Large might be emptied first)
        Remove the first element from Small; call it l.
        Remove the first element from Large; call it g.
        Set Prob[l]=pl.
        Set Alias[l]=g.
        Set pg:=(pg+pl)−1. (This is a more numerically stable option.)
        If pg<1, add g to Small.
            Otherwise (pg≥1), add g to Large.
    While Large is not empty:
        Remove the first element from Large; call it g.
        Set Prob[g]=1.
    While Small is not empty: This is only possible due to numerical instability.
        Remove the first element from Small; call it l.
        Set Prob[l]=1.
Generation:
    Generate a fair die roll from an n-sided die; call the side i.
    Flip a biased coin that comes up heads with probability Prob[i].
    If the coin comes up "heads," return i.
    Otherwise, return Alias[i].
*/

#[derive(Debug)]
pub struct Alias<T: Copy> {
    probabilities: Vec<NormalizedWeightTuple<T>>,
    aliases: Vec<T>,
}

#[derive(Debug, Clone)]
pub struct AliasCreationError {
    message: String,
}

impl Error for AliasCreationError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for AliasCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // write!(f, "problem occured while creating alias: ")?;
        write!(f, "{}", self.message)?;
        Ok(())
    }
}

impl<T: Copy> Alias<T> {
    pub fn from_weighted_tuples(
        items: &[WeightedTuple<T>],
    ) -> Result<Alias<T>, AliasCreationError> {
        // We have to scale all the weights, which involves converting them
        // into fractions in reference to all the other weights for context.
        let count = items.len() as u64;
        if count == 0 {
            return Err(AliasCreationError {
                message: String::from("no weighted tuples were provided"),
            });
        }
        let sum = items
            .iter()
            .map(|wt| wt.weight)
            .fold(0, |total, next| total + next);
        let normalized_weight_tuples = items.iter().map(|wt| NormalizedWeightTuple {
            fractional_weight: Fraction::new(wt.weight * count, sum),
            value: wt.value,
        });
        // This will be our finallized results
        let mut finalized_probabilities: Vec<NormalizedWeightTuple<T>> = Vec::new();
        let mut finalized_aliases: Vec<T> = Vec::new();
        // Now we need to partition the large and small probabilities. The large aliases
        // are spread across multiple "buckets" increasing their odds of being selected.
        let mut small_items = Vec::new();
        let mut large_items = Vec::new();
        let one = Fraction::new(1u64, 1u64);
        for item in normalized_weight_tuples {
            if item.fractional_weight < one {
                small_items.push(item);
            } else {
                large_items.push(item);
            }
        }
        // Pop items until one of the collections is fully consumed
        while !small_items.is_empty() && !large_items.is_empty() {
            let current_small_item = small_items.pop().unwrap();
            let current_large_item = large_items.pop().unwrap();
            finalized_probabilities.push(current_small_item);
            finalized_aliases.push(current_large_item.value);
            let reduced_fraction =
                current_large_item.fractional_weight + current_large_item.fractional_weight - one;
            let reduced_large_item = NormalizedWeightTuple {
                fractional_weight: reduced_fraction,
                value: current_large_item.value,
            };
            if reduced_fraction < one {
                small_items.push(reduced_large_item);
            } else {
                large_items.push(reduced_large_item);
            }
        }

        while !large_items.is_empty() {
            let current_large_item = large_items.pop().unwrap();
            finalized_probabilities.push(NormalizedWeightTuple {
                fractional_weight: one,
                value: current_large_item.value,
            });
        }

        Ok(Alias::<T> {
            probabilities: finalized_probabilities,
            aliases: finalized_aliases,
        })
    }

    pub fn select(&self, rng: &mut impl rand::Rng) -> T {
        let random_values: (usize, f32) = rng.gen();
        let probability_index = random_values.0 % &self.probabilities.len();
        let current_probability = &self.probabilities[probability_index];

        let random_probability = Fraction::from(random_values.1);
        if random_probability <= current_probability.fractional_weight {
            return current_probability.value;
        } else {
            return self.aliases[probability_index];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fraction::Fraction;
    use std::vec;
    use rand::thread_rng;

    #[test]
    fn tuples_must_be_present() {
      let empty: &[WeightedTuple<&str>] = &[];
      let no_tuples_alias = Alias::from_weighted_tuples(empty);
      match no_tuples_alias {
        Ok(_) => panic!("an empty set of tuples should fail, but didn't"),
        Err(e) => {
          assert_eq!(format!("{}", e), "no weighted tuples were provided");
        },
      }
    }

    #[test]
    fn can_select_for_singular_item() {
      type StringProducer = fn() -> String;
      let alias_result = Alias::from_weighted_tuples(&[
        WeightedTuple::new(1, (|| String::from("Paul Freakn Baker")) as StringProducer),
      ]);
      if let Err(e) = alias_result {
        panic!("An error occured, but should not have: {}", e);
      }
      let alias = alias_result.unwrap();
      let mut rng = thread_rng();
      let producer: StringProducer = alias.select(&mut rng);
      assert_eq!(String::from("Paul Freakn Baker"), producer());
    }

    #[test]
    fn closure_properly_executed() {
      type SideEffectingFunction = fn() -> ();
      let mut was_mutated = false;
      let alias_result = Alias::from_weighted_tuples(&[
        WeightedTuple::new(1, (|| was_mutated = true) as SideEffectingFunction),
      ]);
      if let Err(e) = alias_result {
        panic!("An error occured, but should not have: {}", e);
      }
      let alias = alias_result.unwrap();
      let mut rng = thread_rng();
      let has_side_effects: SideEffectingFunction = alias.select(&mut rng);
      has_side_effects();
      assert_eq!(was_mutated, true);
    }

    #[test]
    fn getting_familiarity() {
        let tuple = WeightedTuple {
            weight: 123,
            value: &String::from("Paul Freakn Baker"),
        };

        assert_eq!(tuple.weight, 123);
        assert_eq!(tuple.value, &String::from("Paul Freakn Baker"));
        assert_eq!(
            format!("{:?}", tuple),
            String::from("WeightedTuple { weight: 123, value: \"Paul Freakn Baker\" }")
        );
    }

    #[test]
    fn can_we_use_fractions() {
        let items = vec![1u64, 2u64];
        let sum = items.iter().fold(0, |total, next| total + next);
        let fractions: Vec<Fraction> = items
            .iter()
            .map(|&numerator| Fraction::new(numerator, sum))
            .collect();
        assert_eq!(fractions[0], Fraction::new(1u64, 3u64));
        assert_eq!(fractions[1], Fraction::new(2u64, 3u64));

        assert_eq!(Fraction::new(1u64, 2u64), Fraction::new(2u64, 4u64));
    }
}
