use std::error::Error as StdError;

use std::io;
use liglicko2_research::encounter::RawEncounter;
use liglicko2_research::encounter::PgnResult;
use liglicko2_research::player::PlayerIds;
use liglicko2_research::player::ByPlayerId;
use liglicko2_research::encounter::BySpeed;
use liglicko2_research::encounter::UtcDateTime;
use glicko2::Glicko2Rating;
use glicko2::GameResult;

#[derive(Debug, Default)]
struct PlayerState {
    rating: Glicko2Rating,
    pending: Vec<GameResult>,
}

impl PlayerState {
    fn live_rating(&self) -> Glicko2Rating {
        glicko2::new_rating(self.rating, &self.pending, 1.2)
    }

    fn commit(&mut self) {
        self.rating = self.live_rating();
        self.pending.clear();
    }
}

fn main() -> Result<(), Box<dyn StdError>> {
    let mut reader = csv::Reader::from_reader(io::stdin().lock());

    let mut players = PlayerIds::default();
    let mut states: BySpeed<ByPlayerId<PlayerState>> = BySpeed::default();
    let mut last_rating_period = UtcDateTime::default();

    for encounter in reader.deserialize() {
        let encounter: RawEncounter = encounter?;

        if encounter.utc_date_time.as_seconds() > last_rating_period.as_seconds() + 7 * 24 * 60 * 60 {
            for states in states.values_mut() {
                for state in states.values_mut() {
                    if let Some(state) = state {
                        state.commit();
                    }
                }
            }
            last_rating_period = encounter.utc_date_time; // Close enough, because encounters are dense
        }

        let white = players.get_or_insert(encounter.white);
        let black = players.get_or_insert(encounter.black);
        let states = states.get_mut(encounter.time_control.speed());

        let white_rating = states.get(white).map_or_else(Glicko2Rating::unrated, |state| state.rating);
        let black_rating = states.get(black).map_or_else(Glicko2Rating::unrated, |state| state.rating);

        states.get_mut_or_insert_with(white, PlayerState::default).pending.push(match encounter.result {
            PgnResult::WhiteWins => GameResult::win(black_rating),
            PgnResult::BlackWins => GameResult::loss(black_rating),
            PgnResult::Draw => GameResult::draw(black_rating),
            PgnResult::Unknown => continue,
        });

        states.get_mut_or_insert_with(black, PlayerState::default).pending.push(match encounter.result {
            PgnResult::WhiteWins => GameResult::loss(white_rating),
            PgnResult::BlackWins => GameResult::win(white_rating),
            PgnResult::Draw => GameResult::draw(white_rating),
            PgnResult::Unknown => continue,
        });
    }

    Ok(())
}
