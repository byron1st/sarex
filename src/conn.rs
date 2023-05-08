use std::{
    collections::HashMap,
    error::Error,
    fs::OpenOptions,
    io::{BufReader, Write},
    path::Path,
};

use graphviz_rust::{cmd::CommandArg, dot_structures::*, printer::DotPrinter};
use graphviz_rust::{cmd::Format, dot_generator::*, exec, printer::PrinterContext};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::ci::Ci;

pub fn read_cis(file_path_str: &str) -> Result<Vec<Ci>, Box<dyn Error>> {
    let file_path = Path::new(file_path_str);
    let cis_file = OpenOptions::new().read(true).open(file_path)?;
    let reader = BufReader::new(cis_file);
    let cis: Vec<Ci> = serde_json::from_reader(reader)?;

    Ok(cis)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    pub connectors: Vec<Connector>,
    pub components: Vec<Component>,
}

impl Model {
    fn new() -> Self {
        Self {
            connectors: Vec::new(),
            components: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Connector {
    pub connector_type: String,
    pub source_component_id: String,
    pub target_component_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub component_values: HashMap<String, String>,
}

pub fn build_model(cis: Vec<Ci>) -> Result<Model, Box<dyn Error>> {
    let mut model = Model::new();

    for ci in cis {
        let source_component_id = find_or_create_component_if_not_exist(&mut model, &ci, true);
        let target_component_id = find_or_create_component_if_not_exist(&mut model, &ci, false);
        let connector = Connector {
            connector_type: ci.connector_type,
            source_component_id,
            target_component_id,
        };
        model.connectors.push(connector);
    }

    Ok(model)
}

fn find_or_create_component_if_not_exist(model: &mut Model, ci: &Ci, is_source: bool) -> String {
    let component_values = if is_source {
        &ci.source_component_values
    } else {
        &ci.target_component_values
    };

    let source_component_id = find_component_in_model(&model, component_values);
    if let Some(source_component) = source_component_id {
        source_component
    } else {
        let new_component_id = get_random_id(10);
        let mut new_component_values = component_values.clone();
        if is_source {
            new_component_values.extend(ci.additional_source_component_values.clone());
        }

        model.components.push(Component {
            id: new_component_id.clone(),
            component_values: new_component_values,
        });

        new_component_id
    }
}

fn find_component_in_model(
    model: &Model,
    component_values: &HashMap<String, String>,
) -> Option<String> {
    for component in &model.components {
        let mut is_same = true;
        for (identifier, value) in component_values {
            if value != "" {
                let component_value = component.component_values.get(identifier);
                is_same = is_same && component_value == Some(&value);
            }
        }
        if is_same {
            return Some(component.id.clone());
        }
    }

    None
}

fn get_random_id(len: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub fn write_model(
    model: Model,
    output_file_path_str: &str,
    output_format_str: &str,
) -> Result<(), Box<dyn Error>> {
    match output_format_str {
        "json" => write_model_as_json(model, output_file_path_str)?,
        "png" => write_model_as_png(model, output_file_path_str)?,
        "dot" => write_model_as_dot(model, output_file_path_str)?,
        _ => write_model_as_json(model, output_file_path_str)?,
    }

    Ok(())
}

fn write_model_as_json(model: Model, output_file_path_str: &str) -> Result<(), Box<dyn Error>> {
    let result: String = serde_json::to_string_pretty(&model)?;
    print_result_str(&result, output_file_path_str)
}

fn write_model_as_png(model: Model, output_file_path_str: &str) -> Result<(), Box<dyn Error>> {
    match exec(
        get_dot_graph(model),
        &mut PrinterContext::default(),
        vec![
            Format::Png.into(),
            CommandArg::Output(output_file_path_str.to_string()),
        ],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

fn write_model_as_dot(model: Model, output_file_path_str: &str) -> Result<(), Box<dyn Error>> {
    let result = get_dot_graph(model).print(&mut PrinterContext::default());
    print_result_str(&result, output_file_path_str)
}

fn print_result_str(result: &str, file_path_str: &str) -> Result<(), Box<dyn Error>> {
    let p = Path::new(file_path_str);
    let mut file = OpenOptions::new().write(true).create(true).open(p)?;

    match file.write_all(result.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

fn get_dot_graph(model: Model) -> Graph {
    let mut g = graph!(di id!("model"));

    for component in model.components {
        let label = get_node_label(&component.component_values);
        let node = node!(component.id;attr!("label", &label));
        g.add_stmt(stmt!(node));
    }

    for connector in model.connectors {
        let edge = edge!(node_id!(connector.source_component_id) => node_id!(connector.target_component_id);attr!("label", &connector.connector_type));
        g.add_stmt(stmt!(edge));
    }

    g
}

fn get_node_label(component_values: &HashMap<String, String>) -> String {
    let mut label: String = String::from("\"");
    for (identifier, value) in component_values {
        if value != "" {
            label.push_str(format!("{}:{}\\n", &identifier, &value).as_str());
        }
    }
    label.push_str("\"");

    label
}
