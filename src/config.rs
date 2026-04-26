//! 配置模块 - 读取配置文件

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

/// 配置文件结构
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub mappings: HashMap<String, String>,
}

/// 读取配置文件
pub fn load(path: &str) -> anyhow::Result<Config> {
    let content = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}