use std::fmt;

use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::{prelude::*, sqlite::SqliteConnection};
use serde::{Deserialize, Serialize};
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

/// Profile类型枚举
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProfileType {
    Education,
    Work,
}

impl fmt::Display for ProfileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProfileType::Education => write!(f, "Education"),
            ProfileType::Work => write!(f, "Work"),
        }
    }
}

impl std::str::FromStr for ProfileType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "Education" => Ok(ProfileType::Education),
            "Work" => Ok(ProfileType::Work),
            _ => Err(anyhow::anyhow!("Invalid profile type: '{}'", s)),
        }
    }
}

/// 地理位置信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Location {
    pub country:  String,
    pub city:     Option<String>,
    pub currency: String,
}

/// 工作参数
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkParams {
    pub start_delay:    u32,
    pub duration_limit: Option<u32>,
}

/// 财务参数
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinancialParams {
    pub initial_salary_usd: f64,
    pub salary_growth_rate: f64,
    pub living_cost_usd:    f64,
    pub living_cost_growth: f64,
    pub tax_rate:           f64,
}

/// 成本参数
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostParams {
    pub total_cost_usd: f64,
    pub cost_duration:  u32,
}

/// **核心 Profile 结构** - 统一的数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: Uuid,
    pub name: String,
    pub profile_type: ProfileType,
    pub location: Location,
    pub work_params: WorkParams,
    pub financial_params: FinancialParams,
    pub cost_params: Option<CostParams>,
    pub first_year_opportunity_cost: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub description: Option<String>,
}

/// 数据库适配器 - 只负责类型转换
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = profiles)]
pub struct ProfileDbRecord {
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

impl Profile {
    /// 创建新的Profile
    pub fn new(
        name: String,
        profile_type: ProfileType,
        location: Location,
        work_params: WorkParams,
        financial_params: FinancialParams,
    ) -> Self {
        let now = Utc::now();
        Profile {
            id: Uuid::new_v4(),
            name,
            profile_type,
            location,
            work_params,
            financial_params,
            cost_params: None,
            first_year_opportunity_cost: None,
            created_at: now,
            updated_at: now,
            description: None,
        }
    }

    /// 流式构造器方法
    pub fn with_cost_params(mut self, cost_params: CostParams) -> Self {
        self.cost_params = Some(cost_params);
        self
    }

    pub fn with_opportunity_cost(mut self, cost: f64) -> Self {
        self.first_year_opportunity_cost = Some(cost);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// 更新时间戳
    pub fn touch(&mut self) { self.updated_at = Utc::now(); }
}

/// 实现 Profile 和数据库记录之间的转换
impl TryFrom<ProfileDbRecord> for Profile {
    type Error = anyhow::Error;

