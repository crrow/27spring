// Copyright 2025 Crrow
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fs;

use anyhow::Result;
use chrono::{DateTime, Utc};
use dialoguer::{Confirm, Input, Select};
use diesel::{prelude::*, sqlite::SqliteConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use dotenvy::dotenv;
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use tabled::{
    Table, Tabled,
    settings::{Alignment, Modify, Style, object::Columns},
};
use uuid::Uuid;

// Diesel schema definition
diesel::table! {
    profiles (id) {
        id -> Text,
        name -> Text,
        profile_type -> Text,
        location_country -> Text,
        location_city -> Nullable<Text>,
        location_currency -> Text,
        work_start_delay -> Integer,
        work_duration_limit -> Nullable<Integer>,
        initial_salary_usd -> Double,
        salary_growth_rate -> Double,
        living_cost_usd -> Double,
        living_cost_growth -> Double,
        tax_rate -> Double,
        total_cost_usd -> Nullable<Double>,
        cost_duration -> Nullable<Integer>,
        first_year_opportunity_cost -> Nullable<Double>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        description -> Nullable<Text>,
    }
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Profile类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProfileType {
    Education, // 留学/教育路径
    Work,      // 工作路径
}

impl ProfileType {
    /// 转换为数据库字符串
    pub fn to_string(&self) -> String {
        match self {
            ProfileType::Education => "Education".to_string(),
            ProfileType::Work => "Work".to_string(),
        }
    }

    /// 从数据库字符串解析
    pub fn from_string(s: &str) -> Result<Self> {
        match s {
            "Education" => Ok(ProfileType::Education),
            "Work" => Ok(ProfileType::Work),
            _ => Err(anyhow::anyhow!("Invalid profile type: {}", s)),
        }
    }
}

/// 地理位置信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Location {
    pub country:  String,
    pub city:     Option<String>,
    pub currency: String,
}

/// 用于数据库操作的Profile结构
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = profiles)]
pub struct ProfileDb {
    pub id: String,
    pub name: String,
    pub profile_type: String,
    pub location_country: String,
    pub location_city: Option<String>,
    pub location_currency: String,
    pub work_start_delay: i32,
    pub work_duration_limit: Option<i32>,
    pub initial_salary_usd: f64,
    pub salary_growth_rate: f64,
    pub living_cost_usd: f64,
    pub living_cost_growth: f64,
    pub tax_rate: f64,
    pub total_cost_usd: Option<f64>,
    pub cost_duration: Option<i32>,
    pub first_year_opportunity_cost: Option<f64>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub description: Option<String>,
}

/// 职业发展路径Profile（业务逻辑层）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id:           Uuid,
    pub name:         String,
    pub profile_type: ProfileType,
    pub location:     Location,

    // 工作年限参数
    pub work_start_delay:    u32,         // 开始工作的延迟年数
    pub work_duration_limit: Option<u32>, // 工作年限限制

    // 财务参数
    pub initial_salary_usd: f64, // 初始薪资(USD)
    pub salary_growth_rate: f64, // 薪资增长率
    pub living_cost_usd:    f64, // 初始生活成本(USD)
    pub living_cost_growth: f64, // 生活成本增长率
    pub tax_rate:           f64, // 税率

    // 成本参数（如学费）
    pub total_cost_usd: Option<f64>, // 总成本(USD)
    pub cost_duration:  Option<u32>, // 成本分摊年数

    // 投资参数
    pub first_year_opportunity_cost: Option<f64>, // 第一年机会成本投资

    // 元数据
    pub created_at:  DateTime<Utc>,
    pub updated_at:  DateTime<Utc>,
    pub description: Option<String>,
}

impl Profile {
    /// 从数据库记录转换为业务对象
    pub fn from_db(db_profile: ProfileDb) -> Result<Self> {
        let profile_type = ProfileType::from_string(&db_profile.profile_type)?;
        let id = Uuid::parse_str(&db_profile.id)?;

        Ok(Profile {
            id,
            name: db_profile.name,
            profile_type,
            location: Location {
                country:  db_profile.location_country,
                city:     db_profile.location_city,
                currency: db_profile.location_currency,
            },
            work_start_delay: db_profile.work_start_delay as u32,
            work_duration_limit: db_profile.work_duration_limit.map(|x| x as u32),
            initial_salary_usd: db_profile.initial_salary_usd,
            salary_growth_rate: db_profile.salary_growth_rate,
            living_cost_usd: db_profile.living_cost_usd,
            living_cost_growth: db_profile.living_cost_growth,
            tax_rate: db_profile.tax_rate,
            total_cost_usd: db_profile.total_cost_usd,
            cost_duration: db_profile.cost_duration.map(|x| x as u32),
            first_year_opportunity_cost: db_profile.first_year_opportunity_cost,
            created_at: DateTime::from_naive_utc_and_offset(db_profile.created_at, Utc),
            updated_at: DateTime::from_naive_utc_and_offset(db_profile.updated_at, Utc),
            description: db_profile.description,
        })
    }

    /// 转换为数据库记录
    pub fn to_db(&self) -> ProfileDb {
        ProfileDb {
            id: self.id.to_string(),
            name: self.name.clone(),
            profile_type: self.profile_type.to_string(),
            location_country: self.location.country.clone(),
            location_city: self.location.city.clone(),
            location_currency: self.location.currency.clone(),
            work_start_delay: self.work_start_delay as i32,
            work_duration_limit: self.work_duration_limit.map(|x| x as i32),
            initial_salary_usd: self.initial_salary_usd,
            salary_growth_rate: self.salary_growth_rate,
            living_cost_usd: self.living_cost_usd,
            living_cost_growth: self.living_cost_growth,
            tax_rate: self.tax_rate,
            total_cost_usd: self.total_cost_usd,
            cost_duration: self.cost_duration.map(|x| x as i32),
            first_year_opportunity_cost: self.first_year_opportunity_cost,
            created_at: self.created_at.naive_utc(),
            updated_at: self.updated_at.naive_utc(),
            description: self.description.clone(),
        }
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize, SmartDefault)]
pub struct ROICalculator {
    // 基础参数
    #[default = 7.2]
    pub exchange_rate:          f64, // 汇率 USD:CNY
    #[default = 0.10]
    pub investment_return_rate: f64, // S&P 500 年化回报率
    #[default = 0.20]
    pub investment_portion:     f64, // 可支配收入中用于投资的比例（20%）
    #[default = 2]
    pub asu_duration:           u32, // ASU学习年限
    #[default = 10]
    pub total_years:            u32, // 总分析年限
    #[default = 8]
    pub shanghai_work_limit:    u32, // 上海工作年限限制

