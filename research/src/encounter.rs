use std::{fmt, str::FromStr};

use chrono::{DateTime, NaiveDateTime};
use liglicko2::Score;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use thiserror::Error;

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct RawEncounter {
    pub white: String,
    pub black: String,
    #[serde_as(as = "DisplayFromStr")]
    pub result: PgnResult,
    #[serde_as(as = "DisplayFromStr")]
    pub utc_date_time: UtcDateTime,
    #[serde_as(as = "DisplayFromStr")]
    pub time_control: TimeControl,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PgnResult {
    Unknown,
    WhiteWins,
    BlackWins,
    Draw,
}

#[derive(Debug, Error)]
#[error("invalid game result")]
pub struct InvalidPgnResult;

impl FromStr for PgnResult {
    type Err = InvalidPgnResult;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "1-0" => PgnResult::WhiteWins,
            "0-1" => PgnResult::BlackWins,
            "1/2-1/2" => PgnResult::Draw,
            "*" => PgnResult::Unknown,
            _ => return Err(InvalidPgnResult),
        })
    }
}

impl PgnResult {
    pub fn white_score(self) -> Option<Score> {
        Some(match self {
            PgnResult::WhiteWins => Score::WIN,
            PgnResult::BlackWins => Score::LOSS,
            PgnResult::Draw => Score::DRAW,
            PgnResult::Unknown => return None,
        })
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct UtcDateTime(i64);

impl FromStr for UtcDateTime {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UtcDateTime(
            NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")?
                .and_utc()
                .timestamp(),
        ))
    }
}

impl fmt::Display for UtcDateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            DateTime::from_timestamp(self.0, 0)
                .unwrap_or_default()
                .naive_utc()
        )
    }
}

impl UtcDateTime {
    pub fn as_seconds(self) -> i64 {
        self.0
    }
}

#[derive(Debug)]
pub enum TimeControl {
    Clock { limit: u32, increment: u32 },
    Correspondence,
}

#[derive(Debug, Error)]
#[error("invalid time control")]
pub struct InvalidTimeControl;

impl FromStr for TimeControl {
    type Err = InvalidTimeControl;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s == "-" {
            TimeControl::Correspondence
        } else {
            let mut parts = s.splitn(2, '+');
            let limit = parts
                .next()
                .ok_or(InvalidTimeControl)?
                .parse()
                .map_err(|_| InvalidTimeControl)?;
            let increment = parts
                .next()
                .ok_or(InvalidTimeControl)?
                .parse()
                .map_err(|_| InvalidTimeControl)?;
            TimeControl::Clock { limit, increment }
        })
    }
}

impl TimeControl {
    fn estimate_total_seconds(&self) -> Option<u32> {
        match *self {
            TimeControl::Clock { limit, increment } => Some(limit + 40 * increment),
            TimeControl::Correspondence => None,
        }
    }

    pub fn speed(&self) -> Speed {
        match self.estimate_total_seconds() {
            Some(0..30) => Speed::UltraBullet,
            Some(30..180) => Speed::Bullet,
            Some(180..480) => Speed::Blitz,
            Some(480..1500) => Speed::Rapid,
            Some(1500..) => Speed::Classical,
            None => Speed::Correspondence,
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum Speed {
    UltraBullet,
    Bullet,
    Blitz,
    Rapid,
    Classical,
    Correspondence,
}

#[derive(Debug, Clone, Default)]
pub struct BySpeed<T> {
    pub ultra_bullet: T,
    pub bullet: T,
    pub blitz: T,
    pub rapid: T,
    pub classical: T,
    pub correspondence: T,
}

impl<T> BySpeed<T> {
    pub fn get(&self, speed: Speed) -> &T {
        match speed {
            Speed::UltraBullet => &self.ultra_bullet,
            Speed::Bullet => &self.bullet,
            Speed::Blitz => &self.blitz,
            Speed::Rapid => &self.rapid,
            Speed::Classical => &self.classical,
            Speed::Correspondence => &self.correspondence,
        }
    }

    pub fn get_mut(&mut self, speed: Speed) -> &mut T {
        match speed {
            Speed::UltraBullet => &mut self.ultra_bullet,
            Speed::Bullet => &mut self.bullet,
            Speed::Blitz => &mut self.blitz,
            Speed::Rapid => &mut self.rapid,
            Speed::Classical => &mut self.classical,
            Speed::Correspondence => &mut self.correspondence,
        }
    }

    pub fn values_mut(&mut self) -> [&mut T; 6] {
        [
            &mut self.ultra_bullet,
            &mut self.bullet,
            &mut self.blitz,
            &mut self.rapid,
            &mut self.classical,
            &mut self.correspondence,
        ]
    }
}
