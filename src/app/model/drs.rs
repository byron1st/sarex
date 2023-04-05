use super::mongo::{get_default_db, get_mongo_client};
use bson::oid::ObjectId;
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use std::error::Error;

const DRS_COL: &str = "drs";

#[derive(Debug, Serialize, Deserialize)]
pub struct Dr {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub source: String,
    pub target: String,
}

async fn get_drs_col(url: &str) -> Result<Collection<Dr>, Box<dyn Error>> {
    let client = get_mongo_client(url).await?;
    let db = get_default_db(&client)?;

    Ok(db.collection(DRS_COL))
}
