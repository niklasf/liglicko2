liglicko2 research utilities
============================

Utilities to evaluate rating systems on real-world data.

Encounters
----------

Condense PGNs from https://database.lichess.org to CSV files with relevant
data.

```sh
zstdcat lichess_db_standard_rated_*.pgn.zst | cargo run --release --bin pgn_to_encounters > encounters.csv
```

See `sample-encounters.csv` for an example of the output.

Alternatively download standard chess encounters from 2013-01 to 2024-09:

```sh
wget https://database.lichess.org/standard-encounters-until-2024-09.csv.zst # 287G
```

Replay
------

Replay previously prepared encounters.

```sh
cat encounters.csv | cargo run --release --bin replay_encounters -- --min-deviation 30,45 --first-advantage 0,11
```

See `cargo run --release -- --help` for more rating system parameters.
All combinations will be simulated, so beware of combinatorial explosion.

Output will look something like this:

```
min_deviation,max_deviation,default_volatility,tau,first_advantage,rating_periods_per_day,avg_deviance
30,500,0.09,0.75,11,0,0.28697
30,500,0.09,0.75,11,0.001,0.28696
30,500,0.09,0.75,11,0.05,0.28664
30,500,0.09,0.75,11,0.1,0.28653
30,500,0.09,0.75,11,0.21436,0.28635
30,350,0.09,0.75,11,0,0.28605
30,350,0.09,0.75,11,0.001,0.28605
45,500,0.09,0.75,11,0,0.28591
45,500,0.09,0.75,11,0.001,0.28591
45,500,0.09,0.75,11,0.21436,0.28587
45,500,0.09,0.75,11,0.1,0.28585
45,500,0.09,0.75,11,0.05,0.28585
30,350,0.09,0.75,11,0.05,0.28581
30,350,0.09,0.75,11,0.1,0.28569
30,350,0.09,0.75,11,0.21436,0.28549
45,350,0.09,0.75,11,0,0.28526
45,350,0.09,0.75,11,0.001,0.28526
45,350,0.09,0.75,11,0.05,0.28520
45,350,0.09,0.75,11,0.1,0.28517
45,350,0.09,0.75,11,0.21436,0.28516
# ---
# Sample Blitz rating of thibault: 1393.0 (rd: 45.000, vola: 0.08395)
# Sample Blitz rating of german11: 1176.9 (rd: 45.000, vola: 0.08606)
# Sample Bullet rating of revoof: 1385.7 (rd: 45.000, vola: 0.08776)
# Sample Bullet rating of drnykterstein: 2686.5 (rd: 45.566, vola: 0.08249)
# Sample Bullet rating of penguingim1: 2575.4 (rd: 45.000, vola: 0.07959)
# Sample Blitz rating of lance5500: 1999.5 (rd: 45.330, vola: 0.07738)
# Sample Blitz rating of somethingpretentious: 1659.1 (rd: 45.000, vola: 0.07559)
# Sample Classical rating of igormezentsev: 1663.4 (rd: 205.781, vola: 0.09000)
# ---
# Estimated UltraBullet distribution: p1=812.7 p10=1044.0 p50=1334.1 p90=1616.9 p99=1989.0, avg=1338.6
# Estimated Bullet distribution: p1=548.0 p10=803.2 p50=1141.5 p90=1607.1 p99=1980.3, avg=1173.7
# Estimated Blitz distribution: p1=501.6 p10=759.9 p50=1179.4 p90=1630.3 p99=1974.8, avg=1187.8
# Estimated Bullet distribution: p1=548.0 p10=803.2 p50=1141.5 p90=1607.1 p99=1980.3, avg=1173.7
# Estimated Classical distribution: p1=779.0 p10=1059.3 p50=1347.8 p90=1714.3 p99=2001.4, avg=1377.6
# Estimated Correspondence distribution: p1=1100.9 p10=1261.6 p50=1440.1 p90=1754.5 p99=2050.9, avg=1484.0
# ---
# Distinct players: 5381208
# Processed encounters: 1409000000 (last at: 2020-07-31 17:42:58)
# Total errors: 0
# ---
```

The most important part is the `avg_deviance` column, which is indicates
the predictive power of the rating system with the given parameters
(lower is better).
