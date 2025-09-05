use std::fmt;

use anyhow::Result;
use chrono::Utc;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use uuid::Uuid;

use crate::{
    db::DatabaseManager,
    models::{CostParams, FinancialParams, Location, Profile, ProfileType, WorkParams},
};

/// 状态机的状态定义
#[derive(Debug, Clone, PartialEq)]
pub enum CreationState {
    Start,
    BasicInfo,
    LocationInfo,
    WorkParams,
    FinancialParams,
    CostParams,
    OpportunityParams,
    Summary,
    Confirmation,
    Complete,
    Cancelled,
}

/// 用户输入事件
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum UserAction {
    Continue = 0,
    Back = 1,
    Cancel = 2,
    Confirm = 3,
    Retry = 4,
}

/// Profile构建过程中的数据容器
#[derive(Debug, Clone, Default)]
pub struct ProfileBuilder {
    // 基本信息
    pub name:         Option<String>,
    pub profile_type: Option<ProfileType>,

    // 地理位置
    pub country:  Option<String>,
    pub city:     Option<String>,
    pub currency: Option<String>,

    // 工作参数
    pub work_start_delay:    Option<u32>,
    pub work_duration_limit: Option<Option<u32>>,

    // 财务参数
    pub initial_salary_usd: Option<f64>,
    pub salary_growth_rate: Option<f64>,
    pub living_cost_usd:    Option<f64>,
    pub living_cost_growth: Option<f64>,
    pub tax_rate:           Option<f64>,

    // 成本参数
    pub cost_params: Option<Option<CostParams>>,

    // 机会成本
    pub first_year_opportunity_cost: Option<Option<f64>>,

    // 描述
    pub description: Option<String>,
}

impl ProfileBuilder {
    pub fn new() -> Self { Self::default() }

    /// 验证是否可以构建完整的Profile
    pub fn is_complete(&self) -> bool {
        self.name.is_some()
            && self.profile_type.is_some()
            && self.country.is_some()
            && self.currency.is_some()
            && self.work_start_delay.is_some()
            && self.work_duration_limit.is_some()
            && self.initial_salary_usd.is_some()
            && self.salary_growth_rate.is_some()
            && self.living_cost_usd.is_some()
            && self.living_cost_growth.is_some()
            && self.tax_rate.is_some()
            && self.cost_params.is_some()
            && self.first_year_opportunity_cost.is_some()
    }

    /// 构建最终的Profile
    pub fn build(self) -> Result<Profile> {
        if !self.is_complete() {
            return Err(anyhow::anyhow!("Profile信息不完整"));
        }

        let now = Utc::now();

        Ok(Profile {
            id: Uuid::new_v4(),
            name: self.name.unwrap(),
            profile_type: self.profile_type.unwrap(),
            location: Location {
                country:  self.country.unwrap(),
                city:     self.city,
                currency: self.currency.unwrap(),
            },
            work_params: WorkParams {
                start_delay:    self.work_start_delay.unwrap(),
                duration_limit: self.work_duration_limit.unwrap(),
            },
            financial_params: FinancialParams {
                initial_salary_usd: self.initial_salary_usd.unwrap(),
                salary_growth_rate: self.salary_growth_rate.unwrap(),
                living_cost_usd:    self.living_cost_usd.unwrap(),
                living_cost_growth: self.living_cost_growth.unwrap(),
                tax_rate:           self.tax_rate.unwrap(),
            },
            cost_params: self.cost_params.unwrap(),
            first_year_opportunity_cost: self.first_year_opportunity_cost.unwrap(),
            created_at: now,
            updated_at: now,
            description: self.description,
        })
    }
}

/// 状态机实现
pub struct ProfileCreationStateMachine {
    current_state: CreationState,
    builder:       ProfileBuilder,
    db:            DatabaseManager,
    theme:         ColorfulTheme,
}

impl ProfileCreationStateMachine {
    pub fn new(db: DatabaseManager) -> Self {
        Self {
            current_state: CreationState::Start,
            builder: ProfileBuilder::new(),
            db,
            theme: ColorfulTheme::default(),
        }
    }

    /// 主要的状态机循环
    pub fn run(&mut self) -> Result<Option<Profile>> {
        self.print_welcome();

        loop {
            match self.current_state {
                CreationState::Start => {
                    self.transition_to(CreationState::BasicInfo);
                }
                CreationState::BasicInfo => match self.handle_basic_info()? {
                    UserAction::Continue => self.transition_to(CreationState::LocationInfo),
                    UserAction::Cancel => self.transition_to(CreationState::Cancelled),
                    _ => continue,
                },
                CreationState::LocationInfo => match self.handle_location_info()? {
                    UserAction::Continue => self.transition_to(CreationState::WorkParams),
                    UserAction::Back => self.transition_to(CreationState::BasicInfo),
                    UserAction::Cancel => self.transition_to(CreationState::Cancelled),
                    _ => continue,
                },
                CreationState::WorkParams => match self.handle_work_params()? {
                    UserAction::Continue => self.transition_to(CreationState::FinancialParams),
                    UserAction::Back => self.transition_to(CreationState::LocationInfo),
                    UserAction::Cancel => self.transition_to(CreationState::Cancelled),
                    _ => continue,
                },
                CreationState::FinancialParams => match self.handle_financial_params()? {
                    UserAction::Continue => self.transition_to(CreationState::CostParams),
                    UserAction::Back => self.transition_to(CreationState::WorkParams),
                    UserAction::Cancel => self.transition_to(CreationState::Cancelled),
                    _ => continue,
                },
                CreationState::CostParams => match self.handle_cost_params()? {
                    UserAction::Continue => self.transition_to(CreationState::OpportunityParams),
                    UserAction::Back => self.transition_to(CreationState::FinancialParams),
                    UserAction::Cancel => self.transition_to(CreationState::Cancelled),
                    _ => continue,
                },
                CreationState::OpportunityParams => match self.handle_opportunity_params()? {
                    UserAction::Continue => self.transition_to(CreationState::Summary),
                    UserAction::Back => self.transition_to(CreationState::CostParams),
                    UserAction::Cancel => self.transition_to(CreationState::Cancelled),
                    _ => continue,
                },
                CreationState::Summary => {
                    self.display_summary();
                    match self.handle_summary_confirmation()? {
                        UserAction::Confirm => self.transition_to(CreationState::Complete),
                        UserAction::Back => self.transition_to(CreationState::OpportunityParams),
                        UserAction::Cancel => self.transition_to(CreationState::Cancelled),
                        _ => continue,
                    }
                }
                CreationState::Complete => {
                    let profile = self.builder.clone().build()?;
                    self.save_profile(&profile)?;
                    println!("\n✅ Profile '{}' 已成功创建!", profile.name);
                    return Ok(Some(profile));
                }
                CreationState::Cancelled => {
                    println!("\n❌ 已取消创建Profile");
                    return Ok(None);
                }
                _ => unreachable!(),
            }
        }
    }

