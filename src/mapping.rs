//! 映射模块 - 按键映射逻辑

use std::collections::HashMap;

/// 映射表类型
pub type Mapping = HashMap<u32, u32>;

/// 从 HashMap 创建映射表
pub fn create_from_hashmap(map: &HashMap<u32, u32>) -> Mapping {
    map.clone()
}

/// 常用 VK 码
pub mod vk {
    use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;
    
    pub const VK_A: u32 = 0x41;
    pub const VK_B: u32 = 0x42;
    pub const VK_C: u32 = 0x43;
    pub const VK_ESCAPE: u32 = 0x1B;
    
    pub fn vk_capital() -> u32 {
        VIRTUAL_KEY(0x14).0 as u32
    }
}

/// 按键名称转 VK 码
pub fn name_to_vk(name: &str) -> Option<u32> {
    match name.to_lowercase().as_str() {
        "a" => Some(vk::VK_A),
        "b" => Some(vk::VK_B),
        "c" => Some(vk::VK_C),
        "escape" | "esc" => Some(vk::VK_ESCAPE),
        "capslock" => Some(vk::vk_capital()),
        "enter" => Some(0x0D),
        "space" => Some(0x20),
        "tab" => Some(0x09),
        _ => None,
    }
}

/// 根据配置创建映射表
pub fn create_mapping(config: &crate::config::Config) -> Mapping {
    let mut map = Mapping::new();
    
    for (from, to) in &config.mappings {
        if let (Some(from_vk), Some(to_vk)) = (name_to_vk(from), name_to_vk(to)) {
            map.insert(from_vk, to_vk);
        }
    }
    
    map
}

/// 查询映射
pub fn lookup(map: &Mapping, vk: u32) -> Option<u32> {
    map.get(&vk).copied()
}