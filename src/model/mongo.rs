use log::info;
use mongodb::{bson::doc, options::ClientOptions, Client};
use std::{
    error::Error,
    fmt::{Display, Formatter},
};

#[derive(Debug)]
pub enum MongoError {
    NoDefaultDatabase,
    FailedToParseObjectId,
}

impl Error for MongoError {}

impl Display for MongoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MongoError::NoDefaultDatabase => write!(f, "No default database"),
            MongoError::FailedToParseObjectId => write!(f, "Failed to parse object id"),
        }
    }
}

pub async fn get_mongo_client(url: &str) -> Result<Client, Box<dyn Error>> {
    let opts = ClientOptions::parse(url).await?;
    let client = Client::with_options(opts)?;

    get_default_db(&client)?
        .run_command(doc! {"ping": 1}, None)
        .await?;
    info!("Connected to MongoDB!: {}", url);

    Ok(client)
}

pub fn get_default_db(client: &Client) -> Result<mongodb::Database, MongoError> {
    match client.default_database() {
        Some(db) => Ok(db),
        None => Err(MongoError::NoDefaultDatabase),
    }
}