    fn transition_to(&mut self, new_state: CreationState) {
        println!("\n{}", "=".repeat(50));
        self.current_state = new_state;
    }

    fn print_welcome(&self) {
        println!("\n🎯 职业发展路径Profile创建向导");
        println!("=====================================");
        println!("💡 提示: 任何时候输入 'q' 可以退出，'b' 可以返回上一步");
    }

    fn handle_basic_info(&mut self) -> Result<UserAction> {
        println!("\n📝 第1步: 基本信息");

        let name: String = Input::with_theme(&self.theme)
            .with_prompt("Profile名称")
            .with_initial_text("我的职业路径")
            .interact_text()?;

        if name.eq_ignore_ascii_case("q") {
            return Ok(UserAction::Cancel);
        }

        let profile_types = vec!["🎓 留学/教育路径", "💼 工作路径"];
        let profile_type_idx = Select::with_theme(&self.theme)
            .with_prompt("选择路径类型")
            .items(&profile_types)
            .default(0)
            .interact()?;

        let profile_type = match profile_type_idx {
            0 => ProfileType::Education,
            1 => ProfileType::Work,
            _ => ProfileType::Work,
        };

        self.builder.name = Some(name);
        self.builder.profile_type = Some(profile_type);

        Ok(UserAction::Continue)
    }

    fn handle_location_info(&mut self) -> Result<UserAction> {
        println!("\n🌍 第2步: 地理位置信息");

        let action = self.prompt_navigation()?;
        if action != UserAction::Continue {
            return Ok(action);
        }

        let country: String = Input::with_theme(&self.theme)
            .with_prompt("国家")
            .with_initial_text("United States")
            .interact_text()?;

        let city: String = Input::with_theme(&self.theme)
            .with_prompt("城市 (可选，直接回车跳过)")
            .allow_empty(true)
            .interact_text()?;

        let currency: String = Input::with_theme(&self.theme)
            .with_prompt("货币代码")
            .with_initial_text("USD")
            .interact_text()?;

        self.builder.country = Some(country);
        self.builder.city = if city.is_empty() { None } else { Some(city) };
        self.builder.currency = Some(currency);

        Ok(UserAction::Continue)
    }

    fn handle_work_params(&mut self) -> Result<UserAction> {
        println!("\n💼 第3步: 工作参数设置");

        let action = self.prompt_navigation()?;
        if action != UserAction::Continue {
            return Ok(action);
        }

        let work_start_delay: u32 = Input::with_theme(&self.theme)
            .with_prompt("开始工作前的延迟年数 (如留学年数)")
            .with_initial_text("0")
            .interact_text()?;

        let has_work_limit = Confirm::with_theme(&self.theme)
            .with_prompt("是否有工作年限限制?")
            .default(false)
            .interact()?;

        let work_duration_limit = if has_work_limit {
            Some(
                Input::with_theme(&self.theme)
                    .with_prompt("工作年限限制 (年)")
                    .with_initial_text("10")
                    .interact_text()?,
            )
        } else {
            None
        };

        self.builder.work_start_delay = Some(work_start_delay);
        self.builder.work_duration_limit = Some(work_duration_limit);

        Ok(UserAction::Continue)
    }

    fn handle_financial_params(&mut self) -> Result<UserAction> {
        println!("\n💰 第4步: 财务参数设置");

        let action = self.prompt_navigation()?;
        if action != UserAction::Continue {
            return Ok(action);
        }

        let initial_salary_usd: f64 = Input::with_theme(&self.theme)
            .with_prompt("初始年薪 (USD)")
            .with_initial_text("50000")
            .interact_text()?;

        let salary_growth_rate: f64 = Input::with_theme(&self.theme)
            .with_prompt("年薪增长率 (小数形式，如0.03表示3%)")
            .with_initial_text("0.03")
            .interact_text()?;

        let living_cost_usd: f64 = Input::with_theme(&self.theme)
            .with_prompt("初始年生活成本 (USD)")
            .with_initial_text("30000")
            .interact_text()?;

        let living_cost_growth: f64 = Input::with_theme(&self.theme)
            .with_prompt("生活成本年增长率 (小数形式)")
            .with_initial_text("0.025")
            .interact_text()?;

        let tax_rate: f64 = Input::with_theme(&self.theme)
            .with_prompt("税率 (小数形式，如0.25表示25%)")
            .with_initial_text("0.25")
            .interact_text()?;

        // 验证输入
        if salary_growth_rate < 0.0 || salary_growth_rate > 1.0 {
            println!("⚠️ 薪资增长率应该在0-1之间");
            return Ok(UserAction::Retry);
        }

        if tax_rate < 0.0 || tax_rate > 1.0 {
            println!("⚠️ 税率应该在0-1之间");
            return Ok(UserAction::Retry);
        }

        self.builder.initial_salary_usd = Some(initial_salary_usd);
        self.builder.salary_growth_rate = Some(salary_growth_rate);
        self.builder.living_cost_usd = Some(living_cost_usd);
        self.builder.living_cost_growth = Some(living_cost_growth);
        self.builder.tax_rate = Some(tax_rate);

        Ok(UserAction::Continue)
    }

