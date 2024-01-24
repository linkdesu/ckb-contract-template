use serde_json::json;

use crate::util;
use crate::util::constants::*;
use crate::util::template_generator::*;
use crate::util::template_parser::*;

fn init(action: &str) -> TemplateGenerator {
    let mut template = TemplateGenerator::new(action, None);

    template.push_contract_cell("always-success", ContractType::Contract);
    template.push_contract_cell("playground", ContractType::Contract);
    // template.push_shared_lib_cell("ckb_smt.so", false);
    template.push_contract_cell("secp256k1_data", ContractType::DeployedSharedLib);

    template.push_header_deps(json!({
        "height": HEIGHT,
        "timestamp": TIMESTAMP,
    }));

    template
}

#[test]
fn test_playground() {
    let mut template = init("playground");

    template.push_input(
        json!({
            "capacity": 0,
            "lock": {
                "code_hash": "{{always-success}}"
            },
            "type": {
                "code_hash": "{{playground}}"
            }
        }),
        None,
        None,
    );
    template.push_empty_witness();

    test_tx(template.as_json());
}