    // ASU相关参数
    #[default = 131081.81]
    pub asu_total_cost_usd:     f64, // ASU总成本（美元）
    #[default = 90000.0]
    pub asu_initial_salary_usd: f64, // ASU初始薪资（美元）
    #[default = 0.05]
    pub asu_salary_growth_rate: f64, // ASU薪资增长率
    #[default = 35000.0]
    pub asu_living_cost_usd:    f64, // ASU初始生活成本（美元）
    #[default = 0.0273]
    pub asu_living_cost_growth: f64, // ASU生活成本增长率
    #[default = 0.28]
    pub asu_tax_rate:           f64, // 美国税率

    // 上海相关参数
    #[default = 55555.56]
    pub shanghai_initial_salary_usd: f64, // 上海初始薪资（美元）
    #[default = 0.03]
    pub shanghai_salary_growth:      f64, // 上海薪资增长率
    #[default = 16666.67]
    pub shanghai_living_cost_usd:    f64, // 上海初始生活成本（美元）
    #[default = 0.003]
    pub shanghai_living_cost_growth: f64, // 上海生活成本增长率
    #[default = 0.30]
    pub shanghai_tax_rate:           f64, // 上海税率
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathYearlyData {
    pub year: u32,
    pub work_year: Option<u32>,
    pub income_usd: f64,
    pub net_income_usd: f64,
    pub living_cost_usd: f64,
    pub disposable_income_usd: f64,
    pub cash_savings: f64,
    pub investment_amount: f64,
    pub investment_return: f64,
    pub total_investment: f64,
    pub total_investment_principal: f64, // 累计投资本金
    pub total_cash: f64,
    pub net_worth: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonData {
    pub year:          u32,
    pub asu_data:      PathYearlyData,
    pub shanghai_data: PathYearlyData,
}

/// Profile比较数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileComparisonData {
    pub year:          u32,
    pub profile1_data: PathYearlyData,
    pub profile2_data: PathYearlyData,
    pub profile1_name: String,
    pub profile2_name: String,
}

/// 路径计算参数
#[derive(Debug, Clone)]
pub struct PathCalculationParams {
    // 工作年限计算参数
    pub work_start_delay:    u32,         // 开始工作的延迟年数
    pub work_duration_limit: Option<u32>, // 工作年限限制（None表示无限制）

    // 财务参数
    pub initial_salary_usd: f64, // 初始薪资
    pub salary_growth_rate: f64, // 薪资增长率
    pub living_cost_usd:    f64, // 初始生活成本
    pub living_cost_growth: f64, // 生活成本增长率
    pub tax_rate:           f64, // 税率

    // 成本参数
    pub total_cost_usd: Option<f64>, // 总成本（如学费）
    pub cost_duration:  Option<u32>, // 成本分摊年数