    fn handle_cost_params(&mut self) -> Result<UserAction> {
        println!("\n💸 第5步: 成本参数设置");

        let action = self.prompt_navigation()?;
        if action != UserAction::Continue {
            return Ok(action);
        }

        let is_education = matches!(self.builder.profile_type, Some(ProfileType::Education));

        let has_costs = Confirm::with_theme(&self.theme)
            .with_prompt("是否有初期成本 (如学费、培训费)?")
            .default(is_education)
            .interact()?;

        let cost_params = if has_costs {
            let total_cost: f64 = Input::with_theme(&self.theme)
                .with_prompt("总成本 (USD)")
                .with_initial_text("100000")
                .interact_text()?;

            let duration: u32 = Input::with_theme(&self.theme)
                .with_prompt("成本分摊年数")
                .with_initial_text("2")
                .interact_text()?;

            if total_cost < 0.0 {
                println!("⚠️ 总成本不能为负数");
                return Ok(UserAction::Retry);
            }

            if duration == 0 {
                println!("⚠️ 成本分摊年数必须大于0");
                return Ok(UserAction::Retry);
            }

            Some(CostParams::new(total_cost, duration)?)
        } else {
            None
        };

        self.builder.cost_params = Some(cost_params);

        Ok(UserAction::Continue)
    }

    fn handle_opportunity_params(&mut self) -> Result<UserAction> {
        println!("\n🎯 第6步: 机会成本设置");

        let action = self.prompt_navigation()?;
        if action != UserAction::Continue {
            return Ok(action);
        }

        let is_work = matches!(self.builder.profile_type, Some(ProfileType::Work));

        let has_opportunity_cost = Confirm::with_theme(&self.theme)
            .with_prompt("第一年是否有机会成本投资?")
            .default(is_work)
            .interact()?;

        let opportunity_cost = if has_opportunity_cost {
            Some(
                Input::with_theme(&self.theme)
                    .with_prompt("第一年机会成本投资金额 (USD)")
                    .with_initial_text("100000")
                    .interact_text()?,
            )
        } else {
            None
        };

        // 可选描述
        let description: String = Input::with_theme(&self.theme)
            .with_prompt("描述 (可选)")
            .allow_empty(true)
            .interact_text()?;

        self.builder.first_year_opportunity_cost = Some(opportunity_cost);
        self.builder.description = if description.is_empty() {
            None
        } else {
            Some(description)
        };

        Ok(UserAction::Continue)
    }

    fn display_summary(&self) {
        println!("\n📋 第7步: 确认信息");
        println!("==================");

        if let Some(ref name) = self.builder.name {
            println!("📝 Profile名称: {}", name);
        }

        if let Some(ref profile_type) = self.builder.profile_type {
            let type_icon = match profile_type {
                ProfileType::Education => "🎓",
                ProfileType::Work => "💼",
            };
            println!("{} 路径类型: {:?}", type_icon, profile_type);
        }

        if let Some(ref country) = self.builder.country {
            let location = match &self.builder.city {
                Some(city) => format!("{}, {}", city, country),
                None => country.clone(),
            };
            println!(
                "🌍 位置: {} ({})",
                location,
                self.builder
                    .currency
                    .as_ref()
                    .unwrap_or(&"未知".to_string())
            );
        }

        if let Some(delay) = self.builder.work_start_delay {
            println!("⏱️ 工作延迟: {} 年", delay);
        }

        if let Some(ref limit) = self.builder.work_duration_limit {
            match limit {
                Some(years) => println!("📅 工作年限: {} 年", years),
                None => println!("📅 工作年限: 无限制"),
            }
        }

        if let Some(salary) = self.builder.initial_salary_usd {
            println!("💰 初始年薪: ${:,.0}", salary);
        }

        if let Some(ref cost_params) = self.builder.cost_params {
            match cost_params {
                Some(cost) => println!(
                    "💸 总成本: ${:,.0} (分摊{}年)",
                    cost.total_cost_usd, cost.cost_duration
                ),
                None => println!("💸 无初期成本"),
            }
        }

        if let Some(ref opp_cost) = self.builder.first_year_opportunity_cost {
            match opp_cost {
                Some(cost) => println!("🎯 机会成本: ${:,.0}", cost),
                None => println!("🎯 无机会成本"),
            }
        }
    }

    fn handle_summary_confirmation(&self) -> Result<UserAction> {
        let choices = vec!["✅ 确认创建", "⬅️ 返回修改", "❌ 取消"];

        let choice = Select::with_theme(&self.theme)
            .with_prompt("请选择操作")
            .items(&choices)
            .default(0)
            .interact()?;

        match choice {
            0 => Ok(UserAction::Confirm),
            1 => Ok(UserAction::Back),
            2 => Ok(UserAction::Cancel),
            _ => Ok(UserAction::Cancel),
        }
    }

