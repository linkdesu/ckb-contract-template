use std::collections::HashMap;

use ckb_types::{h256, H256};
use lazy_static::lazy_static;
use regex::Regex;

// ⚠️ The maximum cycles on-chain is 70_000_000.
pub const MAX_CYCLES: u64 = u64::MAX;

pub const USD_1: u64 = 1_000_000;
pub const USD_5: u64 = 5 * USD_1;
pub const USD_10: u64 = 10 * USD_1;
pub const USD_20: u64 = 20 * USD_1;

pub const ONE_CKB: u64 = 100_000_000;
pub const CKB_QUOTE: u64 = 1000;

pub const TIMESTAMP: u64 = 1611200090u64;
pub const TIMESTAMP_20221018: u64 = 1666094400u64;
pub const HEIGHT: u64 = 1000000u64;

pub const HOUR_SEC: u64 = 3600;
pub const DAY_SEC: u64 = 86400;
pub const MONTH_SEC: u64 = DAY_SEC * 30;
pub const YEAR_SEC: u64 = DAY_SEC * 365;

// error numbers
pub const ERROR_EMPTY_ARGS: i8 = 5;

pub const SECP_SIGNATURE_SIZE: usize = 65;

pub const SIGHASH_TYPE_HASH: H256 = h256!("0x709f3fda12f561cfacf92273c57a98fede188a3f1a59b1f888d113f9cce08649");
pub const MULTISIG_TYPE_HASH: H256 = h256!("0x5c5069eb0857efc65e1bca0c07df34c31663b3622fd3876c876320fc9634e2a8");
pub const DAO_TYPE_HASH: H256 = h256!("0x82d76d1b75fe2fd9a27dfbaa65a039221a380d76c926f378d3f81cf3e7e13f2e");
pub const BLACK_HOLE_HASH: H256 = h256!("0x0000000000000000000000000000000000000000000000000000000000000000");

pub const CONFIG_LOCK_ARGS: &str = "0x0000000000000000000000000000000000000000";
pub const DAS_WALLET_LOCK_ARGS: &str = "0x0300000000000000000000000000000000000000";
pub const QUOTE_LOCK_ARGS: &str = "0x0100000000000000000000000000000000000000";
pub const PROFIT_LOCK_ARGS: &str = "0x0400000000000000000000000000000000000000";
pub const DUMMY_LOCK_ARGS: &str = "0xff00000000000000000000000000000000000000";

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Source {
    Input = 1,
    Output = 2,
    CellDep = 3,
}

#[derive(Debug)]
#[repr(u8)]
pub enum ScriptHashType {
    Data = 0,
    Type = 1,
}

lazy_static! {
    pub static ref TYPE_ID_TABLE: HashMap<&'static str, &'static str> = {
        // For calculation of these type ID, you need uncomment a line of debug code in the funtion **mock_contract** in src/util/template_parser .
        //
        // CAREFUL! There may be some error in the map, but the contracts will still work. It is because when parsing scripts in cell_deps, their type
        // ID will be calculated dynamically and insert into the map.
        let mut map = HashMap::new();
        // fake locks
        map.insert(
            "fake-das-lock",
            "0xebd2ca43797df1eae21f5a0d20a09a3851beab063ca06d7b86a1e1e8ef9c7698",
        );
        map.insert(
            "fake-secp256k1-blake160-signhash-all",
            "0x8f2d7cb06512f2777207461d100b0562b0213232a1bd70261e57f37fdc61483d",
        );
        map.insert(
            "always_success",
            "0x34f052fc455fce7c71f4905f223653a5fbe64261c6b2537124de00f1d52820e9",
        );
        map.insert(
            "always-success",
            "0x610b14e8060fca49a46606bf2eaaa01f77a77daf27c22a3bec3cd13c6ceb1a60",
        );
        // types
        map.insert(
            "playground",
            "0xca4d966895b1467702bad4038396b037d8c8f045cae9cf5a7db4eadefa347887",
        );
        map
    };
    pub static ref RE_VARIABLE: Regex = Regex::new(r"\{\{([\w\-\.]+)\}\}").unwrap();
    pub static ref RE_ZH_CHAR: Regex = Regex::new(r"^[\u4E00-\u9FA5]+$").unwrap();
}
