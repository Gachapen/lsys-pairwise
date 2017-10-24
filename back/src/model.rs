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

#[derive(Serialize, Deserialize)]
pub struct Weighting {
    pub token: String,
    pub metric: Metric,
    pub a: String,
    pub b: String,
    pub weight: f32,
}