    fn prompt_navigation(&self) -> Result<UserAction> {
        if self.current_state == CreationState::BasicInfo {
            return Ok(UserAction::Continue);
        }

        let choices = vec!["➡️ 继续", "⬅️ 返回上一步", "❌ 取消"];

        let choice = Select::with_theme(&self.theme)
            .with_prompt("选择操作")
            .items(&choices)
            .default(0)
            .interact()?;

        match choice {
            0 => Ok(UserAction::Continue),
            1 => Ok(UserAction::Back),
            2 => Ok(UserAction::Cancel),
            _ => Ok(UserAction::Continue),
        }
    }

    fn save_profile(&mut self, profile: &Profile) -> Result<()> {
        self.db
            .save_profile(profile)
            .map_err(|e| anyhow::anyhow!("保存Profile失败: {}", e))
    }
}

/// 交互式Profile管理器
pub struct ProfileManager {
    db:    DatabaseManager,
    theme: ColorfulTheme,
}

impl ProfileManager {
    pub fn new(db: DatabaseManager) -> Self {
        Self {
            db,
            theme: ColorfulTheme::default(),
        }
    }

    /// 主菜单
    pub fn run(&mut self) -> Result<()> {
        loop {
            self.display_main_menu()?;

            let choices = vec![
                "📝 创建新Profile",
                "📋 查看所有Profile",
                "🔍 查看Profile详情",
                "✏️ 编辑Profile",
                "🗑️ 删除Profile",
                "❌ 退出",
            ];

            let choice = Select::with_theme(&self.theme)
                .with_prompt("请选择操作")
                .items(&choices)
                .default(0)
                .interact()?;

            match choice {
                0 => self.create_profile()?,
                1 => self.list_profiles()?,
                2 => self.view_profile_details()?,
                3 => self.edit_profile()?,
                4 => self.delete_profile()?,
                5 => {
                    println!("\n👋 再见!");
                    break;
                }
                _ => continue,
            }
        }

        Ok(())
    }

    fn display_main_menu(&self) -> Result<()> {
        println!("\n{}", "=".repeat(60));
        println!("🎯 职业发展路径Profile管理系统");
        println!("{}", "=".repeat(60));

        // 显示统计信息
        let profiles = self
            .db
            .get_profiles()
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let education_count = profiles
            .iter()
            .filter(|p| matches!(p.profile_type, ProfileType::Education))
            .count();
        let work_count = profiles
            .iter()
            .filter(|p| matches!(p.profile_type, ProfileType::Work))
            .count();

        println!(
            "📊 当前状态: 总共 {} 个Profile (🎓 {} 个教育路径, 💼 {} 个工作路径)",
            profiles.len(),
            education_count,
            work_count
        );

        Ok(())
    }

    fn create_profile(&mut self) -> Result<()> {
        let mut creator = ProfileCreationStateMachine::new(self.db.clone());
        creator.run()?;
        Ok(())
    }

    fn list_profiles(&self) -> Result<()> {
        let profiles = self
            .db
            .get_profiles()
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        if profiles.is_empty() {
            println!("\n📭 暂无Profile，请先创建一个。");
            return Ok(());
        }

        println!("\n📋 已保存的Profile列表:");
        println!("{}", "=".repeat(80));

        for (i, profile) in profiles.iter().enumerate() {
            let type_icon = match profile.profile_type {
                ProfileType::Education => "🎓",
                ProfileType::Work => "💼",
            };

            let location_str = match &profile.location.city {
                Some(city) => format!("{}, {}", city, profile.location.country),
                None => profile.location.country.clone(),
            };

            println!(
                "{}. {} {} | 📍 {} | 💰 ${:,.0}/年",
                i + 1,
                type_icon,
                profile.name,
                location_str,
                profile.financial_params.initial_salary_usd
            );
        }

        println!("{}", "=".repeat(80));

        // 等待用户按键继续
        Input::<String>::with_theme(&self.theme)
            .with_prompt("按回车键继续")
            .allow_empty(true)
            .interact_text()?;

        Ok(())
    }

    fn view_profile_details(&self) -> Result<()> {
        let profile = self.select_profile("查看详情")?;
        if let Some(profile) = profile {
            self.display_profile_details(&profile);
        }
        Ok(())
    }

    fn edit_profile(&mut self) -> Result<()> {
        println!("\n✏️ 编辑功能开发中...");
        Ok(())
    }

    fn delete_profile(&mut self) -> Result<()> {
        let profile = self.select_profile("删除")?;
        if let Some(profile) = profile {
            let confirm = Confirm::with_theme(&self.theme)
                .with_prompt(&format!(
                    "确定要删除Profile '{}'吗？此操作不可撤销!",
                    profile.name
                ))
                .default(false)
                .interact()?;

            if confirm {
                self.db
                    .delete_profile(profile.id)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                println!("✅ Profile '{}' 已删除", profile.name);
            }
        }
        Ok(())
    }

    fn select_profile(&self, action: &str) -> Result<Option<Profile>> {
        let profiles = self
            .db
            .get_profiles()
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        if profiles.is_empty() {
            println!("\n📭 暂无Profile可{}。", action);
            return Ok(None);
        }

        let profile_names: Vec<String> = profiles
            .iter()
            .map(|p| {
                let type_icon = match p.profile_type {
                    ProfileType::Education => "🎓",
                    ProfileType::Work => "💼",
                };
                format!("{} {}", type_icon, p.name)
            })
            .collect();

        let mut choices = profile_names;
        choices.push("❌ 取消".to_string());

        let choice = Select::with_theme(&self.theme)
            .with_prompt(&format!("选择要{}的Profile", action))
            .items(&choices)
            .interact()?;

        if choice == choices.len() - 1 {
            return Ok(None);
        }

        Ok(Some(profiles[choice].clone()))
    }

