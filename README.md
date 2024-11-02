liglicko2
=========

Lichess-flavored Glicko-2 rating system with fractional rating periods and
instant rating updates.

This does not (yet) exactly match the Lichess implementation.
Instead, it's a proof of concept for potential improvements and parameter
tweaks.

See <http://glicko.net/glicko/glicko2.pdf> for a description of the
original Glicko-2 rating system. The following changes have been made:

- Optimized default parameters based on Lichess data. Optimal parameters
  depend on the application, so this will not be ideal for all use cases.
- All rating components are clamped to specific ranges, so that even
  pathological scenarios cannot cause degenerate results.
- Glicko-2 updates ratings in bulk in discrete *rating periods*. Lichess
  instead updates pairs of ratings, so that ratings can be immediately
  updated after each game.
- Lichess keeps the time decay of rating deviations, but generalizes it
  to work with fractional rating periods.
- Allows considering an inherent advantage for the first player in a game.

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
