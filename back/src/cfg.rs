#[derive(Deserialize)]
pub struct Config {
    pub db: Db,
}

#[derive(Deserialize)]
pub struct Db {
    pub host: String,
}