    fn display_profile_details(&self, profile: &Profile) {
        println!("\n📋 Profile详细信息");
        println!("{}", "=".repeat(50));
        println!("📝 名称: {}", profile.name);
        println!("🆔 ID: {}", profile.id);

        let type_icon = match profile.profile_type {
            ProfileType::Education => "🎓",
            ProfileType::Work => "💼",
        };
        println!("{} 类型: {:?}", type_icon, profile.profile_type);

        let location_str = match &profile.location.city {
            Some(city) => format!("{}, {}", city, profile.location.country),
            None => profile.location.country.clone(),
        };
        println!("🌍 位置: {} ({})", location_str, profile.location.currency);

        println!("⏱️ 工作延迟: {} 年", profile.work_params.start_delay);
        match profile.work_params.duration_limit {
            Some(limit) => println!("📅 工作年限: {} 年", limit),
            None => println!("📅 工作年限: 无限制"),
        }

        println!(
            "💰 初始年薪: ${:,.0}",
            profile.financial_params.initial_salary_usd
        );
        println!(
            "📈 薪资增长率: {:.1}%",
            profile.financial_params.salary_growth_rate * 100.0
        );
        println!(
            "🏠 生活成本: ${:,.0}/年",
            profile.financial_params.living_cost_usd
        );
        println!(
            "📊 生活成本增长率: {:.1}%",
            profile.financial_params.living_cost_growth * 100.0
        );
        println!("🏛️ 税率: {:.1}%", profile.financial_params.tax_rate * 100.0);

        match &profile.cost_params {
            Some(cost) => {
                println!("💸 总成本: ${:,.0}", cost.total_cost_usd);
                println!("📅 成本分摊: {} 年", cost.cost_duration);
                println!("💸 年均成本: ${:,.0}", cost.annual_cost());
            }
            None => println!("💸 无初期成本"),
        }

        match profile.first_year_opportunity_cost {
            Some(cost) => println!("🎯 机会成本: ${:,.0}", cost),
            None => println!("🎯 无机会成本"),
        }

        if let Some(ref description) = profile.description {
            println!("📝 描述: {}", description);
        }

        println!(
            "🕐 创建时间: {}",
            profile.created_at.format("%Y-%m-%d %H:%M:%S")
        );
        println!(
            "🕐 更新时间: {}",
            profile.updated_at.format("%Y-%m-%d %H:%M:%S")
        );

        println!("{}", "=".repeat(50));

        // 等待用户按键继续
        Input::<String>::with_theme(&self.theme)
            .with_prompt("按回车键继续")
            .allow_empty(true)
            .interact_text()
            .unwrap_or_default();
    }
}

/// 高级Profile编辑器 - 使用状态机实现
pub struct ProfileEditStateMachine {
    current_state:    EditState,
    original_profile: Profile,
    builder:          ProfileBuilder,
    db:               DatabaseManager,
    theme:            ColorfulTheme,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditState {
    Start,
    SelectField,
    EditBasicInfo,
    EditLocationInfo,
    EditWorkParams,
    EditFinancialParams,
    EditCostParams,
    EditOpportunityParams,
    Summary,
    Confirmation,
    Complete,
    Cancelled,
}

impl ProfileEditStateMachine {
    pub fn new(profile: Profile, db: DatabaseManager) -> Self {
        let builder = ProfileBuilder {
            name: Some(profile.name.clone()),
            profile_type: Some(profile.profile_type),
            country: Some(profile.location.country.clone()),
            city: profile.location.city.clone(),
            currency: Some(profile.location.currency.clone()),
            work_start_delay: Some(profile.work_params.start_delay),
            work_duration_limit: Some(profile.work_params.duration_limit),
            initial_salary_usd: Some(profile.financial_params.initial_salary_usd),
            salary_growth_rate: Some(profile.financial_params.salary_growth_rate),
            living_cost_usd: Some(profile.financial_params.living_cost_usd),
            living_cost_growth: Some(profile.financial_params.living_cost_growth),
            tax_rate: Some(profile.financial_params.tax_rate),
            cost_params: Some(profile.cost_params.clone()),
            first_year_opportunity_cost: Some(profile.first_year_opportunity_cost),
            description: profile.description.clone(),
        };

        Self {
            current_state: EditState::Start,
            original_profile: profile,
            builder,
            db,
            theme: ColorfulTheme::default(),
        }
    }

