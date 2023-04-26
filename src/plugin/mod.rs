use crate::model::drs::Dr;
use std::error::Error;

mod dir;
mod java;

pub enum PluginKind {
    Java,
    Go,
    JavaScript,
}

pub fn read_drs(
    project_id: String,
    kind: PluginKind,
    params: Vec<String>,
) -> Result<Vec<Dr>, Box<dyn Error>> {
    match kind {
        PluginKind::Java => java::read_drs(&project_id, params),
        PluginKind::Go => Ok(Vec::new()),
        PluginKind::JavaScript => Ok(Vec::new()),
    }
}
