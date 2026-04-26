#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod hook;
mod mapping;

use crate::database::{Database, MappingEntry, Profile};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, State,
};

struct AppState {
    db: Database,
    current_profile_id: RwLock<i64>,
    mappings: RwLock<HashMap<u32, u32>>,
    enabled: RwLock<bool>,
    vk_names: HashMap<String, u32>,
}

impl AppState {
    fn new() -> Self {
        let db = Database::new().expect("数据库初始化失败");
        
        let profiles = db.get_all_profiles().unwrap_or_default();
        let current_profile_id = profiles.first().map(|p| p.id).unwrap_or(1);
        
        let mut vk_names: HashMap<String, u32> = HashMap::new();
        Self::init_vk_names(&mut vk_names);
        
        let mut mappings = HashMap::new();
        if let Some(profile) = profiles.first() {
            for m in &profile.mappings {
                if let (Some(from_vk), Some(to_vk)) = (
                    name_to_vk(&m.from, &vk_names),
                    name_to_vk(&m.to, &vk_names),
                ) {
                    mappings.insert(from_vk, to_vk);
                }
            }
        }
        
        hook::set_mapping(mapping::create_from_hashmap(&mappings));
        
        Self {
            db,
            current_profile_id: RwLock::new(current_profile_id),
            mappings: RwLock::new(mappings),
            enabled: RwLock::new(true),
            vk_names,
        }
    }
    
    fn init_vk_names(vk_names: &mut HashMap<String, u32>) {
        for i in 0..26 {
            let c = (b'A' + i) as char;
            vk_names.insert(c.to_string(), 0x41 + i as u32);
        }
        for i in 0..10 {
            vk_names.insert(i.to_string(), 0x30 + i as u32);
        }
        vk_names.insert("Space".to_string(), 0x20);
        vk_names.insert("Enter".to_string(), 0x0D);
        vk_names.insert("Tab".to_string(), 0x09);
        vk_names.insert("Escape".to_string(), 0x1B);
        vk_names.insert("Shift".to_string(), 0x10);
        vk_names.insert("Ctrl".to_string(), 0x11);
        vk_names.insert("Alt".to_string(), 0x12);
        vk_names.insert("Left".to_string(), 0x25);
        vk_names.insert("Up".to_string(), 0x26);
        vk_names.insert("Right".to_string(), 0x27);
        vk_names.insert("Down".to_string(), 0x28);
        vk_names.insert("F1".to_string(), 0x70);
        vk_names.insert("F2".to_string(), 0x71);
        vk_names.insert("F3".to_string(), 0x72);
        vk_names.insert("F4".to_string(), 0x73);
        vk_names.insert("F5".to_string(), 0x74);
        vk_names.insert("F6".to_string(), 0x75);
        vk_names.insert("F7".to_string(), 0x76);
        vk_names.insert("F8".to_string(), 0x77);
        vk_names.insert("F9".to_string(), 0x78);
        vk_names.insert("F10".to_string(), 0x79);
        vk_names.insert("F11".to_string(), 0x7A);
        vk_names.insert("F12".to_string(), 0x7B);
        vk_names.insert("无".to_string(), 0x00);
    }
    
    fn save_mappings(&self) {
        let profile_id = *self.current_profile_id.read().unwrap();
        let vk_to_name: HashMap<u32, String> = self.vk_names.iter()
            .map(|(k, v)| (*v, k.clone()))
            .collect();
        
        let entries: Vec<MappingEntry> = self.mappings.read().unwrap()
            .iter()
            .filter_map(|(from, to)| {
                let from_name = vk_to_name.get(from)?.clone();
                let to_name = vk_to_name.get(to)?.clone();
                Some(MappingEntry { from: from_name, to: to_name })
            })
            .collect();
        
        if let Err(e) = self.db.save_mappings(profile_id, &entries) {
            eprintln!("保存映射失败: {}", e);
        }
    }
}

fn name_to_vk(name: &str, vk_names: &HashMap<String, u32>) -> Option<u32> {
    // 先检查 HashMap（包括 "无" 等多字符名称）
    if let Some(&vk) = vk_names.get(name) {
        return Some(vk);
    }
    // 再处理单字符输入
    if name.len() == 1 {
        let c = name.chars().next().unwrap();
        if c.is_ascii_alphabetic() {
            return Some(c.to_ascii_uppercase() as u32);
        }
        if c.is_ascii_digit() {
            return Some(c as u32);
        }
    }
    None
}

#[derive(Serialize, Deserialize)]
struct Mapping {
    from: String,
    to: String,
}

