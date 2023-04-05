use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use log::info;
use mongodb::{
    bson::{doc, oid::ObjectId, to_document, DateTime},
    options::ClientOptions,
    Client,
};
use serde::{Deserialize, Serialize};

pub async fn get_mongo_client(url: &str) -> Result<Client, Box<dyn Error>> {
    let opts = ClientOptions::parse(url).await?;
    let client = Client::with_options(opts)?;

    get_default_db(&client)?
        .run_command(doc! {"ping": 1}, None)
        .await?;
    info!("Connected to MongoDB!");

    Ok(client)
}

#[derive(Debug)]
enum MongoError {
    NoDefaultDatabase,
}

impl Error for MongoError {}

impl Display for MongoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MongoError::NoDefaultDatabase => write!(f, "No default database"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Project {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name: String,
    created_at: DateTime,
}

pub async fn create_new_project(client: &Client, name: String) -> Result<Project, Box<dyn Error>> {
    let collection = get_default_db(client)?.collection("projects");

    let mut new_project = Project {
        id: None,
        name,
        created_at: DateTime::now(),
    };
    let doc = to_document(&new_project)?;

    let result = collection.insert_one(doc, None).await?;

    if let Some(id) = result.inserted_id.as_object_id() {
        new_project.id = Some(id.clone());
    }

    Ok(new_project)
}

pub async fn read_projects(client: &Client) -> Result<(), Box<dyn Error>> {
    let collection = get_default_db(client)?.collection::<Project>("projects");

    let mut cursor = collection.find(None, None).await?;

    Ok(())
}

fn get_default_db(client: &Client) -> Result<mongodb::Database, MongoError> {
    match client.default_database() {
        Some(db) => Ok(db),
        None => Err(MongoError::NoDefaultDatabase),
    }
}