    // 投资参数
    pub first_year_opportunity_cost: Option<f64>, // 第一年机会成本投资
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
    pub fn get_profile(&mut self, id: &Uuid) -> Result<Option<Profile>, DatabaseError> {
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
    pub fn delete_profile(&mut self, id: &Uuid) -> Result<(), DatabaseError> {
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

impl Profile {
    /// 将Profile转换为PathCalculationParams
    pub fn to_path_params(&self) -> PathCalculationParams {
        PathCalculationParams {
            work_start_delay:            self.work_start_delay,
            work_duration_limit:         self.work_duration_limit,
            initial_salary_usd:          self.initial_salary_usd,
            salary_growth_rate:          self.salary_growth_rate,
            living_cost_usd:             self.living_cost_usd,
            living_cost_growth:          self.living_cost_growth,
            tax_rate:                    self.tax_rate,
            total_cost_usd:              self.total_cost_usd,
            cost_duration:               self.cost_duration,
            first_year_opportunity_cost: self.first_year_opportunity_cost,
        }
    }
}

/// 交互式Profile创建器
pub struct ProfileCreator {
    db: DatabaseManager,
}

impl ProfileCreator {
    pub fn new(db: DatabaseManager) -> Self { Self { db } }

    /// 创建新的Profile
    pub fn create_profile(&mut self) -> Result<Profile> {
        println!("\n🎯 创建新的职业发展路径Profile");
        println!("=====================================");

        // 基本信息
        let name: String = Input::new()
            .with_prompt("Profile名称")
            .with_initial_text("我的职业路径")
            .interact_text()?;

        let profile_types = vec!["留学/教育路径", "工作路径"];
        let profile_type_idx = Select::new()
            .with_prompt("选择路径类型")
            .items(&profile_types)
            .default(0)
            .interact()?;

        let profile_type = match profile_type_idx {
            0 => ProfileType::Education,
            1 => ProfileType::Work,
            _ => ProfileType::Work,
        };

        // 地理位置
        let country: String = Input::new()
            .with_prompt("国家")
            .with_initial_text("United States")
            .interact_text()?;

        let city: String = Input::new()
            .with_prompt("城市 (可选)")
            .allow_empty(true)
            .interact_text()?;

        let currency: String = Input::new()
            .with_prompt("货币代码")
            .with_initial_text("USD")
            .interact_text()?;

        // 工作年限参数
        let work_start_delay: u32 = Input::new()
            .with_prompt("开始工作前的延迟年数 (如留学年数)")
            .with_initial_text("0")
            .interact_text()?;

        let has_work_limit = Confirm::new()
            .with_prompt("是否有工作年限限制?")
            .default(false)
            .interact()?;

        let work_duration_limit = if has_work_limit {
            Some(
                Input::new()
                    .with_prompt("工作年限限制")
                    .with_initial_text("10")
                    .interact_text()?,
            )
        } else {
            None
        };

        // 财务参数
        let initial_salary_usd: f64 = Input::new()
            .with_prompt("初始年薪 (USD)")
            .with_initial_text("50000")
            .interact_text()?;

        let salary_growth_rate: f64 = Input::new()
            .with_prompt("年薪增长率 (小数, 如0.03表示3%)")
            .with_initial_text("0.03")
            .interact_text()?;

        let living_cost_usd: f64 = Input::new()
            .with_prompt("初始年生活成本 (USD)")
            .with_initial_text("30000")
            .interact_text()?;

        let living_cost_growth: f64 = Input::new()
            .with_prompt("生活成本年增长率 (小数)")
            .with_initial_text("0.025")
            .interact_text()?;

        let tax_rate: f64 = Input::new()
            .with_prompt("税率 (小数, 如0.25表示25%)")
            .with_initial_text("0.25")
            .interact_text()?;

        // 成本参数
        let has_costs = Confirm::new()
            .with_prompt("是否有初期成本 (如学费)?")
            .default(profile_type_idx == 0)
            .interact()?;

        let (total_cost_usd, cost_duration) = if has_costs {
            let cost: f64 = Input::new()
                .with_prompt("总成本 (USD)")
                .with_initial_text("100000")
                .interact_text()?;
            let duration: u32 = Input::new()
                .with_prompt("成本分摊年数")
                .with_initial_text("2")
                .interact_text()?;
            (Some(cost), Some(duration))
        } else {
            (None, None)
        };

        // 投资参数
        let has_opportunity_cost = Confirm::new()
            .with_prompt("第一年是否有机会成本投资?")
            .default(profile_type_idx == 1)
            .interact()?;

        let first_year_opportunity_cost = if has_opportunity_cost {
            Some(
                Input::new()
                    .with_prompt("第一年机会成本投资金额 (USD)")
                    .with_initial_text("100000")
                    .interact_text()?,
            )
        } else {
            None
        };

        // 描述
        let description: String = Input::new()
            .with_prompt("描述 (可选)")
            .allow_empty(true)
            .interact_text()?;

        let now = Utc::now();
        let profile = Profile {
            id: Uuid::new_v4(),
            name,
            profile_type,
            location: Location {
                country,
                city: if city.is_empty() { None } else { Some(city) },
                currency,
            },
            work_start_delay,
            work_duration_limit,
            initial_salary_usd,
            salary_growth_rate,
            living_cost_usd,
            living_cost_growth,
            tax_rate,
            total_cost_usd,
            cost_duration,
            first_year_opportunity_cost,
            created_at: now,
            updated_at: now,
            description: if description.is_empty() {
                None
            } else {
                Some(description)
            },
        };

        // 保存到数据库
        self.db
            .save_profile(&profile)
            .map_err(|e| anyhow::Error::new(e))?;

        println!("\n✅ Profile '{}' 已成功创建!", profile.name);
        Ok(profile)
    }

    /// 列出所有Profile
    pub fn list_profiles(&mut self) -> Result<()> {
        let profiles = self.db.get_profiles().map_err(|e| anyhow::Error::new(e))?;

        if profiles.is_empty() {
            println!("📭 暂无Profile，请先创建一个。");
            return Ok(());
        }

        println!("\n📋 已保存的Profile列表:");
        println!("=======================");

        for (i, profile) in profiles.iter().enumerate() {
            let type_str = match profile.profile_type {
                ProfileType::Education => "🎓 教育",
                ProfileType::Work => "💼 工作",
            };

            let location_str = if let Some(city) = &profile.location.city {
                format!("{}, {}", city, profile.location.country)
            } else {
                profile.location.country.clone()
            };

            println!(
                "{}. {} | {} | {} | ${:.0}/年",
                i + 1,
                profile.name,
                type_str,
                location_str,
                profile.initial_salary_usd
            );
        }

        Ok(())
    }

    /// 选择Profile进行比较
    pub fn select_profiles_for_comparison(&mut self) -> Result<Vec<Profile>> {
        let profiles = self.db.get_profiles().map_err(|e| anyhow::Error::new(e))?;

        if profiles.len() < 2 {
            println!(
                "⚠️ 需要至少2个Profile才能进行比较，当前只有{}个。",
                profiles.len()
            );
            return Ok(vec![]);
        }

        let profile_names: Vec<String> = profiles
            .iter()
            .map(|p| {
                format!(
                    "{} ({})",
                    p.name,
                    match p.profile_type {
                        ProfileType::Education => "教育",
                        ProfileType::Work => "工作",
                    }
                )
            })
            .collect();

        println!("\n🔍 选择要比较的Profile:");

        let first_idx = Select::new()
            .with_prompt("选择第一个Profile")
            .items(&profile_names)
            .interact()?;

        let second_idx = Select::new()
            .with_prompt("选择第二个Profile")
            .items(&profile_names)
            .interact()?;

        if first_idx == second_idx {
            println!("⚠️ 不能选择相同的Profile进行比较");
            return Ok(vec![]);
        }

        Ok(vec![
            profiles[first_idx].clone(),
            profiles[second_idx].clone(),
        ])
    }
}

// 辅助函数：格式化货币数字
fn format_currency(amount: f64) -> String {
    if amount.abs() < 0.01 {
        "$0".to_string()
    } else if amount.abs() >= 1_000_000.0 {
        format!("${:.1}M", amount / 1_000_000.0)
    } else if amount.abs() >= 1_000.0 {
        format!("${:.1}K", amount / 1_000.0)
    } else {
        format!("${:.0}", amount)
    }
}

impl ROICalculator {
    /// 通用路径计算方法
    ///
    /// # Parameters
    /// * `params` - 路径计算参数
    ///
    /// # Returns
    /// 返回年度财务数据向量
    fn calculate_path_data(&self, params: &PathCalculationParams) -> Vec<PathYearlyData> {
        let mut results = Vec::new();
        let mut total_cash = 0.0;
        let mut total_investment = 0.0;
        let mut total_investment_principal = 0.0;
        let mut total_cost_paid = 0.0;

        for year in 1..=self.total_years {
            // 确定工作年数
            let work_year = self.get_work_year(year, params);

            // 计算年度财务数据
            let (income_usd, net_income_usd, living_cost_usd, disposable_income_usd) =
                self.calculate_year_finances(year, work_year, params, &mut total_cost_paid);

            // 计算投资分配
            let (investment_amount, cash_savings) =
                self.calculate_investment_allocation(year, disposable_income_usd, params);

            // 计算投资收益
            let (existing_return, new_investment_return) =
                self.calculate_investment_returns(total_investment, investment_amount);
            let total_return = existing_return + new_investment_return;

            // 更新投资和现金
            total_investment += total_return + investment_amount;
            total_investment_principal += investment_amount;
            total_cash += cash_savings;

            // 计算净资产
            let net_worth = self.calculate_net_worth(
                total_cash,
                total_investment,
                total_cost_paid,
                params.total_cost_usd.is_some(),
            );

            results.push(PathYearlyData {
                year,
                work_year,
                income_usd,
                net_income_usd,
                living_cost_usd,
                disposable_income_usd,
                cash_savings,
                investment_amount,
                investment_return: total_return,
                total_investment,
                total_investment_principal,
                total_cash,
                net_worth,
            });
        }

        results
    }

    /// 获取指定年份的工作年数
    fn get_work_year(&self, year: u32, params: &PathCalculationParams) -> Option<u32> {
        if year > params.work_start_delay {
            let work_year = year - params.work_start_delay;
            match params.work_duration_limit {
                Some(limit) if work_year > limit => None,
                _ => Some(work_year),
            }
        } else {
            None
        }
    }

    /// 计算年度财务数据
    fn calculate_year_finances(
        &self,
        year: u32,
        work_year: Option<u32>,
        params: &PathCalculationParams,
        total_cost_paid: &mut f64,
    ) -> (f64, f64, f64, f64) {
        if let Some(work_year) = work_year {
            // 工作期间
            let salary_usd = params.initial_salary_usd
                * (1.0 + params.salary_growth_rate).powi(work_year as i32 - 1);
            let living_cost_usd =
                params.living_cost_usd * (1.0 + params.living_cost_growth).powi(year as i32 - 1);
            let net_income_usd = salary_usd * (1.0 - params.tax_rate);
            let disposable_income_usd = (net_income_usd - living_cost_usd).max(0.0);

            (
                salary_usd,
                net_income_usd,
                living_cost_usd,
                disposable_income_usd,
            )
        } else {
            // 非工作期间（学习或退休）
            if let (Some(total_cost), Some(cost_duration)) =
                (params.total_cost_usd, params.cost_duration)
            {
                // 有学习成本的情况（如ASU）
                let annual_cost = total_cost / cost_duration as f64;
                *total_cost_paid += annual_cost;
                let living_cost_usd = params.living_cost_usd
                    * (1.0 + params.living_cost_growth).powi(year as i32 - 1);
                (0.0, 0.0, living_cost_usd + annual_cost, 0.0)
            } else {
                // 无成本的非工作期间（如退休）
                (0.0, 0.0, 0.0, 0.0)
            }
        }
    }

    /// 计算投资分配
    fn calculate_investment_allocation(
        &self,
        year: u32,
        disposable_income: f64,
        params: &PathCalculationParams,
    ) -> (f64, f64) {
        let investment_amount = if year == 1 && params.first_year_opportunity_cost.is_some() {
            // 第一年有机会成本投资
            params.first_year_opportunity_cost.unwrap()
                + disposable_income * self.investment_portion
        } else {
            // 正常投资分配
            disposable_income * self.investment_portion
        };

        let cash_savings = disposable_income - (disposable_income * self.investment_portion);
        (investment_amount, cash_savings)
    }

    /// 计算净资产
    fn calculate_net_worth(
        &self,
        total_cash: f64,
        total_investment: f64,
        total_cost_paid: f64,
        has_costs: bool,
    ) -> f64 {
        if has_costs {
            total_cash + total_investment - total_cost_paid
        } else {
            total_cash + total_investment
        }
    }

    /// 计算ASU路径的财务数据
    pub fn calculate_asu_path(&self) -> Vec<PathYearlyData> {
        let params = PathCalculationParams {
            work_start_delay:            self.asu_duration,
            work_duration_limit:         None, // ASU毕业后可以工作到分析期结束
            initial_salary_usd:          self.asu_initial_salary_usd,
            salary_growth_rate:          self.asu_salary_growth_rate,
            living_cost_usd:             self.asu_living_cost_usd,
            living_cost_growth:          self.asu_living_cost_growth,
            tax_rate:                    self.asu_tax_rate,
            total_cost_usd:              Some(self.asu_total_cost_usd),
            cost_duration:               Some(self.asu_duration),
            first_year_opportunity_cost: None, // ASU路径没有第一年机会成本投资
        };
        self.calculate_path_data(&params)
    }

    /// 计算上海路径的财务数据
    pub fn calculate_shanghai_path(&self) -> Vec<PathYearlyData> {
        let params = PathCalculationParams {
            work_start_delay:            0, // 上海路径立即开始工作
            work_duration_limit:         Some(self.shanghai_work_limit), // 上海工作年限有限制
            initial_salary_usd:          self.shanghai_initial_salary_usd,
            salary_growth_rate:          self.shanghai_salary_growth,
            living_cost_usd:             self.shanghai_living_cost_usd,
            living_cost_growth:          self.shanghai_living_cost_growth,
            tax_rate:                    self.shanghai_tax_rate,
            total_cost_usd:              None, // 上海路径没有学习成本
            cost_duration:               None,
            first_year_opportunity_cost: Some(self.asu_total_cost_usd), // 第一年投资机会成本
        };
        self.calculate_path_data(&params)
    }

    /// 计算投资收益（考虑每月定投）
    fn calculate_investment_returns(
        &self,
        existing_investment: f64,
        new_investment: f64,
    ) -> (f64, f64) {
        // 存量投资获得全年收益
        let existing_return = existing_investment * self.investment_return_rate;

        // 新投资按平均持有6个月计算（每月定投的近似）
        let new_investment_return = new_investment * self.investment_return_rate * 0.5;

        (existing_return, new_investment_return)
    }

    /// 使用Profile计算财务数据
    pub fn calculate_profile_path(&self, profile: &Profile) -> Vec<PathYearlyData> {
        let params = profile.to_path_params();
        self.calculate_path_data(&params)
    }

    /// 比较两个Profile的ROI数据
    pub fn compare_profiles(
        &self,
        profile1: &Profile,
        profile2: &Profile,
    ) -> Vec<ProfileComparisonData> {
        let data1 = self.calculate_profile_path(profile1);
        let data2 = self.calculate_profile_path(profile2);

        data1
            .into_iter()
            .zip(data2.into_iter())
            .map(|(data1, data2)| ProfileComparisonData {
                year:          data1.year,
                profile1_data: data1,
                profile2_data: data2,
                profile1_name: profile1.name.clone(),
                profile2_name: profile2.name.clone(),
            })
            .collect()
    }

    /// 计算完整的ROI比较数据 (保留原有方法用于向后兼容)
    pub fn calculate_roi(&self) -> Vec<ComparisonData> {
        let asu_data = self.calculate_asu_path();
        let shanghai_data = self.calculate_shanghai_path();

        asu_data
            .into_iter()
            .zip(shanghai_data.into_iter())
            .map(|(asu, shanghai)| ComparisonData {
                year:          asu.year,
                asu_data:      asu,
                shanghai_data: shanghai,
            })
            .collect()
    }

    pub fn calculate_final_roi(&self) -> (f64, f64, f64) {
        let results = self.calculate_roi();
        let final_data = results.last().unwrap();

        let asu_roi =
            (final_data.asu_data.net_worth + self.asu_total_cost_usd) / self.asu_total_cost_usd;
        let shanghai_roi = final_data.shanghai_data.net_worth / self.asu_total_cost_usd;
        let roi_difference = shanghai_roi - asu_roi;

        (asu_roi, shanghai_roi, roi_difference)
    }

    pub fn generate_chart(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let results = self.calculate_roi();

        let root = BitMapBackend::new(filename, (1200, 800)).into_drawing_area();
        root.fill(&WHITE)?;

        let max_net_worth = results
            .iter()
            .map(|d| d.asu_data.net_worth.max(d.shanghai_data.net_worth))
            .fold(0.0f64, |acc, x| acc.max(x));

        let min_net_worth = results
            .iter()
            .map(|d| d.asu_data.net_worth.min(d.shanghai_data.net_worth))
            .fold(0.0f64, |acc, x| acc.min(x));

        let mut chart = ChartBuilder::on(&root)
            .caption("ASU留学 vs 上海工作 净资产对比", ("Arial", 30))
            .margin(10)
            .x_label_area_size(50)
            .y_label_area_size(80)
            .build_cartesian_2d(
                1u32..self.total_years,
                (min_net_worth * 1.1)..(max_net_worth * 1.1),
            )?;

        chart
            .configure_mesh()
            .x_desc("年份")
            .y_desc("净资产 (美元)")
            .draw()?;

        // ASU路径
        chart
            .draw_series(LineSeries::new(
                results.iter().map(|d| (d.year, d.asu_data.net_worth)),
                &RED,
            ))?
            .label("ASU路径")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], RED));

        // 上海路径
        chart
            .draw_series(LineSeries::new(
                results.iter().map(|d| (d.year, d.shanghai_data.net_worth)),
                &BLUE,
            ))?
            .label("上海路径")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], BLUE));

        chart.configure_series_labels().draw()?;
        root.present()?;

        println!("图表已保存到: {}", filename);
        Ok(())
    }

    pub fn print_detailed_analysis(&self) {
        let results = self.calculate_roi();
        let (asu_roi, shanghai_roi, roi_diff) = self.calculate_final_roi();

        println!("=== ASU vs 上海工作 ROI 详细分析 ===\n");

        self.print_parameters();
        self.print_yearly_data(&results);
        self.print_roi_summary(asu_roi, shanghai_roi, roi_diff, &results);
    }

    fn print_parameters(&self) {
        println!("=== 计算参数 ===");

        #[derive(Tabled)]
        struct Parameter {
            #[tabled(rename = "参数")]
            name:        String,
            #[tabled(rename = "数值")]
            value:       String,
            #[tabled(rename = "说明")]
            description: String,
        }

        let parameters = vec![
            Parameter {
                name:        "ASU学费总计".to_string(),
                value:       format!("${:.0}", self.asu_total_cost_usd),
                description: "2年总学费及相关费用".to_string(),
            },
            Parameter {
                name:        "ASU初始薪资".to_string(),
                value:       format!("${:.0}/年", self.asu_initial_salary_usd),
                description: "毕业后起薪".to_string(),
            },
            Parameter {
                name:        "ASU薪资增长率".to_string(),
                value:       format!("{:.1}%/年", self.asu_salary_growth_rate * 100.0),
                description: "年度薪资涨幅".to_string(),
            },
            Parameter {
                name:        "上海初始薪资".to_string(),
                value:       format!("${:.0}/年", self.shanghai_initial_salary_usd),
                description: "在上海工作起薪".to_string(),
            },
            Parameter {
                name:        "上海薪资增长率".to_string(),
                value:       format!("{:.1}%/年", self.shanghai_salary_growth * 100.0),
                description: "年度薪资涨幅".to_string(),
            },
            Parameter {
                name:        "投资回报率".to_string(),
                value:       format!("{:.1}%/年", self.investment_return_rate * 100.0),
                description: "S&P500年化收益(按月定投)".to_string(),
            },
            Parameter {
                name:        "投资比例".to_string(),
                value:       format!("{:.1}%", self.investment_portion * 100.0),
                description: "可支配收入投资比例".to_string(),
            },
        ];

        let params_table = Table::new(parameters);
        println!("{}", params_table);
        println!();
    }

    fn print_yearly_data(&self, results: &[ComparisonData]) {
        println!("=== 年度详细数据 ===");

        #[derive(Tabled)]
        struct YearlyTableRow {
            #[tabled(rename = "年份")]
            year:                   u32,
            #[tabled(rename = "ASU工作年")]
            asu_work_year:          String,
            #[tabled(rename = "ASU月薪")]
            asu_monthly_salary:     String,
            #[tabled(rename = "ASU税后月薪")]
            asu_net_monthly_salary: String,
            #[tabled(rename = "ASU总收入")]
            asu_income:             String,
            #[tabled(rename = "ASU税后")]
            asu_net_income:         String,
            #[tabled(rename = "ASU可支配")]
            asu_disposable:         String,
            #[tabled(rename = "ASU当年投资")]
            asu_investment:         String,
            #[tabled(rename = "ASU累计收益")]
            asu_cumulative_return:  String,
            #[tabled(rename = "ASU净资产")]
            asu_net_worth:          String,
            #[tabled(rename = "上海工作年")]
            sh_work_year:           String,
            #[tabled(rename = "上海月薪")]
            sh_monthly_salary:      String,
            #[tabled(rename = "上海税后月薪")]
            sh_net_monthly_salary:  String,
            #[tabled(rename = "上海总收入")]
            sh_income:              String,
            #[tabled(rename = "上海税后")]
            sh_net_income:          String,
            #[tabled(rename = "上海可支配")]
            sh_disposable:          String,
            #[tabled(rename = "上海当年投资")]
            sh_investment:          String,
            #[tabled(rename = "上海累计收益")]
            sh_cumulative_return:   String,
            #[tabled(rename = "上海净资产")]
            sh_net_worth:           String,
        }

        let table_data: Vec<YearlyTableRow> = results
            .iter()
            .map(|data| {
                // 计算累计收益 = 总投资价值 - 累计投资本金
                let asu_cumulative_return =
                    data.asu_data.total_investment - data.asu_data.total_investment_principal;
                let sh_cumulative_return = data.shanghai_data.total_investment
                    - data.shanghai_data.total_investment_principal;

                YearlyTableRow {
                    year:                   data.year,
                    asu_work_year:          data
                        .asu_data
                        .work_year
                        .map_or("学习中".to_string(), |y| format!("第{}年", y)),
                    asu_monthly_salary:     if data.asu_data.income_usd > 0.0 {
                        format_currency(data.asu_data.income_usd / 12.0)
                    } else {
                        "-".to_string()
                    },
                    asu_net_monthly_salary: if data.asu_data.net_income_usd > 0.0 {
                        format_currency(data.asu_data.net_income_usd / 12.0)
                    } else {
                        "-".to_string()
                    },
                    asu_income:             if data.asu_data.income_usd > 0.0 {
                        format_currency(data.asu_data.income_usd)
                    } else {
                        "-".to_string()
                    },
                    asu_net_income:         if data.asu_data.net_income_usd > 0.0 {
                        format_currency(data.asu_data.net_income_usd)
                    } else {
                        "-".to_string()
                    },
                    asu_disposable:         if data.asu_data.disposable_income_usd > 0.0 {
                        format_currency(data.asu_data.disposable_income_usd)
                    } else {
                        "-".to_string()
                    },
                    asu_investment:         if data.asu_data.investment_amount > 0.0 {
                        format_currency(data.asu_data.investment_amount)
                    } else {
                        "-".to_string()
                    },
                    asu_cumulative_return:  if asu_cumulative_return > 0.0 {
                        format_currency(asu_cumulative_return)
                    } else {
                        "$0".to_string()
                    },
                    asu_net_worth:          format_currency(data.asu_data.net_worth),
                    sh_work_year:           data
                        .shanghai_data
                        .work_year
                        .map_or("退休".to_string(), |y| format!("第{}年", y)),
                    sh_monthly_salary:      if data.shanghai_data.income_usd > 0.0 {
                        format_currency(data.shanghai_data.income_usd / 12.0)
                    } else {
                        "-".to_string()
                    },
                    sh_net_monthly_salary:  if data.shanghai_data.net_income_usd > 0.0 {
                        format_currency(data.shanghai_data.net_income_usd / 12.0)
                    } else {
                        "-".to_string()
                    },
                    sh_income:              if data.shanghai_data.income_usd > 0.0 {
                        format_currency(data.shanghai_data.income_usd)
                    } else {
                        "-".to_string()
                    },
                    sh_net_income:          if data.shanghai_data.net_income_usd > 0.0 {
                        format_currency(data.shanghai_data.net_income_usd)
                    } else {
                        "-".to_string()
                    },
                    sh_disposable:          if data.shanghai_data.disposable_income_usd > 0.0 {
                        format_currency(data.shanghai_data.disposable_income_usd)
                    } else {
                        "-".to_string()
                    },
                    sh_investment:          if data.shanghai_data.investment_amount > 0.0 {
                        format_currency(data.shanghai_data.investment_amount)
                    } else {
                        "-".to_string()
                    },
                    sh_cumulative_return:   format_currency(sh_cumulative_return),
                    sh_net_worth:           format_currency(data.shanghai_data.net_worth),
                }
            })
            .collect();

        let mut table = Table::new(table_data);
        table
            .with(Style::modern())
            .with(Modify::new(Columns::new(0..=19)).with(Alignment::center()));

        println!("{}", table);
        println!();
    }

    fn print_roi_summary(
        &self,
        asu_roi: f64,
        shanghai_roi: f64,
        roi_diff: f64,
        results: &[ComparisonData],
    ) {
        println!("\n=== 最终ROI分析 ===");

        #[derive(Tabled)]
        struct ROISummary {
            #[tabled(rename = "路径")]
            path:      String,
            #[tabled(rename = "最终ROI")]
            roi:       String,
            #[tabled(rename = "净资产")]
            net_worth: String,
        }

        let final_data = results.last().unwrap();
        let summary_data = vec![
            ROISummary {
                path:      "ASU留学路径".to_string(),
                roi:       format!("{:.2}%", (asu_roi - 1.0) * 100.0),
                net_worth: format_currency(final_data.asu_data.net_worth),
            },
            ROISummary {
                path:      "上海工作路径".to_string(),
                roi:       format!("{:.2}%", (shanghai_roi - 1.0) * 100.0),
                net_worth: format_currency(final_data.shanghai_data.net_worth),
            },
            ROISummary {
                path:      "差异".to_string(),
                roi:       format!("{:.2}%", roi_diff * 100.0),
                net_worth: format_currency(
                    final_data.shanghai_data.net_worth - final_data.asu_data.net_worth,
                ),
            },
        ];

        let summary_table = Table::new(summary_data);
        println!("{}", summary_table);

        // 盈亏平衡点分析
        let breakeven_year = results
            .iter()
            .find(|d| d.asu_data.net_worth >= d.shanghai_data.net_worth);
        if let Some(data) = breakeven_year {
            println!("\n💡 ASU路径在第{}年追平上海路径", data.year);
        } else {
            println!(
                "\n⚠️  在{}年分析期内，ASU路径未能追平上海路径",
                self.total_years
            );
        }

        // 结论
        if final_data.shanghai_data.net_worth > final_data.asu_data.net_worth {
            println!("\n📊 结论: 在当前假设下，上海路径的财务回报更优");
        } else {
            println!("\n📊 结论: 在当前假设下，ASU路径的财务回报更优");
        }
    }

    pub fn sensitivity_analysis(&self) {
        println!("\n=== 敏感性分析 ===");

        #[derive(Tabled)]
        struct SensitivityRow {
            #[tabled(rename = "场景")]
            scenario:     String,
            #[tabled(rename = "ASU ROI")]
            asu_roi:      String,
            #[tabled(rename = "上海 ROI")]
            shanghai_roi: String,
            #[tabled(rename = "差异")]
            difference:   String,
        }

        let mut sensitivity_data = Vec::new();

        // 基准情况
        let (base_asu_roi, base_sh_roi, base_diff) = self.calculate_final_roi();
        sensitivity_data.push(SensitivityRow {
            scenario:     "基准情况".to_string(),
            asu_roi:      format!("{:.1}%", (base_asu_roi - 1.0) * 100.0),
            shanghai_roi: format!("{:.1}%", (base_sh_roi - 1.0) * 100.0),
            difference:   format!("{:.1}%", base_diff * 100.0),
        });

        // 不同的上海薪资增长率
        let growth_rates = [0.0, 0.02, 0.04, 0.06];
        for &rate in &growth_rates {
            let mut calc = self.clone();
            calc.shanghai_salary_growth = rate;
            let (asu_roi, sh_roi, diff) = calc.calculate_final_roi();
            sensitivity_data.push(SensitivityRow {
                scenario:     format!("上海薪资增长{}%/年", rate * 100.0),
                asu_roi:      format!("{:.1}%", (asu_roi - 1.0) * 100.0),
                shanghai_roi: format!("{:.1}%", (sh_roi - 1.0) * 100.0),
                difference:   format!("{:.1}%", diff * 100.0),
            });
        }

        // 不同的投资回报率
        let return_rates = [0.07, 0.08, 0.10, 0.12];
        for &rate in &return_rates {
            let mut calc = self.clone();
            calc.investment_return_rate = rate;
            let (asu_roi, sh_roi, diff) = calc.calculate_final_roi();
            sensitivity_data.push(SensitivityRow {
                scenario:     format!("投资回报率{}%/年", rate * 100.0),
                asu_roi:      format!("{:.1}%", (asu_roi - 1.0) * 100.0),
                shanghai_roi: format!("{:.1}%", (sh_roi - 1.0) * 100.0),
                difference:   format!("{:.1}%", diff * 100.0),
            });
        }

        let sensitivity_table = Table::new(sensitivity_data);
        println!("{}", sensitivity_table);
    }

    /// Profile比较分析
    pub fn analyze_profile_comparison(&self, profile1: &Profile, profile2: &Profile) {
        let results = self.compare_profiles(profile1, profile2);
        let (roi1, roi2, roi_diff) = self.calculate_profile_final_roi(profile1, profile2);

        println!(
            "=== {} vs {} ROI 详细分析 ===\n",
            profile1.name, profile2.name
        );

        self.print_profile_parameters(profile1, profile2);
        self.print_profile_yearly_data(&results);
        self.print_profile_roi_summary(roi1, roi2, roi_diff, &results, profile1, profile2);
    }

    /// 计算Profile的最终ROI
    pub fn calculate_profile_final_roi(
        &self,
        profile1: &Profile,
        profile2: &Profile,
    ) -> (f64, f64, f64) {
        let results = self.compare_profiles(profile1, profile2);
        let final_data = results.last().unwrap();

        // 计算ROI时需要考虑不同的成本基准
        let profile1_cost_basis = profile1.total_cost_usd.unwrap_or(1.0);
        let profile2_cost_basis = profile2.total_cost_usd.unwrap_or(profile1_cost_basis);

        let roi1 = if profile1.total_cost_usd.is_some() {
            (final_data.profile1_data.net_worth + profile1_cost_basis) / profile1_cost_basis
        } else {
            final_data.profile1_data.net_worth / profile1_cost_basis
        };

        let roi2 = if profile2.total_cost_usd.is_some() {
            (final_data.profile2_data.net_worth + profile2_cost_basis) / profile2_cost_basis
        } else {
            final_data.profile2_data.net_worth / profile2_cost_basis
        };

        let roi_difference = roi2 - roi1;
        (roi1, roi2, roi_difference)
    }

    /// 打印Profile参数对比
    fn print_profile_parameters(&self, profile1: &Profile, profile2: &Profile) {
        println!("=== Profile参数对比 ===");

        #[derive(Tabled)]
        struct ProfileParameter {
            #[tabled(rename = "参数")]
            name:   String,
            #[tabled(rename = "Profile 1")]
            value1: String,
            #[tabled(rename = "Profile 2")]
            value2: String,
        }

        let parameters = vec![
            ProfileParameter {
                name:   "Profile名称".to_string(),
                value1: profile1.name.clone(),
                value2: profile2.name.clone(),
            },
            ProfileParameter {
                name:   "类型".to_string(),
                value1: match profile1.profile_type {
                    ProfileType::Education => "教育路径".to_string(),
                    ProfileType::Work => "工作路径".to_string(),
                },
                value2: match profile2.profile_type {
                    ProfileType::Education => "教育路径".to_string(),
                    ProfileType::Work => "工作路径".to_string(),
                },
            },
            ProfileParameter {
                name:   "地点".to_string(),
                value1: if let Some(city) = &profile1.location.city {
                    format!("{}, {}", city, profile1.location.country)
                } else {
                    profile1.location.country.clone()
                },
                value2: if let Some(city) = &profile2.location.city {
                    format!("{}, {}", city, profile2.location.country)
                } else {
                    profile2.location.country.clone()
                },
            },
            ProfileParameter {
                name:   "初始薪资".to_string(),
                value1: format!("${:.0}/年", profile1.initial_salary_usd),
                value2: format!("${:.0}/年", profile2.initial_salary_usd),
            },
            ProfileParameter {
                name:   "薪资增长率".to_string(),
                value1: format!("{:.1}%/年", profile1.salary_growth_rate * 100.0),
                value2: format!("{:.1}%/年", profile2.salary_growth_rate * 100.0),
            },
            ProfileParameter {
                name:   "生活成本".to_string(),
                value1: format!("${:.0}/年", profile1.living_cost_usd),
                value2: format!("${:.0}/年", profile2.living_cost_usd),
            },
            ProfileParameter {
                name:   "税率".to_string(),
                value1: format!("{:.1}%", profile1.tax_rate * 100.0),
                value2: format!("{:.1}%", profile2.tax_rate * 100.0),
            },
        ];

        let params_table = Table::new(parameters);
        println!("{}", params_table);
        println!();
    }

    /// 打印Profile年度数据对比
    fn print_profile_yearly_data(&self, results: &[ProfileComparisonData]) {
        println!("=== 年度详细数据对比 ===");

        #[derive(Tabled)]
        struct ProfileYearlyTableRow {
            #[tabled(rename = "年份")]
            year:               u32,
            #[tabled(rename = "Profile1净资产")]
            profile1_net_worth: String,
            #[tabled(rename = "Profile2净资产")]
            profile2_net_worth: String,
            #[tabled(rename = "差异")]
            difference:         String,
        }

        let table_data: Vec<ProfileYearlyTableRow> = results
            .iter()
            .map(|data| {
                let difference = data.profile2_data.net_worth - data.profile1_data.net_worth;
                ProfileYearlyTableRow {
                    year:               data.year,
                    profile1_net_worth: format_currency(data.profile1_data.net_worth),
                    profile2_net_worth: format_currency(data.profile2_data.net_worth),
                    difference:         format_currency(difference),
                }
            })
            .collect();

        let mut table = Table::new(table_data);
        table.with(Style::modern());
        println!("{}", table);
        println!();
    }

    /// 打印Profile ROI总结
    fn print_profile_roi_summary(
        &self,
        roi1: f64,
        roi2: f64,
        roi_diff: f64,
        results: &[ProfileComparisonData],
        profile1: &Profile,
        profile2: &Profile,
    ) {
        println!("\n=== 最终ROI分析 ===");

        #[derive(Tabled)]
        struct ProfileROISummary {
            #[tabled(rename = "Profile")]
            profile:   String,
            #[tabled(rename = "最终ROI")]
            roi:       String,
            #[tabled(rename = "净资产")]
            net_worth: String,
        }

        let final_data = results.last().unwrap();
        let summary_data = vec![
            ProfileROISummary {
                profile:   profile1.name.clone(),
                roi:       format!("{:.2}%", (roi1 - 1.0) * 100.0),
                net_worth: format_currency(final_data.profile1_data.net_worth),
            },
            ProfileROISummary {
                profile:   profile2.name.clone(),
                roi:       format!("{:.2}%", (roi2 - 1.0) * 100.0),
                net_worth: format_currency(final_data.profile2_data.net_worth),
            },
            ProfileROISummary {
                profile:   "差异".to_string(),
                roi:       format!("{:.2}%", roi_diff * 100.0),
                net_worth: format_currency(
                    final_data.profile2_data.net_worth - final_data.profile1_data.net_worth,
                ),
            },
        ];

        let summary_table = Table::new(summary_data);
        println!("{}", summary_table);

        // 盈亏平衡点分析
        let breakeven_year = results
            .iter()
            .find(|d| d.profile1_data.net_worth >= d.profile2_data.net_worth);
        if let Some(data) = breakeven_year {
            println!(
                "\n💡 {}在第{}年追平{}",
                profile1.name, data.year, profile2.name
            );
        } else {
            println!(
                "\n⚠️ 在{}年分析期内，{}未能追平{}",
                self.total_years, profile1.name, profile2.name
            );
        }

        // 结论
        if final_data.profile2_data.net_worth > final_data.profile1_data.net_worth {
            println!("\n📊 结论: 在当前假设下，{}的财务回报更优", profile2.name);
        } else {
            println!("\n📊 结论: 在当前假设下，{}的财务回报更优", profile1.name);
        }
    }

    /// 生成Profile比较图表
    pub fn generate_profile_comparison_chart(
        &self,
        profile1: &Profile,
        profile2: &Profile,
        filename: &str,
    ) -> Result<()> {
        let results = self.compare_profiles(profile1, profile2);

        let root = BitMapBackend::new(filename, (1200, 800)).into_drawing_area();
        root.fill(&WHITE)?;

        let max_net_worth = results
            .iter()
            .map(|d| d.profile1_data.net_worth.max(d.profile2_data.net_worth))
            .fold(0.0f64, |acc, x| acc.max(x));

        let min_net_worth = results
            .iter()
            .map(|d| d.profile1_data.net_worth.min(d.profile2_data.net_worth))
            .fold(0.0f64, |acc, x| acc.min(x));

        let mut chart = ChartBuilder::on(&root)
            .caption(
                &format!("{} vs {} 净资产对比", profile1.name, profile2.name),
                ("Arial", 30),
            )
            .margin(10)
            .x_label_area_size(50)
            .y_label_area_size(80)
            .build_cartesian_2d(
                1u32..self.total_years,
                (min_net_worth * 1.1)..(max_net_worth * 1.1),
            )?;

        chart
            .configure_mesh()
            .x_desc("年份")
            .y_desc("净资产 (美元)")
            .draw()?;

        // Profile 1路径
        chart
            .draw_series(LineSeries::new(
                results.iter().map(|d| (d.year, d.profile1_data.net_worth)),
                &RED,
            ))?
            .label(&profile1.name)
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], RED));

        // Profile 2路径
        chart
            .draw_series(LineSeries::new(
                results.iter().map(|d| (d.year, d.profile2_data.net_worth)),
                &BLUE,
            ))?
            .label(&profile2.name)
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], BLUE));

        chart.configure_series_labels().draw()?;
        root.present()?;

        println!("图表已保存到: {}", filename);
        Ok(())
    }
}

