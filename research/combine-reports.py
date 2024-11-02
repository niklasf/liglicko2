#!/usr/bin/env python3

import sys
import dataclasses

@dataclasses.dataclass
class Experiment:
    min_deviation: float
    max_deviation: float
    default_volatility: float
    tau: float
    first_advantage: float
    rating_periods_per_day: float
    avg_deviance: float

experiments = []

for line in sys.stdin:
    line = line.strip()
    if not line or line.startswith("#") or line.endswith("avg_deviance"):
        continue

    min_deviation, max_deviation, default_volatility, tau, first_advantage, rating_periods_per_day, avg_deviance = line.split(",")
    experiments.append(
        Experiment(
            min_deviation=float(min_deviation),
            max_deviation=float(max_deviation),
            default_volatility=float(default_volatility),
            tau=float(tau),
            first_advantage=float(first_advantage),
            rating_periods_per_day=float(rating_periods_per_day),
            avg_deviance=float(avg_deviance)))

experiments.sort(key=lambda e: e.avg_deviance, reverse=True)

print("min_deviation,max_deviation,default_volatility,tau,first_advantage,rating_periods_per_day,avg_deviance")
for ex in experiments:
    print(f"{ex.min_deviation},{ex.max_deviation},{ex.default_volatility},{ex.tau},{ex.first_advantage},{ex.rating_periods_per_day},{ex.avg_deviance}")
