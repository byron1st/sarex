use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Display, Formatter},
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::model::mapping_rules::MappingRule;

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionTrace {
    pub id: String, // <MappingRuleId,Procedure,Index>

    #[serde(rename = "sourceValues")]
    pub source_values: HashMap<String, String>,

    #[serde(rename = "targetValues")]
    pub target_values: HashMap<String, String>,
}

pub fn read_execution_traces(file_path_str: String) -> Result<Vec<ExecutionTrace>, Box<dyn Error>> {
    let file_path = Path::new(&file_path_str);
    let execution_traces_file = OpenOptions::new().read(true).open(file_path)?;
    let reader = BufReader::new(execution_traces_file);
    let mut execution_traces: Vec<ExecutionTrace> = Vec::new();
    for line in reader.lines().flatten() {
        let execution_trace: ExecutionTrace = match serde_json::from_str(&line) {
            Ok(execution_trace) => execution_trace,
            Err(_) => {
                continue;
            }
        };
        execution_traces.push(execution_trace);
    }

    Ok(execution_traces)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ci {
    pub id: String,
    pub connector_type: String,
    pub source_component_values: HashMap<String, String>,
    // pub additional_source_component_values: HashMap<String, String>,
    pub target_component_values: HashMap<String, String>,
}

pub fn create_cis(
    execution_traces: Vec<ExecutionTrace>,
    mapping_rules: Vec<MappingRule>,
) -> Result<Vec<Ci>, Box<dyn Error>> {
    let mut cis: Vec<Ci> = Vec::new();

    for execution_trace in execution_traces {
        let mapping_rule = find_corresponding_mapping_rule(&mapping_rules, &execution_trace.id)?;

        // 모든 Source Value 들을 기록해두어야, 나중에 찾을 수 있음 (execution context)
        let mut source_component_values: HashMap<String, String> = HashMap::new();
        for (key, value) in execution_trace.source_values {
            if !value.is_empty() {
                source_component_values.insert(key, value);
            }
        }

        let mut target_component_values: HashMap<String, String> = HashMap::new();
        for identifier in mapping_rule.target_component_identifier_schema {
            let value = execution_trace.target_values.get(&identifier);
            if let Some(value) = value {
                if !value.is_empty() {
                    target_component_values.insert(identifier, value.clone());
                }
            }
        }

        let ci = Ci {
            id: execution_trace.id.clone(),
            connector_type: mapping_rule.connector_type.clone(),
            source_component_values,
            // additional_source_component_values,
            target_component_values,
        };

        cis.push(ci);
    }

    Ok(cis)
}

fn find_corresponding_mapping_rule(
    mapping_rules: &[MappingRule],
    execution_trace_id: &str,
) -> Result<MappingRule, CIError> {
    let mapping_rule_id = get_mapping_rule_id(execution_trace_id)?;
    let mapping_rule = mapping_rules
        .iter()
        .find(|mapping_rule| {
            if let Some(id) = mapping_rule.id {
                id.to_hex() == mapping_rule_id
            } else {
                false
            }
        })
        .ok_or(CIError::NoCorrespondingMappingRule)?;

    Ok(mapping_rule.clone())
}

fn get_mapping_rule_id(execution_trace_id: &str) -> Result<String, CIError> {
    let split: Vec<&str> = execution_trace_id.split('_').collect();
    if split.is_empty() {
        return Err(CIError::MalformedExecutionTraceId);
    }

    Ok(split[0].to_string())
}

pub fn write_cis(cis: Vec<Ci>, output_file_path_str: &str) -> Result<(), Box<dyn Error>> {
    let p = Path::new(output_file_path_str);
    let mut file = OpenOptions::new().write(true).create(true).open(p)?;
    file.write_all(serde_json::to_string_pretty(&cis)?.as_bytes())?;
    Ok(())
}

#[derive(Debug)]
enum CIError {
    MalformedExecutionTraceId,
    NoCorrespondingMappingRule,
}

impl Error for CIError {}

impl Display for CIError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            CIError::MalformedExecutionTraceId => write!(f, "Malformed execution trace id"),
            CIError::NoCorrespondingMappingRule => write!(f, "No corresponding mapping rule"),
        }
    }
}
