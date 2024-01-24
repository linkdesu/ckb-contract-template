use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::{env, str};

use ckb_types::bytes;
use ckb_types::packed::{Byte, Byte32, Bytes, Script};
use ckb_types::prelude::{Builder, Entity};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::constants::*;
use super::since_util::SinceFlag;
use super::{since_util, util};

pub enum ContractType {
    DeployedContract,
    Contract,
    DeployedSharedLib,
    SharedLib,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum SubAccountActionType {
    Create,
    Edit,
    Renew,
    Recycle,
}

impl SubAccountActionType {
    fn to_string(self) -> String {
        self.into()
    }
}

impl Into<String> for SubAccountActionType {
    fn into(self) -> String {
        match self {
            SubAccountActionType::Create => "create".to_string(),
            SubAccountActionType::Edit => "edit".to_string(),
            SubAccountActionType::Renew => "renew".to_string(),
            SubAccountActionType::Recycle => "recycle".to_string(),
        }
    }
}

pub fn gen_fake_signhash_all_lock(lock_args: &str) -> Script {
    Script::new_builder()
        .code_hash(util::slice_to_byte32(&util::get_type_id_bytes(
            "fake-secp256k1-blake160-signhash-all",
        )))
        .hash_type(Byte::new(1))
        .args(util::slice_to_bytes(&util::hex_to_bytes(lock_args)))
        .build()
}

pub fn gen_since(relative_flag: SinceFlag, metric_flag: SinceFlag, value: u64) -> Option<u64> {
    let mut since = 0u64;
    since = since_util::set_relative_flag(since, relative_flag);
    since = since_util::set_metric_flag(since, metric_flag);
    since = since_util::set_value(since, value);

    // println!("0b{:064b}", since);
    Some(since)
}

/// Parse string in JSON
///
/// All string will be treated as utf8 encoding.
fn parse_json_str<'a>(field_name: &str, field: &'a Value) -> &'a str {
    field.as_str().expect(&format!("{} is missing", field_name))
}

fn parse_json_str_with_default<'a>(field_name: &str, field: &'a Value, default: &'a str) -> &'a str {
    if field.is_null() {
        default
    } else {
        parse_json_str(field_name, field)
    }
}

/// Parse string in JSON and return &[u8]
///
/// All string will be treated as utf8 encoding.
fn parse_json_str_to_bytes<'a>(field_name: &str, field: &'a Value) -> &'a [u8] {
    field.as_str().expect(&format!("{} is missing", field_name)).as_bytes()
}

/// Parse array in JSON
fn parse_json_array<'a>(field_name: &str, field: &'a Value) -> &'a [Value] {
    field
        .as_array()
        .map(|v| v.as_slice())
        .expect(&format!("{} is missing", field_name))
}

/// Parse struct Script and fill optional fields
///
/// Example:
/// ```json
/// // input
/// {
///     code_hash: "{{xxx-cell-type}}"
///     hash_type: "type", // could be omit if it is "type"
///     args: "" // could be omit if it it empty
/// }
/// // output
/// {
///     code_hash: "{{xxx-cell-type}}",
///     hash_type: "type",
///     args: ""
/// }
/// ```
fn parse_json_script(field_name: &str, field: &Value) -> Value {
    let code_hash = field["code_hash"]
        .as_str()
        .expect(&format!("{} is missing", field_name));
    let hash_type = match field["hash_type"].as_str() {
        Some("data") => "data",
        _ => "type",
    };
    let args = match field["args"].as_str() {
        Some(val) => val,
        _ => "",
    };

    json!({
        "code_hash": code_hash,
        "hash_type": hash_type,
        "args": args
    })
}