    pub fn run(&mut self) -> Result<Option<Profile>> {
        println!("\n✏️ 编辑Profile: {}", self.original_profile.name);
        println!("{}", "=".repeat(50));

        loop {
            match self.current_state {
                EditState::Start => {
                    self.current_state = EditState::SelectField;
                }
                EditState::SelectField => match self.handle_field_selection()? {
                    UserAction::Continue => continue,
                    UserAction::Cancel => self.current_state = EditState::Cancelled,
                    _ => continue,
                },
                EditState::EditBasicInfo => match self.edit_basic_info()? {
                    UserAction::Continue => self.current_state = EditState::SelectField,
                    UserAction::Cancel => self.current_state = EditState::Cancelled,
                    _ => continue,
                },
                EditState::EditLocationInfo => match self.edit_location_info()? {
                    UserAction::Continue => self.current_state = EditState::SelectField,
                    UserAction::Cancel => self.current_state = EditState::Cancelled,
                    _ => continue,
                },
                EditState::EditWorkParams => match self.edit_work_params()? {
                    UserAction::Continue => self.current_state = EditState::SelectField,
                    UserAction::Cancel => self.current_state = EditState::Cancelled,
                    _ => continue,
                },
                EditState::EditFinancialParams => match self.edit_financial_params()? {
                    UserAction::Continue => self.current_state = EditState::SelectField,
                    UserAction::Cancel => self.current_state = EditState::Cancelled,
                    _ => continue,
                },
                EditState::EditCostParams => match self.edit_cost_params()? {
                    UserAction::Continue => self.current_state = EditState::SelectField,
                    UserAction::Cancel => self.current_state = EditState::Cancelled,
                    _ => continue,
                },
                EditState::EditOpportunityParams => match self.edit_opportunity_params()? {
                    UserAction::Continue => self.current_state = EditState::SelectField,
                    UserAction::Cancel => self.current_state = EditState::Cancelled,
                    _ => continue,
                },
                EditState::Summary => {
                    self.display_changes_summary();
                    match self.handle_save_confirmation()? {
                        UserAction::Confirm => self.current_state = EditState::Complete,
                        UserAction::Back => self.current_state = EditState::SelectField,
                        UserAction::Cancel => self.current_state = EditState::Cancelled,
                        _ => continue,
                    }
                }
                EditState::Complete => {
                    let mut updated_profile = self.builder.clone().build()?;
                    updated_profile.id = self.original_profile.id; // 保持原始ID
                    updated_profile.created_at = self.original_profile.created_at; // 保持创建时间
                    updated_profile.updated_at = Utc::now(); // 更新修改时间

                    self.save_profile(&updated_profile)?;
                    println!("\n✅ Profile '{}' 已成功更新!", updated_profile.name);
                    return Ok(Some(updated_profile));
                }
                EditState::Cancelled => {
                    println!("\n❌ 已取消编辑");
                    return Ok(None);
                }
            }
        }
    }

    fn handle_field_selection(&mut self) -> Result<UserAction> {
        let choices = vec![
            "📝 基本信息 (名称、类型)",
            "🌍 地理位置信息",
            "💼 工作参数",
            "💰 财务参数",
            "💸 成本参数",
            "🎯 机会成本参数",
            "📋 预览所有更改",
            "💾 保存更改",
            "❌ 取消编辑",
        ];

        let choice = Select::with_theme(&self.theme)
            .with_prompt("选择要编辑的部分")
            .items(&choices)
            .interact()?;

        match choice {
            0 => self.current_state = EditState::EditBasicInfo,
            1 => self.current_state = EditState::EditLocationInfo,
            2 => self.current_state = EditState::EditWorkParams,
            3 => self.current_state = EditState::EditFinancialParams,
            4 => self.current_state = EditState::EditCostParams,
            5 => self.current_state = EditState::EditOpportunityParams,
            6 => self.current_state = EditState::Summary,
            7 => self.current_state = EditState::Complete,
            8 => return Ok(UserAction::Cancel),
            _ => return Ok(UserAction::Cancel),
        }

        Ok(UserAction::Continue)
    }

    fn edit_basic_info(&mut self) -> Result<UserAction> {
        println!("\n📝 编辑基本信息");

        let current_name = self.builder.name.as_ref().unwrap();
        let name: String = Input::with_theme(&self.theme)
            .with_prompt("Profile名称")
            .with_initial_text(current_name)
            .interact_text()?;

        let profile_types = vec!["🎓 留学/教育路径", "💼 工作路径"];
        let current_type_idx = match self.builder.profile_type.unwrap() {
            ProfileType::Education => 0,
            ProfileType::Work => 1,
        };

        let profile_type_idx = Select::with_theme(&self.theme)
            .with_prompt("选择路径类型")
            .items(&profile_types)
            .default(current_type_idx)
            .interact()?;

        let profile_type = match profile_type_idx {
            0 => ProfileType::Education,
            1 => ProfileType::Work,
            _ => ProfileType::Work,
        };

        self.builder.name = Some(name);
        self.builder.profile_type = Some(profile_type);

        println!("✅ 基本信息已更新");
        Ok(UserAction::Continue)
    }

    fn edit_location_info(&mut self) -> Result<UserAction> {
        println!("\n🌍 编辑地理位置信息");

        let country: String = Input::with_theme(&self.theme)
            .with_prompt("国家")
            .with_initial_text(self.builder.country.as_ref().unwrap())
            .interact_text()?;

        let city_initial = self.builder.city.as_deref().unwrap_or("");
        let city: String = Input::with_theme(&self.theme)
            .with_prompt("城市 (可选)")
            .with_initial_text(city_initial)
            .allow_empty(true)
            .interact_text()?;

        let currency: String = Input::with_theme(&self.theme)
            .with_prompt("货币代码")
            .with_initial_text(self.builder.currency.as_ref().unwrap())
            .interact_text()?;

        self.builder.country = Some(country);
        self.builder.city = if city.is_empty() { None } else { Some(city) };
        self.builder.currency = Some(currency);

        println!("✅ 地理位置信息已更新");
        Ok(UserAction::Continue)
    }

