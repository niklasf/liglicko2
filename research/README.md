liglicko2 research utilities
============================

Utilities to evaluate rating systems on real-world data.

Why work with such large data sets?
-----------------------------------

Replaying the entire history of Lichess encounters takes a long time, but
I don't know how to avoid it.

* The observed period of time should be long, because optimial rating periods
  may or may not be on the scale of months.
* The observed period of time should be long, because long-term changes of
  rating distributions may be an important effect.
* The observed period should cover important changes, like the introduction
  of quick pairing pools at the start of 2017.
* Its not clear that sampling players does not introduce bias (for example,
  how often players around a specific rating meet).

Prepare or download encounters
------------------------------

Condense PGNs from https://database.lichess.org to CSV files with relevant
data.

```sh
zstdcat lichess_db_standard_rated_*.pgn.zst | cargo run --release --bin pgn_to_encounters > encounters.csv
```

See `sample-encounters.csv` for an example of the output.

Alternatively download standard chess encounters from 2013-01 to 2024-09:

```sh
wget https://database.lichess.org/standard-encounters-until-2024-09.csv.zst # 73G
pzstd -d standard-encounters-until-2024-09.csv.zst # 287G
```

Replay with `glicko2`
---------------------

Replay previously prepared encounters with the third-party `glicko2` crate,
which implements the original Glicko-2.

```sh
cat encounters.csv | cargo run --release --bin replay_vanilla_glicko2
```

Note that the implementation is deliberately unoptimized to avoid
possible bugs.

Replay with `liglicko2`
-----------------------

Replay previously prepared encounters with the `liglicko2` implementation.
See `../README.md` for differences from Glicko-2 and Lichess's current
implementation.

```sh
cat encounters.csv | cargo run --release --bin replay_encounters -- --min-deviation 30,45 --first-advantage 0,11
```

See `cargo run --release -- --help` for more rating system parameters.
All combinations will be simulated, so beware of combinatorial explosion.
Ratings of all players for all experiments for all time controls will be
kept in memory.

Output will look something like this:

```csv
# Parallel experiments: 4
# ---
min_deviation,max_deviation,default_volatility,tau,first_advantage,rating_periods_per_day,avg_deviance
45,500,0.09,0.75,0,0.21436,0.26833
45,500,0.09,0.75,11,0.21436,0.26810
30,500,0.09,0.75,0,0.21436,0.26807
30,500,0.09,0.75,11,0.21436,0.26784
# ---
# Sample Blitz rating of german11: 1510.1 (rd: 30.000, vola: 0.08094)
# ---
# Estimated UltraBullet distribution: p1=NaN p10=NaN p50=NaN p90=NaN p99=NaN, avg=NaN
# Estimated Bullet distribution: p1=763.9 p10=997.9 p50=1321.5 p90=1757.0 p99=2063.8, avg=1355.8
# Estimated Blitz distribution: p1=809.6 p10=1074.1 p50=1375.2 p90=1817.8 p99=2175.8, avg=1422.6
# Estimated Bullet distribution: p1=763.9 p10=997.9 p50=1321.5 p90=1757.0 p99=2063.8, avg=1355.8
# Estimated Classical distribution: p1=966.1 p10=1182.5 p50=1423.6 p90=1872.2 p99=2200.0, avg=1490.5
# Estimated Correspondence distribution: p1=798.0 p10=1191.6 p50=1466.0 p90=1813.7 p99=2142.0, avg=1497.7
# ---
# Distinct players: 284931
# Processed encounters: 18000000 (last at: 2015-03-01 13:43:26)
# Total errors: 0
# ---
```

The most important part is the `avg_deviance` column, which is indicates
the predictive power of the rating system with the given parameters
(lower is better).

PGO
---

Profile-guided optimization can be used to create a faster `./replay_encounters`
binary.

```sh
# Install dependencies
rustup component add llvm-tools-preview
cargo install cargo-pgo

# Check dependencies (bolt not needed)
cargo pgo info

# Build with PGO
./pgo.sh
```
