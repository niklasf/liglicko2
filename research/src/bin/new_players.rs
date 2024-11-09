use compensated_summation::KahanBabuskaNeumaier;
use liglicko2_research::encounter::BySpeed;
use liglicko2_research::encounter::{RawEncounter, UtcDateTime};
use rustc_hash::FxHashSet;
use std::error::Error as StdError;
use std::io;

#[derive(Default, Debug)]
struct Stats {
    total_first_score: KahanBabuskaNeumaier<f64>,
    total_first_games: u64,
    players: FxHashSet<String>,
}

impl Stats {
    pub fn csv_header(prefix: &str) -> String {
        format!("{}_avg_first_player_score", prefix)
    }

    pub fn csv(&self) -> String {
        format!(
            "{}",
            self.total_first_score.total() / self.total_first_games as f64
        )
    }
}

fn main() -> Result<(), Box<dyn StdError>> {
    let mut reader = csv::Reader::from_reader(io::stdin().lock());

    let mut last_intermediate_report = UtcDateTime::default();

    let mut by_speed: BySpeed<Stats> = BySpeed::default();

    println!(
        "date,{},{},{},{},{},{}",
        Stats::csv_header("ultra_bullet"),
        Stats::csv_header("bullet"),
        Stats::csv_header("blitz"),
        Stats::csv_header("rapid"),
        Stats::csv_header("classical"),
        Stats::csv_header("correspondence"),
    );

    for encounter in reader.deserialize() {
        let encounter: RawEncounter = encounter?;

        if encounter.utc_date_time.as_seconds()
            > last_intermediate_report.as_seconds() + 7 * 24 * 60 * 60
        {
            last_intermediate_report = encounter.utc_date_time;
            println!(
                "{},{},{},{},{},{},{}",
                last_intermediate_report,
                by_speed.ultra_bullet.csv(),
                by_speed.bullet.csv(),
                by_speed.blitz.csv(),
                by_speed.rapid.csv(),
                by_speed.classical.csv(),
                by_speed.correspondence.csv(),
            );
        }

        let score = match encounter.result.white_score() {
            Some(score) => score,
            None => continue,
        };

        let speed = encounter.time_control.speed();

        let stats = by_speed.get_mut(speed);

        if stats.players.insert(encounter.white) {
            stats.total_first_score += score.value();
            stats.total_first_games += 1;
        }

        if stats.players.insert(encounter.black) {
            stats.total_first_score += score.opposite().value();
            stats.total_first_games += 1;
        }
    }

    Ok(())
}