    fn edit_work_params(&mut self) -> Result<UserAction> {
        println!("\n💼 编辑工作参数");

        let work_start_delay: u32 = Input::with_theme(&self.theme)
            .with_prompt("开始工作前的延迟年数")
            .with_initial_text(&self.builder.work_start_delay.unwrap().to_string())
            .interact_text()?;

        let current_limit = self.builder.work_duration_limit.as_ref().unwrap();
        let has_work_limit = Confirm::with_theme(&self.theme)
            .with_prompt("是否有工作年限限制?")
            .default(current_limit.is_some())
            .interact()?;

        let work_duration_limit = if has_work_limit {
            let initial = current_limit
                .map(|x| x.to_string())
                .unwrap_or_else(|| "10".to_string());
            Some(
                Input::with_theme(&self.theme)
                    .with_prompt("工作年限限制 (年)")
                    .with_initial_text(&initial)
                    .interact_text()?,
            )
        } else {
            None
        };

        self.builder.work_start_delay = Some(work_start_delay);
        self.builder.work_duration_limit = Some(work_duration_limit);

        println!("✅ 工作参数已更新");
        Ok(UserAction::Continue)
    }

    fn edit_financial_params(&mut self) -> Result<UserAction> {
        println!("\n💰 编辑财务参数");

        let initial_salary_usd: f64 = Input::with_theme(&self.theme)
            .with_prompt("初始年薪 (USD)")
            .with_initial_text(&self.builder.initial_salary_usd.unwrap().to_string())
            .interact_text()?;

        let salary_growth_rate: f64 = Input::with_theme(&self.theme)
            .with_prompt("年薪增长率 (小数形式)")
            .with_initial_text(&self.builder.salary_growth_rate.unwrap().to_string())
            .interact_text()?;

        let living_cost_usd: f64 = Input::with_theme(&self.theme)
            .with_prompt("初始年生活成本 (USD)")
            .with_initial_text(&self.builder.living_cost_usd.unwrap().to_string())
            .interact_text()?;

        let living_cost_growth: f64 = Input::with_theme(&self.theme)
            .with_prompt("生活成本年增长率 (小数形式)")
            .with_initial_text(&self.builder.living_cost_growth.unwrap().to_string())
            .interact_text()?;

        let tax_rate: f64 = Input::with_theme(&self.theme)
            .with_prompt("税率 (小数形式)")
            .with_initial_text(&self.builder.tax_rate.unwrap().to_string())
            .interact_text()?;

        // 验证输入
        if salary_growth_rate < 0.0 || salary_growth_rate > 1.0 {
            println!("⚠️ 薪资增长率应该在0-1之间");
            return Ok(UserAction::Retry);
        }

        if tax_rate < 0.0 || tax_rate > 1.0 {
            println!("⚠️ 税率应该在0-1之间");
            return Ok(UserAction::Retry);
        }

        self.builder.initial_salary_usd = Some(initial_salary_usd);
        self.builder.salary_growth_rate = Some(salary_growth_rate);
        self.builder.living_cost_usd = Some(living_cost_usd);
        self.builder.living_cost_growth = Some(living_cost_growth);
        self.builder.tax_rate = Some(tax_rate);

        println!("✅ 财务参数已更新");
        Ok(UserAction::Continue)
    }

    fn edit_cost_params(&mut self) -> Result<UserAction> {
        println!("\n💸 编辑成本参数");

        let current_cost_params = self.builder.cost_params.as_ref().unwrap();
        let has_costs = Confirm::with_theme(&self.theme)
            .with_prompt("是否有初期成本 (如学费)?")
            .default(current_cost_params.is_some())
            .interact()?;

        let cost_params = if has_costs {
            let (current_total, current_duration) = match current_cost_params {
                Some(cost) => (
                    cost.total_cost_usd.to_string(),
                    cost.cost_duration.to_string(),
                ),
                None => ("100000".to_string(), "2".to_string()),
            };

            let total_cost: f64 = Input::with_theme(&self.theme)
                .with_prompt("总成本 (USD)")
                .with_initial_text(&current_total)
                .interact_text()?;

            let duration: u32 = Input::with_theme(&self.theme)
                .with_prompt("成本分摊年数")
                .with_initial_text(&current_duration)
                .interact_text()?;

            if total_cost < 0.0 {
                println!("⚠️ 总成本不能为负数");
                return Ok(UserAction::Retry);
            }

            if duration == 0 {
                println!("⚠️ 成本分摊年数必须大于0");
                return Ok(UserAction::Retry);
            }

            Some(CostParams::new(total_cost, duration)?)
        } else {
            None
        };

        self.builder.cost_params = Some(cost_params);

        println!("✅ 成本参数已更新");
        Ok(UserAction::Continue)
    }

    fn edit_opportunity_params(&mut self) -> Result<UserAction> {
        println!("\n🎯 编辑机会成本参数");

        let current_opp_cost = self.builder.first_year_opportunity_cost.as_ref().unwrap();
        let has_opportunity_cost = Confirm::with_theme(&self.theme)
            .with_prompt("第一年是否有机会成本投资?")
            .default(current_opp_cost.is_some())
            .interact()?;

        let opportunity_cost = if has_opportunity_cost {
            let initial = current_opp_cost
                .map(|x| x.to_string())
                .unwrap_or_else(|| "100000".to_string());
            Some(
                Input::with_theme(&self.theme)
                    .with_prompt("第一年机会成本投资金额 (USD)")
                    .with_initial_text(&initial)
                    .interact_text()?,
            )
        } else {
            None
        };

        let current_desc = self.builder.description.as_deref().unwrap_or("");
        let description: String = Input::with_theme(&self.theme)
            .with_prompt("描述 (可选)")
            .with_initial_text(current_desc)
            .allow_empty(true)
            .interact_text()?;

        self.builder.first_year_opportunity_cost = Some(opportunity_cost);
        self.builder.description = if description.is_empty() {
            None
        } else {
            Some(description)
        };

        println!("✅ 机会成本参数已更新");
        Ok(UserAction::Continue)
    }