/// Parse struct Script to hex of molecule encoding, if field is null will return Script::default()
///
/// Example:
/// ```json
/// {
///     code_hash: "{{xxx-cell-type}}"
///     hash_type: "type", // could be omit if it is "type"
///     args: "" // could be omit if it it empty
/// }
/// ```
fn parse_json_script_to_mol(field_name: &str, field: &Value) -> Script {
    if field.is_null() {
        return Script::default();
    }

    let code_hash = field["code_hash"]
        .as_str()
        .expect(&format!("{} is missing", field_name));
    let code_hash_bytes = if let Some(caps) = RE_VARIABLE.captures(code_hash) {
        let cap = caps.get(1).expect("The captures[1] should always exist.");
        util::get_type_id_bytes(cap.as_str())
    } else {
        util::hex_to_bytes(code_hash)
    };

    let hash_type = match field["hash_type"].as_str() {
        Some("data") => ScriptHashType::Data,
        _ => ScriptHashType::Type,
    };
    let args = match field["args"].as_str() {
        Some(val) => util::hex_to_bytes(val),
        _ => Vec::new(),
    };

    Script::new_builder()
        .code_hash(util::slice_to_byte32(&code_hash_bytes))
        .hash_type(Byte::new(hash_type as u8))
        .args(util::slice_to_bytes(&args))
        .build()
}

fn length_of(data: &[u8]) -> Vec<u8> {
    (data.len() as u32).to_le_bytes().to_vec()
}

pub struct TemplateGenerator {
    loaded_contracts: Vec<String>,
    // Transaction fields
    pub header_deps: Vec<Value>,
    pub cell_deps: Vec<Value>,
    pub inputs: Vec<Value>,
    pub outputs: Vec<Value>,
    pub inner_witnesses: Vec<String>,
    pub outer_witnesses: Vec<String>,
}

impl TemplateGenerator {
    pub fn new(action: &str, params_opt: Option<Bytes>) -> TemplateGenerator {
        TemplateGenerator {
            loaded_contracts: vec![],
            header_deps: Vec::new(),
            cell_deps: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            inner_witnesses: Vec::new(),
            outer_witnesses: Vec::new(),
        }
    }

    pub fn push_empty_witness(&mut self) {
        self.inner_witnesses.push(String::from("0x"));
    }

    /// The header_deps should be an array of objects like below:
    ///
    /// ```json
    /// {
    ///     "version": u32,
    ///     "number": u64,
    ///     "timestamp": u64 | "YYYY-MM-DD HH:MM:SS",
    ///     "epoch": u64,
    ///     "transactions_root": "0x...",
    /// }
    /// ```
    pub fn push_header_deps(&mut self, header: Value) {
        let version = util::parse_json_u32("header.version", &header["version"], Some(0));
        let number = if header["number"].is_null() {
            util::parse_json_u64("header.height", &header["height"], Some(0))
        } else {
            util::parse_json_u64("header.number", &header["number"], Some(0))
        };
        let timestamp = util::parse_json_u64("header.timestamp", &header["timestamp"], Some(0));
        let epoch = util::parse_json_u64("header.epoch", &header["epoch"], Some(0));
        let transactions_root = header["transactions_root"].clone();

        let value = json!({
            "version": version,
            "number": number,
            "timestamp": timestamp,
            "epoch": epoch,
            "transactions_root": transactions_root
        });

        self.header_deps.push(value);
    }

    pub fn push_contract_cell(&mut self, contract_filename: &str, type_: ContractType) {
        let value = match type_ {
            ContractType::Contract => {
                json!({
                    "tmp_type": "contract",
                    "tmp_file_name": contract_filename
                })
            }
            ContractType::DeployedContract => {
                json!({
                    "tmp_type": "deployed_contract",
                    "tmp_file_name": contract_filename
                })
            }
            ContractType::SharedLib => {
                json!({
                    "tmp_type": "shared_lib",
                    "tmp_file_name": contract_filename
                })
            }
            ContractType::DeployedSharedLib => {
                json!({
                    "tmp_type": "deployed_shared_lib",
                    "tmp_file_name": contract_filename
                })
            }
        };

        self.loaded_contracts.push(contract_filename.to_string());
        self.cell_deps.push(value)
    }

    pub fn push_dep(&mut self, cell: Value, version_opt: Option<u32>) -> usize {
        self.push_cell(cell, Source::CellDep, version_opt, None)
    }

    pub fn push_input(&mut self, cell: Value, since_opt: Option<u64>, version_opt: Option<u32>) -> usize {
        self.push_cell(cell, Source::Input, version_opt, since_opt)
    }

    pub fn push_output(&mut self, cell: Value, version_opt: Option<u32>) -> usize {
        self.push_cell(cell, Source::Output, version_opt, None)
    }