fn main() -> Result<()> {
    let calculator = ROICalculator::default();

    // 初始化数据库
    let db = DatabaseManager::new().map_err(|e| anyhow::Error::new(e))?;
    let mut profile_creator = ProfileCreator::new(db);

    println!("🎯 ROI Calculator - Profile版");
    println!("============================");

    loop {
        let actions = vec![
            "创建新Profile",
            "查看已有Profile",
            "比较Profile",
            "运行原始ASU vs 上海比较",
            "退出",
        ];

        let action = Select::new()
            .with_prompt("选择操作")
            .items(&actions)
            .interact()?;

        match action {
            0 => {
                // 创建新Profile
                profile_creator.create_profile()?;
            }
            1 => {
                // 查看已有Profile
                profile_creator.list_profiles()?;
            }
            2 => {
                // 比较Profile
                let profiles = profile_creator.select_profiles_for_comparison()?;
                if profiles.len() == 2 {
                    let profile1 = &profiles[0];
                    let profile2 = &profiles[1];

                    println!("\n🔄 开始分析比较...");

                    // 进行Profile比较分析
                    calculator.analyze_profile_comparison(profile1, profile2);

                    // 生成比较图表
                    let chart_filename = format!(
                        "{}_vs_{}_comparison.png",
                        profile1.name.replace(" ", "_"),
                        profile2.name.replace(" ", "_")
                    );
                    calculator.generate_profile_comparison_chart(
                        profile1,
                        profile2,
                        &chart_filename,
                    )?;

                    println!("\n✅ 比较分析完成！");
                }
            }
            3 => {
                // 运行原始比较
                println!("\n🔄 运行原始ASU vs 上海比较分析...");
                calculator.print_detailed_analysis();
                calculator
                    .generate_chart("roi_comparison.png")
                    .map_err(|e| anyhow::Error::msg(e.to_string()))?;
                calculator.sensitivity_analysis();
            }
            4 => {
                // 退出
                println!("👋 再见！");
                break;
            }
            _ => {}
        }

        println!("\n{}\n", "=".repeat(50));
    }

    Ok(())
}