    fn display_changes_summary(&self) {
        println!("\n📋 更改摘要");
        println!("{}", "=".repeat(60));

        // 比较并显示更改
        self.compare_field(
            "📝 名称",
            &self.original_profile.name,
            self.builder.name.as_ref().unwrap(),
        );

        let original_type = format!("{:?}", self.original_profile.profile_type);
        let new_type = format!("{:?}", self.builder.profile_type.unwrap());
        self.compare_field("🎯 类型", &original_type, &new_type);

        self.compare_field(
            "🌍 国家",
            &self.original_profile.location.country,
            self.builder.country.as_ref().unwrap(),
        );

        let original_city = self
            .original_profile
            .location
            .city
            .as_deref()
            .unwrap_or("(无)");
        let new_city = self.builder.city.as_deref().unwrap_or("(无)");
        self.compare_field("🏙️ 城市", original_city, new_city);

        self.compare_field(
            "💱 货币",
            &self.original_profile.location.currency,
            self.builder.currency.as_ref().unwrap(),
        );

        self.compare_numeric_field(
            "⏱️ 工作延迟 (年)",
            self.original_profile.work_params.start_delay,
            self.builder.work_start_delay.unwrap(),
        );

        let original_limit = self
            .original_profile
            .work_params
            .duration_limit
            .map(|x| x.to_string())
            .unwrap_or_else(|| "无限制".to_string());
        let new_limit = self
            .builder
            .work_duration_limit
            .as_ref()
            .unwrap()
            .map(|x| x.to_string())
            .unwrap_or_else(|| "无限制".to_string());
        self.compare_field("📅 工作年限", &original_limit, &new_limit);

        self.compare_currency_field(
            "💰 初始年薪",
            self.original_profile.financial_params.initial_salary_usd,
            self.builder.initial_salary_usd.unwrap(),
        );

        self.compare_percentage_field(
            "📈 薪资增长率",
            self.original_profile.financial_params.salary_growth_rate,
            self.builder.salary_growth_rate.unwrap(),
        );

        self.compare_currency_field(
            "🏠 生活成本",
            self.original_profile.financial_params.living_cost_usd,
            self.builder.living_cost_usd.unwrap(),
        );

        self.compare_percentage_field(
            "📊 生活成本增长率",
            self.original_profile.financial_params.living_cost_growth,
            self.builder.living_cost_growth.unwrap(),
        );

        self.compare_percentage_field(
            "🏛️ 税率",
            self.original_profile.financial_params.tax_rate,
            self.builder.tax_rate.unwrap(),
        );

        // 成本参数比较
        let original_cost = match &self.original_profile.cost_params {
            Some(cost) => format!("${:,.0} ({}年)", cost.total_cost_usd, cost.cost_duration),
            None => "无成本".to_string(),
        };
        let new_cost = match self.builder.cost_params.as_ref().unwrap() {
            Some(cost) => format!("${:,.0} ({}年)", cost.total_cost_usd, cost.cost_duration),
            None => "无成本".to_string(),
        };
        self.compare_field("💸 成本参数", &original_cost, &new_cost);

        // 机会成本比较
        let original_opp = self
            .original_profile
            .first_year_opportunity_cost
            .map(|x| format!("${:,.0}", x))
            .unwrap_or_else(|| "无".to_string());
        let new_opp = self
            .builder
            .first_year_opportunity_cost
            .as_ref()
            .unwrap()
            .map(|x| format!("${:,.0}", x))
            .unwrap_or_else(|| "无".to_string());
        self.compare_field("🎯 机会成本", &original_opp, &new_opp);

        println!("{}", "=".repeat(60));
    }

    fn compare_field(&self, label: &str, original: &str, new: &str) {
        if original != new {
            println!("{}: {} → {}", label, original, new);
        } else {
            println!("{}: {} (无更改)", label, original);
        }
    }

    fn compare_numeric_field<T: fmt::Display + PartialEq>(&self, label: &str, original: T, new: T) {
        if original != new {
            println!("{}: {} → {}", label, original, new);
        } else {
            println!("{}: {} (无更改)", label, original);
        }
    }

    fn compare_currency_field(&self, label: &str, original: f64, new: f64) {
        if (original - new).abs() > 0.01 {
            println!("{}: ${:,.0} → ${:,.0}", label, original, new);
        } else {
            println!("{}: ${:,.0} (无更改)", label, original);
        }
    }

    fn compare_percentage_field(&self, label: &str, original: f64, new: f64) {
        if (original - new).abs() > 0.001 {
            println!("{}: {:.1}% → {:.1}%", label, original * 100.0, new * 100.0);
        } else {
            println!("{}: {:.1}% (无更改)", label, original * 100.0);
        }
    }

    fn handle_save_confirmation(&self) -> Result<UserAction> {
        let choices = vec!["💾 保存更改", "⬅️ 继续编辑", "❌ 取消"];

        let choice = Select::with_theme(&self.theme)
            .with_prompt("请选择操作")
            .items(&choices)
            .default(0)
            .interact()?;

        match choice {
            0 => Ok(UserAction::Confirm),
            1 => Ok(UserAction::Back),
            2 => Ok(UserAction::Cancel),
            _ => Ok(UserAction::Cancel),
        }
    }

    fn save_profile(&mut self, profile: &Profile) -> Result<()> {
        self.db
            .save_profile(profile)
            .map_err(|e| anyhow::anyhow!("保存Profile失败: {}", e))
    }
}

impl ProfileManager {
    fn edit_profile(&mut self) -> Result<()> {
        let profile = self.select_profile("编辑")?;
        if let Some(profile) = profile {
            let mut editor = ProfileEditStateMachine::new(profile, self.db.clone());
            editor.run()?;
        }
        Ok(())
    }
}