    pub fn push_cell(
        &mut self,
        cell: Value,
        source: Source,
        version_opt: Option<u32>,
        since_opt: Option<u64>,
    ) -> usize {
        macro_rules! push_cell {
            ($gen_fn:ident, $cell:expr) => {{
                let cell = self.$gen_fn($cell);
                self.push_cell_json(cell, source, since_opt)
            }};
            ($data_type:expr, $gen_fn:ident, $version_opt:expr, $cell:expr) => {{
                let version = if let Some(version) = $version_opt {
                    version
                } else {
                    1
                };

                let (cell, entity_opt) = self.$gen_fn(version, $cell);
                self.push_cell_json_with_entity(cell, source, $data_type, version, entity_opt, since_opt)
            }};
        }

        if let Some(type_script) = cell.get("type") {
            let code_hash = type_script
                .get("code_hash")
                .expect("cell.type.code_hash is missing")
                .as_str()
                .expect("cell.type.code_hash should be a string");

            if let Some(caps) = RE_VARIABLE.captures(code_hash) {
                let type_id = caps
                    .get(1)
                    .map(|m| m.as_str())
                    .expect("type.code_hash is something like '{{...}}'");

                if source != Source::CellDep && !self.loaded_contracts.contains(&type_id.to_string()) {
                    panic!("The contract {} has no cell_deps, please use TemplateGenerater::push_contract_cell to push the related cell_deps.", type_id);
                }

                let index = match type_id {
                    "playground" => push_cell!(gen_custom_cell, cell),
                    _ => panic!("Unknown type ID {}", type_id),
                };

                index
            } else {
                panic!("{}", "type.code_hash is something like '{{...}}'")
            }
        } else {
            push_cell!(gen_custom_cell, cell)
        }
    }

    pub fn push_cell_json(&mut self, mut cell: Value, source: Source, since_opt: Option<u64>) -> usize {
        if !cell["tmp_header"].is_null() {
            let mut timestamp = util::parse_json_u64("header.timestamp", &cell["tmp_header"]["timestamp"], Some(0));
            timestamp = timestamp * 1000; // The timestamp in real block header contains milliseconds.

            let field = &mut cell["tmp_header"]["timestamp"];
            *field = json!(timestamp);
        }

        if source == Source::Input {
            let since = if let Some(since) = since_opt { since } else { 0 };

            cell = json!({
                "previous_output": cell,
                "since": since
            });
        }

        match source {
            Source::CellDep => {
                self.cell_deps.push(cell);
                self.cell_deps.len() - 1
            }
            Source::Input => {
                self.inputs.push(cell);
                self.inputs.len() - 1
            }
            Source::Output => {
                self.outputs.push(cell);
                self.outputs.len() - 1
            }
        }
    }

    /// Cell structure:
    ///
    /// ```json
    /// json!({
    ///     "capacity": u64,
    ///     "lock": Script,
    ///     "type": null | Script,
    ///     "data": null | "0x..."
    /// })
    /// ```
    fn gen_custom_cell(&mut self, cell: Value) -> Value {
        let capacity: u64 = util::parse_json_u64("cell.capacity", &cell["capacity"], Some(0));

        let lock_script = parse_json_script("cell.lock", &cell["lock"]);
        let type_script = cell["type"].clone();
        let outputs_data = if !cell["data"].is_null() {
            util::bytes_to_hex(&util::parse_json_hex("cell.data", &cell["data"]))
        } else {
            String::from("0x")
        };

        json!({
            "tmp_header": cell["header"],
            "tmp_type": "full",
            "capacity": capacity,
            "lock": lock_script,
            "type": type_script,
            "tmp_data": outputs_data
        })
    }

    // ======

    pub fn as_json(&self) -> Value {
        let mut witnesses = [self.inner_witnesses.clone(), self.outer_witnesses.clone()].concat();

        json!({
            "header_deps": self.header_deps,
            "cell_deps": self.cell_deps,
            "inputs": self.inputs,
            "outputs": self.outputs,
            "witnesses": witnesses,
        })
    }

    pub fn write_template(&self, filename: &str) {
        let mut filepath = env::current_dir().unwrap();
        filepath.push("templates");
        filepath.push(filename);

        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(filepath.clone())
            .expect(format!("Expect file path {:?} to be writable.", filepath).as_str());

        let data = serde_json::to_string_pretty(&self.as_json()).unwrap();
        file.write(data.as_bytes()).expect("Write file failed.");
    }
}
