//! LGO library entry point. Exists so integration tests under `tests/` can
//! reach the resolver, gearstats reader, and gear types directly. The
//! `lgo` binary still lives in `src/main.rs`.

#![allow(dead_code)]

pub mod base_stats;
pub mod build_db;
pub mod gear;
pub mod gearstats;
pub mod optimizer;
pub mod plugindata;
pub mod report;
pub mod report_files;
pub mod slot_resolver;
pub mod stat;
pub mod virtues;
