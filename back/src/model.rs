use std::i8;
use std::fmt::{self, Formatter};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self, Unexpected, Visitor};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gender {
    Male,
    Female,
    Other,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    Realistic,
    Pleasing,
}

#[derive(Deserialize)]
pub struct Weighting {
    pub token: String,
    pub metric: Metric,
    pub a: String,
    pub b: String,
    pub weight: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Sample {
    pub task: String,
    pub name: String,
    pub fitness: f32,
}

pub struct Likert5(i8);

impl Likert5 {
    pub fn new(selected: i8) -> Result<Likert5, ()> {
        if selected > 2 || selected < -2 {
            Err(())
        } else {
            Ok(Likert5(selected))
        }
    }
}

impl Serialize for Likert5 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(i32::from(self.0))
    }
}

struct Likert5Visitor;

impl<'de> Visitor<'de> for Likert5Visitor {
    type Value = Likert5;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("an integer between -2 and 2")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Likert5, E>
    where
        E: de::Error,
    {
        match Likert5::new(value as i8) {
            Ok(likert) => Ok(likert),
            Err(()) => Err(de::Error::invalid_value(Unexpected::Signed(value), &self)),
        }
    }

    fn visit_u64<E>(self, value: u64) -> Result<Likert5, E>
    where
        E: de::Error,
    {
        if value > i8::MAX as u64 {
            return Err(de::Error::invalid_value(Unexpected::Unsigned(value), &self));
        }

        match Likert5::new(value as i8) {
            Ok(likert) => Ok(likert),
            Err(()) => Err(de::Error::invalid_value(Unexpected::Unsigned(value), &self)),
        }
    }
}

impl<'de> Deserialize<'de> for Likert5 {
    fn deserialize<D>(deserializer: D) -> Result<Likert5, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_i8(Likert5Visitor)
    }
}

#[derive(Serialize, Deserialize)]
pub struct PostQuestionnaire {
    ranking_agree: Likert5,
    differentiates: String,
    comments: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PreQuestionnaire {
    plant_work: Likert5,
    plant_like: Likert5,
    video_game: Likert5,
}
