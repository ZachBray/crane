#[derive(Deserialize, Debug)]
pub struct Installation {
    pub id: u32
}

#[derive(Deserialize, Debug)]
pub struct Repository {
    pub url: String
}
