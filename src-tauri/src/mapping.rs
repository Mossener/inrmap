//! 映射模块 - 按键映射逻辑

use std::collections::HashMap;

/// 映射表类型
pub type Mapping = HashMap<u32, u32>;

/// 从 HashMap 创建映射表
pub fn create_from_hashmap(map: &HashMap<u32, u32>) -> Mapping {
    map.clone()
}

/// 查询映射
pub fn lookup(map: &Mapping, vk: u32) -> Option<u32> {
    map.get(&vk).copied()
}
