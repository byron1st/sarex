use std::error::Error;

use bson::{doc, oid::ObjectId};
use futures::TryStreamExt;
use mongodb::Collection;
use serde::{Deserialize, Serialize};

use super::mongo::{get_default_db, get_mongo_client};

const MAPPINGRULES_COL: &str = "mappingrules";

#[derive(Debug, Serialize, Deserialize)]
pub struct MappingRule {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    #[serde(rename = "projectId")]
    pub project_id: String,

    pub procedure: String,
    pub relation: Relation,

    #[serde(rename = "connectorType")]
    pub connector_type: String,

    #[serde(rename = "sourceComponentIdentifierSchema")]
    pub source_component_identifier_schema: Vec<String>,

    #[serde(rename = "targetComponentIdentifierSchema")]
    pub target_component_identifier_schema: Vec<String>,
}

impl Clone for MappingRule {
    fn clone(&self) -> Self {
        MappingRule {
            id: self.id,
            project_id: self.project_id.clone(),
            procedure: self.procedure.clone(),
            relation: self.relation.clone(),
            connector_type: self.connector_type.clone(),
            source_component_identifier_schema: self.source_component_identifier_schema.clone(),
            target_component_identifier_schema: self.target_component_identifier_schema.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Relation {
    #[serde(rename = "_id")]
    pub id: String,

    pub source: String,
    pub target: String,

    #[serde(rename = "projectId")]
    pub project_id: String,
}

impl Clone for Relation {
    fn clone(&self) -> Self {
        Relation {
            id: self.id.clone(),
            source: self.source.clone(),
            target: self.target.clone(),
            project_id: self.project_id.clone(),
        }
    }
}

pub async fn read_many(url: &str, project_id: &str) -> Result<Vec<MappingRule>, Box<dyn Error>> {
    let collection = get_col(url).await?;

    let filter = doc! {"projectId": project_id};
    let mut cursor = collection.find(filter, None).await?;

    let mut mapping_rules: Vec<MappingRule> = Vec::new();
    while let Some(mapping_rule) = cursor.try_next().await? {
        mapping_rules.push(mapping_rule);
    }

    Ok(mapping_rules)
}

async fn get_col(url: &str) -> Result<Collection<MappingRule>, Box<dyn Error>> {
    let client = get_mongo_client(url).await?;
    let db = get_default_db(&client)?;

    Ok(db.collection(MAPPINGRULES_COL))
}
