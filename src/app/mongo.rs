use futures::stream::TryStreamExt;
use log::info;
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime},
    options::ClientOptions,
    Client, Collection,
};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{Display, Formatter},
    str::FromStr,
};

#[derive(Debug)]
enum MongoError {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub created_at: DateTime,
}

pub async fn create_new_project(url: &str, name: String) -> Result<String, Box<dyn Error>> {
    let collection = get_projects_col(url).await?;

    let new_project = Project {
        id: None,
        name,
        created_at: DateTime::now(),
    };

    let result = collection.insert_one(new_project, None).await?;

    match result.inserted_id.as_object_id() {
        Some(id) => Ok(id.to_hex()),
        None => Err(Box::new(MongoError::FailedToParseObjectId)),
    }
}

pub async fn read_projects(url: &str) -> Result<Vec<Project>, Box<dyn Error>> {
    let collection = get_projects_col(url).await?;

    let mut cursor = collection.find(None, None).await?;

    let mut projects: Vec<Project> = Vec::new();
    while let Some(project) = cursor.try_next().await? {
        projects.push(project);
    }

    Ok(projects)
}

pub async fn read_project_by_id(url: &str, id: &str) -> Result<Option<Project>, Box<dyn Error>> {
    let collection = get_projects_col(url).await?;

    let oid = ObjectId::from_str(id)?;
    let filter = doc! {"_id": oid};

    match collection.find_one(filter, None).await {
        Ok(result) => Ok(result),
        Err(e) => return Err(Box::new(e)),
    }
}

pub async fn update_project_name(
    url: &str,
    id: &str,
    name: String,
) -> Result<String, Box<dyn Error>> {
    let collection = get_projects_col(url).await?;

    let oid = ObjectId::from_str(id)?;
    let filter = doc! {"_id": oid};
    let update = doc! {"$set": {"name": name}};

    collection.update_one(filter, update, None).await?;

    Ok(String::from(id))
}

const PROJECTS_COL: &str = "projects";
async fn get_projects_col(url: &str) -> Result<Collection<Project>, Box<dyn Error>> {
    let client = get_mongo_client(url).await?;
    let db = get_default_db(&client)?;

    Ok(db.collection(PROJECTS_COL))
}

// TODO: Other collections will come...

fn get_default_db(client: &Client) -> Result<mongodb::Database, MongoError> {
    match client.default_database() {
        Some(db) => Ok(db),
        None => Err(MongoError::NoDefaultDatabase),
    }
}
