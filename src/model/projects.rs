use super::mongo::{get_default_db, get_mongo_client, MongoError};
use futures::stream::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime},
    Collection,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, str::FromStr};

const PROJECTS_COL: &str = "projects";

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub created_at: DateTime,
}

pub async fn create(url: &str, name: String) -> Result<String, Box<dyn Error>> {
    let collection = get_col(url).await?;

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

pub async fn read_many(url: &str) -> Result<Vec<Project>, Box<dyn Error>> {
    let collection = get_col(url).await?;

    let mut cursor = collection.find(None, None).await?;

    let mut projects: Vec<Project> = Vec::new();
    while let Some(project) = cursor.try_next().await? {
        projects.push(project);
    }

    Ok(projects)
}

pub async fn read_one(url: &str, id: &str) -> Result<Option<Project>, Box<dyn Error>> {
    let collection = get_col(url).await?;

    let oid = ObjectId::from_str(id)?;
    let filter = doc! {"_id": oid};

    match collection.find_one(filter, None).await {
        Ok(result) => Ok(result),
        Err(e) => Err(Box::new(e)),
    }
}

pub async fn update(url: &str, id: &str, name: String) -> Result<String, Box<dyn Error>> {
    let collection = get_col(url).await?;

    let oid = ObjectId::from_str(id)?;
    let filter = doc! {"_id": oid};
    let update = doc! {"$set": {"name": name}};

    collection.update_one(filter, update, None).await?;

    Ok(String::from(id))
}

async fn get_col(url: &str) -> Result<Collection<Project>, Box<dyn Error>> {
    let client = get_mongo_client(url).await?;
    let db = get_default_db(&client)?;

    Ok(db.collection(PROJECTS_COL))
}
