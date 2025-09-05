use std::fs;

use anyhow::Result;
use diesel::{prelude::*, sqlite::SqliteConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use dotenvy::dotenv;
use crate::models::{Profile, ProfileDb, ProfileType, profiles};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// 自定义错误类型
#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    Connection(#[from] diesel::ConnectionError),
    #[error("Database query error: {0}")]
    Query(#[from] diesel::result::Error),
    #[error("Migration error: {0}")]
    Migration(String),
    #[error("Profile not found")]
    ProfileNotFound,
    #[error("UUID parse error: {0}")]
    UuidParse(#[from] uuid::Error),
}

/// 数据库连接管理器
pub struct DatabaseConnection {
    conn: SqliteConnection,
}

impl DatabaseConnection {
    /// 新建数据库连接
    pub fn establish() -> Result<Self, DatabaseError> {
        dotenv().ok();

        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            // 确保数据目录存在
            if let Some(parent) = std::path::Path::new("data/profiles.db").parent() {
                fs::create_dir_all(parent).unwrap_or_else(|e| {
                    eprintln!("ℹ️ 创建数据目录失败: {}", e);
                });
            }
            "data/profiles.db".to_string()
        });

        let mut conn = SqliteConnection::establish(&database_url)?;

        // 运行迁移
        MigrationHarness::run_pending_migrations(&mut conn, MIGRATIONS)
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        println!("✅ 数据库连接成功: {}", database_url);

        Ok(Self { conn })
    }

    /// 获取可变连接引用
    pub fn connection(&mut self) -> &mut SqliteConnection { &mut self.conn }
}

/// 数据库管理器（业务逻辑层）
pub struct DatabaseManager {
    db_conn: DatabaseConnection,
}

impl DatabaseManager {
    /// 初始化数据库连接
    pub fn new() -> Result<Self, DatabaseError> {
        let db_conn = DatabaseConnection::establish()?;
        Ok(Self { db_conn })
    }

    /// 保存Profile到数据库
    pub fn save_profile(&mut self, profile: &Profile) -> Result<(), DatabaseError> {
        let db_profile = profile.to_db();
        diesel::insert_into(profiles::table)
            .values(&db_profile)
            .execute(self.db_conn.connection())?;
        Ok(())
    }

    /// 获取所有Profile
    pub fn get_profiles(&mut self) -> Result<Vec<Profile>, DatabaseError> {
        let db_profiles: Vec<ProfileDb> = profiles::table.load(self.db_conn.connection())?;

        let mut profiles = Vec::new();
        for db_profile in db_profiles {
            match Profile::from_db(db_profile) {
                Ok(profile) => profiles.push(profile),
                Err(e) => eprintln!("⚠️ 解析Profile失败: {}", e),
            }
        }

        Ok(profiles)
    }

    /// 根据ID获取Profile
    pub fn get_profile(&mut self, id: &uuid::Uuid) -> Result<Option<Profile>, DatabaseError> {
        let db_profile: Option<ProfileDb> = profiles::table
            .find(id.to_string())
            .first(self.db_conn.connection())
            .optional()?;

        match db_profile {
            Some(db_prof) => {
                let profile =
                    Profile::from_db(db_prof).map_err(|_| DatabaseError::ProfileNotFound)?;
                Ok(Some(profile))
            }
            None => Ok(None),
        }
    }

    /// 更新Profile
    pub fn update_profile(&mut self, profile: &Profile) -> Result<(), DatabaseError> {
        let db_profile = profile.to_db();
        let profile_id = profile.id.to_string();
        let target = profiles::table.find(&profile_id);
        diesel::update(target)
            .set(&db_profile)
            .execute(self.db_conn.connection())?;
        Ok(())
    }

    /// 删除Profile
    pub fn delete_profile(&mut self, id: &uuid::Uuid) -> Result<(), DatabaseError> {
        let target = profiles::table.find(id.to_string());
        diesel::delete(target).execute(self.db_conn.connection())?;
        Ok(())
    }

    /// 按名称搜索Profile
    pub fn search_profiles_by_name(
        &mut self,
        name_pattern: &str,
    ) -> Result<Vec<Profile>, DatabaseError> {
        let db_profiles: Vec<ProfileDb> = profiles::table
            .filter(profiles::name.like(format!("%{}%", name_pattern)))
            .load(self.db_conn.connection())?;

        let mut profiles = Vec::new();
        for db_profile in db_profiles {
            match Profile::from_db(db_profile) {
                Ok(profile) => profiles.push(profile),
                Err(e) => eprintln!("⚠️ 解析Profile失败: {}", e),
            }
        }

        Ok(profiles)
    }

    /// 按类型筛选Profile
    pub fn get_profiles_by_type(
        &mut self,
        profile_type: ProfileType,
    ) -> Result<Vec<Profile>, DatabaseError> {
        let type_str = profile_type.to_string();
        let db_profiles: Vec<ProfileDb> = profiles::table
            .filter(profiles::profile_type.eq(type_str))
            .load(self.db_conn.connection())?;

        let mut profiles = Vec::new();
        for db_profile in db_profiles {
            match Profile::from_db(db_profile) {
                Ok(profile) => profiles.push(profile),
                Err(e) => eprintln!("⚠️ 解析Profile失败: {}", e),
            }
        }

        Ok(profiles)
    }
}