#[tauri::command]
fn get_profiles(state: State<AppState>) -> Result<Vec<Profile>, String> {
    state.db.get_all_profiles().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_current_profile(state: State<AppState>) -> i64 {
    *state.current_profile_id.read().unwrap()
}

#[tauri::command]
fn switch_profile(id: i64, state: State<AppState>) -> Result<(), String> {
    if let Some(profile) = state.db.get_profile(id).map_err(|e| e.to_string())? {
        {
            let mut mappings = state.mappings.write().unwrap();
            mappings.clear();
            
            for m in &profile.mappings {
                if let (Some(from_vk), Some(to_vk)) = (
                    name_to_vk(&m.from, &state.vk_names),
                    name_to_vk(&m.to, &state.vk_names),
                ) {
                    mappings.insert(from_vk, to_vk);
                }
            }
            
            hook::set_mapping(mapping::create_from_hashmap(&mappings));
        }
        *state.current_profile_id.write().unwrap() = id;
    }
    Ok(())
}

#[tauri::command]
fn create_profile(name: String, state: State<AppState>) -> Result<i64, String> {
    state.db.create_profile(&name).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_profile(id: i64, state: State<AppState>) -> Result<(), String> {
    let profiles = state.db.get_all_profiles().map_err(|e| e.to_string())?;
    if profiles.len() <= 1 {
        return Err("至少需要保留一个配置".to_string());
    }
    
    state.db.delete_profile(id).map_err(|e| e.to_string())?;
    
    if *state.current_profile_id.read().unwrap() == id {
        if let Some(first) = profiles.first() {
            if first.id != id {
                let _ = switch_profile(first.id, state);
            } else if profiles.len() > 1 {
                let _ = switch_profile(profiles[1].id, state);
            }
        }
    }
    
    Ok(())
}

#[tauri::command]
fn get_mappings(state: State<AppState>) -> Vec<Mapping> {
    let vk_to_name: HashMap<u32, String> = state.vk_names.iter()
        .map(|(k, v)| (*v, k.clone()))
        .collect();
    
    state.mappings.read().unwrap()
        .iter()
        .filter_map(|(from, to)| {
            let from_name = vk_to_name.get(from).cloned()?;
            let to_name = if *to == 0 {
                "无".to_string()
            } else {
                vk_to_name.get(to).cloned()?
            };
            Some(Mapping { from: from_name, to: to_name })
        })
        .collect()
}

#[tauri::command]
fn add_mapping(from: String, to: String, state: State<AppState>) -> Result<(), String> {
    let from_vk = name_to_vk(&from, &state.vk_names).ok_or_else(|| format!("未知按键: {}", from))?;
    let to_vk = name_to_vk(&to, &state.vk_names).ok_or_else(|| format!("未知目标按键: {}", to))?;

    {
        let mut mappings = state.mappings.write().unwrap();
        mappings.insert(from_vk, to_vk);
        hook::set_mapping(mapping::create_from_hashmap(&mappings));
    }
    
    state.save_mappings();
    Ok(())
}

#[tauri::command]
fn remove_mapping(from: String, state: State<AppState>) -> Result<(), String> {
    let from_vk = name_to_vk(&from, &state.vk_names).ok_or_else(|| format!("未知按键: {}", from))?;

    {
        let mut mappings = state.mappings.write().unwrap();
        mappings.remove(&from_vk);
        hook::set_mapping(mapping::create_from_hashmap(&mappings));
    }

    state.save_mappings();
    Ok(())
}

#[tauri::command]
fn set_enabled(enabled: bool, state: State<AppState>) {
    *state.enabled.write().unwrap() = enabled;
    hook::set_enabled(enabled);
}

#[tauri::command]
fn is_enabled(state: State<AppState>) -> bool {
    *state.enabled.read().unwrap()
}

fn main() {
    let app_state = AppState::new();

    if let Err(e) = hook::init() {
        eprintln!("钩子初始化失败: {}", e);
    }

    tauri::Builder::default()
        .manage(app_state)
        .setup(|app| {
            // 创建托盘菜单 - 使用勾选框
            let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let enabled_item = MenuItem::with_id(app, "toggle", "映射切换", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            
            let menu = Menu::with_items(app, &[&show, &enabled_item, &quit])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().unwrap())
                .menu(&menu)
                .tooltip("inR Remapper")
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "toggle" => {
                            if let Some(state) = app.try_state::<AppState>() {
                                let current = *state.enabled.read().unwrap();
                                let new_enabled = !current;
                                *state.enabled.write().unwrap() = new_enabled;
                                hook::set_enabled(new_enabled);
                            }
                        }
                        "quit" => {
                            hook::cleanup();
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // 窗口关闭时隐藏到托盘而不是退出
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_profiles,
            get_current_profile,
            switch_profile,
            create_profile,
            delete_profile,
            get_mappings,
            add_mapping,
            remove_mapping,
            set_enabled,
            is_enabled
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri");
}
