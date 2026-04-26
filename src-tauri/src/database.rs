//! 数据库模块 - 配置持久化

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub id: i64,
    pub name: String,
    pub mappings: Vec<MappingEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingEntry {
    pub from: String,
    pub to: String,
}

pub struct Database {
    conn: Mutex<Connection>,
}

unsafe impl Send for Database {}
unsafe impl Sync for Database {}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path();
        
        // 确保目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        
        let conn = Connection::open(&db_path)?;
        let db = Self { conn: Mutex::new(conn) };
        db.init_tables()?;
        Ok(db)
    }
    
    fn get_db_path() -> PathBuf {
        let base = if cfg!(windows) {
            std::env::var("APPDATA")
                .ok()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
        } else {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".config"))
                .unwrap_or_else(|| PathBuf::from("."))
        };
        
        base.join("inR_Remapper").join("config.db")
    }
    
    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS profiles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS mappings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                profile_id INTEGER NOT NULL,
                from_key TEXT NOT NULL,
                to_key TEXT NOT NULL,
                FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE
            )",
            [],
        )?;
        
        // 确保有默认配置
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM profiles",
            [],
            |row| row.get(0),
        )?;
        
        if count == 0 {
            conn.execute("INSERT INTO profiles (name) VALUES ('默认配置')", [])?;
        }
        
        Ok(())
    }
    
    pub fn get_all_profiles(&self) -> Result<Vec<Profile>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name FROM profiles ORDER BY id")?;
        let profiles = stmt.query_map([], |row| {
            Ok(Profile {
                id: row.get(0)?,
                name: row.get(1)?,
                mappings: vec![],
            })
        })?;
        
        let mut result = Vec::new();
        for profile in profiles {
            let mut p = profile?;
            p.mappings = Self::get_mappings_by_profile_internal(&conn, p.id)?;
            result.push(p);
        }
        
        Ok(result)
    }
    
    pub fn get_profile(&self, id: i64) -> Result<Option<Profile>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name FROM profiles WHERE id = ?")?;
        let mut rows = stmt.query([id])?;
        
        if let Some(row) = rows.next()? {
            let profile = Profile {
                id: row.get(0)?,
                name: row.get(1)?,
                mappings: Self::get_mappings_by_profile_internal(&conn, id)?,
            };
            Ok(Some(profile))
        } else {
            Ok(None)
        }
    }
    
    fn get_mappings_by_profile_internal(conn: &Connection, profile_id: i64) -> Result<Vec<MappingEntry>> {
        let mut stmt = conn.prepare("SELECT from_key, to_key FROM mappings WHERE profile_id = ?")?;
        let mappings = stmt.query_map([profile_id], |row| {
            Ok(MappingEntry {
                from: row.get(0)?,
                to: row.get(1)?,
            })
        })?;
        
        let mut result = Vec::new();
        for m in mappings {
            result.push(m?);
        }
        Ok(result)
    }
    
    pub fn create_profile(&self, name: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT INTO profiles (name) VALUES (?)", [name])?;
        Ok(conn.last_insert_rowid())
    }
    
    pub fn delete_profile(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM mappings WHERE profile_id = ?", [id])?;
        conn.execute("DELETE FROM profiles WHERE id = ?", [id])?;
        Ok(())
    }
    
    pub fn save_mappings(&self, profile_id: i64, mappings: &[MappingEntry]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        // 删除旧映射
        conn.execute("DELETE FROM mappings WHERE profile_id = ?", [profile_id])?;
        
        // 插入新映射
        for m in mappings {
            conn.execute(
                "INSERT INTO mappings (profile_id, from_key, to_key) VALUES (?, ?, ?)",
                rusqlite::params![profile_id, m.from, m.to],
            )?;
        }
        
        Ok(())
    }
}
