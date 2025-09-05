use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use plotters::prelude::*;
use tabled::{Table, Tabled, settings::{Alignment, Modify, Style, object::Columns}};
use anyhow::Result;

use crate::models::{Profile, PathCalculationParams};

#[derive(Debug, Clone, Serialize, Deserialize, SmartDefault)]
pub struct ROICalculator {
    // 基础参数
    #[default = 7.2] // 汇率 USD:CNY
    pub exchange_rate:          f64,
    #[default = 0.10] // S&P 500 年化回报率
    pub investment_return_rate: f64,
    #[default = 0.20] // 可支配收入中用于投资的比例（20%）
    pub investment_portion:     f64,
    #[default = 10] // 总分析年限
    pub total_years:            u32,
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
    /// * `profile` - 职业发展路径Profile
    ///
    /// # Returns
    /// 返回年度财务数据向量
    fn calculate_path_data(&self, profile: &Profile) -> Vec<PathYearlyData> {
        let params = profile.to_path_params();
        let mut results = Vec::new();
        let mut total_cash = 0.0;
        let mut total_investment = 0.0;
        let mut total_investment_principal = 0.0;
        let mut total_cost_paid = 0.0;

        for year in 1..=self.total_years {
            // 确定工作年数
            let work_year = self.get_work_year(year, &params);

            // 计算年度财务数据
            let (income_usd, net_income_usd, living_cost_usd, disposable_income_usd) = self.calculate_year_finances(year, work_year, &params, &mut total_cost_paid);

            // 计算投资分配
            let (investment_amount, cash_savings) = self.calculate_investment_allocation(year, disposable_income_usd, &params);

            // 计算投资收益
            let (existing_return, new_investment_return) = self.calculate_investment_returns(total_investment, investment_amount);
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
        self.calculate_path_data(profile)
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

    /// Profile比较分析
    pub fn analyze_profile_comparison(&self, profile1: &Profile, profile2: &Profile) {
        let results = self.compare_profiles(profile1, profile2);
        let (roi1, roi2, roi_diff) = self.calculate_profile_final_roi(profile1, profile2);

        println!(
            "=== {} vs {} ROI 详细分析 ===\n",
            profile1.name,
            profile2.name
        );

        self.print_profile_parameters(profile1, profile2);
        self.print_profile_yearly_data(&results);
        self.print_profile_roi_summary(roi1, roi2, roi_diff, &results, profile1, profile2);
    }

    /// 打印Profile参数对比
    fn print_profile_parameters(&self, profile1: &Profile, profile2: &Profile) {
        println!("=== Profile参数对比 ==");

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
                    crate::models::ProfileType::Education => "教育路径".to_string(),
                    crate::models::ProfileType::Work => "工作路径".to_string(),
                },
                value2: match profile2.profile_type {
                    crate::models::ProfileType::Education => "教育路径".to_string(),
                    crate::models::ProfileType::Work => "工作路径".to_string(),
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
        println!("=== 年度详细数据对比 ==");

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
                profile1.name,
                data.year,
                profile2.name
            );
        } else {
            println!(
                "\n⚠️ 在{}年分析期内，{}未能追平{}",
                self.total_years,
                profile1.name,
                profile2.name
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