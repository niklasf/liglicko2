liglicko2
=========

Lichess-flavored Glicko-2 rating system.

Documentation
-------------

https://docs.rs/liglicko2

Example
-------

```rust
use liglicko2::{RatingSystem, Score, Instant, Periods};

let system = RatingSystem::new();

let alice = system.new_rating();
let bob = system.new_rating();

let now = Instant::default() + Periods(2.3);

// Initial prediction is indifferent.
let expected_score = system.expected_score(&alice, &bob, now);
assert!(Score(0.49) < expected_score && expected_score < Score(0.51));

// Alice wins. Update ratings.
let (alice, bob) = system.update_ratings(&alice, &bob, Score::WIN, now).unwrap();
assert!(alice.rating > bob.rating);

let now = now + Periods(1.0);

// Alice is expected to win the next game.
let expected_score = system.expected_score(&alice, &bob, now);
assert!(Score(0.84) < expected_score);
```

License
-------

`liglicko2` is licensed under MIT or APACHE-2.0, at your option.