    fn try_from(db: ProfileDbRecord) -> Result<Self> {
        Ok(Profile {
            id: Uuid::parse_str(&db.id)?,
            name: db.name,
            profile_type: db.profile_type.parse()?,
            location: Location {
                country:  db.location_country,
                city:     db.location_city,
                currency: db.location_currency,
            },
            work_params: WorkParams {
                start_delay:    db.work_start_delay as u32,
                duration_limit: db.work_duration_limit.map(|x| x as u32),
            },
            financial_params: FinancialParams {
                initial_salary_usd: db.initial_salary_usd,
                salary_growth_rate: db.salary_growth_rate,
                living_cost_usd:    db.living_cost_usd,
                living_cost_growth: db.living_cost_growth,
                tax_rate:           db.tax_rate,
            },
            cost_params: match (db.total_cost_usd, db.cost_duration) {
                (Some(total), Some(duration)) if duration > 0 => Some(CostParams {
                    total_cost_usd: total,
                    cost_duration:  duration as u32,
                }),
                _ => None,
            },
            first_year_opportunity_cost: db.first_year_opportunity_cost,
            created_at: DateTime::from_naive_utc_and_offset(db.created_at, Utc),
            updated_at: DateTime::from_naive_utc_and_offset(db.updated_at, Utc),
            description: db.description,
        })
    }
}

impl From<Profile> for ProfileDbRecord {
    fn from(profile: Profile) -> Self {
        let (total_cost_usd, cost_duration) = match profile.cost_params {
            Some(cost) => (Some(cost.total_cost_usd), Some(cost.cost_duration as i32)),
            None => (None, None),
        };

        ProfileDbRecord {
            id: profile.id.to_string(),
            name: profile.name,
            profile_type: profile.profile_type.to_string(),
            location_country: profile.location.country,
            location_city: profile.location.city,
            location_currency: profile.location.currency,
            work_start_delay: profile.work_params.start_delay as i32,
            work_duration_limit: profile.work_params.duration_limit.map(|x| x as i32),
            initial_salary_usd: profile.financial_params.initial_salary_usd,
            salary_growth_rate: profile.financial_params.salary_growth_rate,
            living_cost_usd: profile.financial_params.living_cost_usd,
            living_cost_growth: profile.financial_params.living_cost_growth,
            tax_rate: profile.financial_params.tax_rate,
            total_cost_usd,
            cost_duration,
            first_year_opportunity_cost: profile.first_year_opportunity_cost,
            created_at: profile.created_at.naive_utc(),
            updated_at: profile.updated_at.naive_utc(),
            description: profile.description,
        }
    }
}

/// 计算参数 - 使用借用而不是拥有
pub struct CalculationContext<'a> {
    pub work_params:                 &'a WorkParams,
    pub financial_params:            &'a FinancialParams,
    pub cost_params:                 Option<&'a CostParams>,
    pub first_year_opportunity_cost: Option<f64>,
}

impl Profile {
    /// 获取计算上下文（避免数据复制）
    pub fn calculation_context(&self) -> CalculationContext {
        CalculationContext {
            work_params:                 &self.work_params,
            financial_params:            &self.financial_params,
            cost_params:                 self.cost_params.as_ref(),
            first_year_opportunity_cost: self.first_year_opportunity_cost,
        }
    }
}

/// 数据库操作trait
pub trait ProfileRepository {
    fn save(&self, profile: &Profile) -> Result<()>;
    fn find_by_id(&self, id: Uuid) -> Result<Option<Profile>>;
    fn find_all(&self) -> Result<Vec<Profile>>;
    fn delete(&self, id: Uuid) -> Result<()>;
}

/// Diesel 实现
pub struct DieselProfileRepository<'a> {
    conn: &'a mut SqliteConnection,
}

impl<'a> DieselProfileRepository<'a> {
    pub fn new(conn: &'a mut SqliteConnection) -> Self { Self { conn } }
}

impl<'a> ProfileRepository for DieselProfileRepository<'a> {
    fn save(&self, profile: &Profile) -> Result<()> {
        use crate::profiles::dsl::*;

        let db_record = ProfileDbRecord::from(profile.clone());
        diesel::insert_into(profiles)
            .values(&db_record)
            .on_conflict(id)
            .do_update()
            .set(&db_record)
            .execute(self.conn)?;
        Ok(())
    }

    fn find_by_id(&self, profile_id: Uuid) -> Result<Option<Profile>> {
        use crate::profiles::dsl::*;

        let record: Option<ProfileDbRecord> = profiles
            .filter(id.eq(profile_id.to_string()))
            .first(self.conn)
            .optional()?;

        match record {
            Some(db_record) => Ok(Some(Profile::try_from(db_record)?)),
            None => Ok(None),
        }
    }

    fn find_all(&self) -> Result<Vec<Profile>> {
        use crate::profiles::dsl::*;

        let records: Vec<ProfileDbRecord> = profiles.load(self.conn)?;
        records.into_iter().map(Profile::try_from).collect()
    }

    fn delete(&self, profile_id: Uuid) -> Result<()> {
        use crate::profiles::dsl::*;

        diesel::delete(profiles.filter(id.eq(profile_id.to_string()))).execute(self.conn)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_conversion() {
        let profile = Profile::new(
            "Test Profile".to_string(),
            ProfileType::Work,
            Location {
                country:  "USA".to_string(),
                city:     Some("NYC".to_string()),
                currency: "USD".to_string(),
            },
            WorkParams {
                start_delay:    0,
                duration_limit: Some(40),
            },
            FinancialParams {
                initial_salary_usd: 75000.0,
                salary_growth_rate: 0.05,
                living_cost_usd:    40000.0,
                living_cost_growth: 0.03,
                tax_rate:           0.22,
            },
        );

        // 转换为数据库记录
        let db_record = ProfileDbRecord::from(profile.clone());

        // 再转换回来
        let restored_profile = Profile::try_from(db_record).unwrap();

        assert_eq!(profile.id, restored_profile.id);
        assert_eq!(profile.name, restored_profile.name);
    }
}
