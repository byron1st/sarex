use crate::model::drs::Dr;
use std::error::Error;

mod dir;
mod go;
mod java;
mod js;

pub enum PluginKind {
    Java,
    Go,
    JavaScript,
}

pub fn read_drs(
    project_id: &str,
    kind: PluginKind,
    params: Vec<&str>,
) -> Result<Vec<Dr>, Box<dyn Error>> {
    match kind {
        PluginKind::Java => java::read_drs(project_id, params),
        PluginKind::Go => go::read_drs(project_id, params),
        PluginKind::JavaScript => js::read_drs(project_id, params),
    }
}
