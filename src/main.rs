use std::fmt;

/// 投资回报率计算结果
#[derive(Debug, Clone)]
pub struct ROIResult {
    pub simple_roi:     f64, // 简单投资回报率 (%)
    pub annualized_roi: f64, // 年化投资回报率 (%)
    pub total_return:   f64, // 总收益
    pub profit_loss:    f64, // 盈亏
}

impl fmt::Display for ROIResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "投资回报率分析:\n简单ROI: {:.2}%\n年化ROI: {:.2}%\n总收益: {:.2}\n盈亏: {:.2}",
            self.simple_roi, self.annualized_roi, self.total_return, self.profit_loss
        )
    }
}

/// 投资回报率计算器
pub struct InvestmentCalculator;

impl InvestmentCalculator {
    /// 计算简单投资回报率
    /// 公式: ROI = (当前价值 - 初始投资) / 初始投资 * 100%
    pub fn simple_roi(initial_investment: f64, current_value: f64) -> Result<f64, &'static str> {
        if initial_investment <= 0.0 {
            return Err("初始投资金额必须大于0");
        }

        let roi = (current_value - initial_investment) / initial_investment * 100.0;
        Ok(roi)
    }

    /// 计算年化投资回报率 (CAGR - 复合年增长率)
    /// 公式: CAGR = (期末价值/期初价值)^(1/年数) - 1
    pub fn annualized_roi(
        initial_investment: f64,
        current_value: f64,
        years: f64,
    ) -> Result<f64, &'static str> {
        if initial_investment <= 0.0 {
            return Err("初始投资金额必须大于0");
        }
        if years <= 0.0 {
            return Err("投资年限必须大于0");
        }
        if current_value <= 0.0 {
            return Err("当前价值必须大于0");
        }

        let cagr = (current_value / initial_investment).powf(1.0 / years) - 1.0;
        Ok(cagr * 100.0)
    }

    /// 计算完整的投资回报率分析
    pub fn calculate_roi(
        initial_investment: f64,
        current_value: f64,
        years: f64,
    ) -> Result<ROIResult, &'static str> {
        let simple_roi = Self::simple_roi(initial_investment, current_value)?;
        let annualized_roi = Self::annualized_roi(initial_investment, current_value, years)?;
        let total_return = current_value - initial_investment;
        let profit_loss = total_return;

        Ok(ROIResult {
            simple_roi,
            annualized_roi,
            total_return,
            profit_loss,
        })
    }

    /// 计算定期投资的回报率 (适用于定投场景)
    pub fn periodic_investment_roi(
        periodic_amount: f64,
        periods: u32,
        final_value: f64,
        years: f64,
    ) -> Result<ROIResult, &'static str> {
        if periodic_amount <= 0.0 || periods == 0 {
            return Err("定投金额和期数必须大于0");
        }

        let total_investment = periodic_amount * periods as f64;
        Self::calculate_roi(total_investment, final_value, years)
    }

    /// 计算需要多少年才能达到目标回报率
    pub fn years_to_target_return(
        initial_investment: f64,
        target_value: f64,
        annual_return_rate: f64,
    ) -> Result<f64, &'static str> {
        if initial_investment <= 0.0 || target_value <= initial_investment {
            return Err("参数无效：初始投资必须大于0，目标值必须大于初始投资");
        }
        if annual_return_rate <= 0.0 {
            return Err("年化收益率必须大于0");
        }

        let years =
            (target_value / initial_investment).ln() / (1.0 + annual_return_rate / 100.0).ln();
        Ok(years)
    }

    /// 计算复利终值
    pub fn compound_interest_future_value(
        principal: f64,
        annual_rate: f64,
        compound_frequency: u32,
        years: f64,
    ) -> Result<f64, &'static str> {
        if principal <= 0.0 || annual_rate < 0.0 || compound_frequency == 0 || years < 0.0 {
            return Err("参数无效");
        }

        let rate_per_period = annual_rate / 100.0 / compound_frequency as f64;
        let total_periods = compound_frequency as f64 * years;
        let future_value = principal * (1.0 + rate_per_period).powf(total_periods);

        Ok(future_value)
    }
}

// 使用示例
fn main() {
    println!("=== 投资回报率计算器示例 ===\n");

    // 示例1: 计算简单投资的ROI
    println!("示例1: 股票投资");
    println!("初始投资: ¥10,000, 当前价值: ¥12,500, 投资时间: 1.5年");

    match InvestmentCalculator::calculate_roi(10000.0, 12500.0, 1.5) {
        Ok(result) => println!("{}\n", result),
        Err(e) => println!("计算错误: {}", e),
    }

    // 示例2: 定期投资
    println!("示例2: 基金定投");
    println!("每月投资: ¥1,000, 投资12个月, 最终价值: ¥13,000");

    match InvestmentCalculator::periodic_investment_roi(1000.0, 12, 13000.0, 1.0) {
        Ok(result) => println!("{}\n", result),
        Err(e) => println!("计算错误: {}", e),
    }

    // 示例3: 计算达到目标需要的时间
    println!("示例3: 投资翻倍需要多长时间");
    println!("初始投资: ¥50,000, 目标: ¥100,000, 年化收益率: 8%");

    match InvestmentCalculator::years_to_target_return(50000.0, 100000.0, 8.0) {
        Ok(years) => println!("需要 {:.1} 年可以翻倍\n", years),
        Err(e) => println!("计算错误: {}", e),
    }

    // 示例4: 复利计算
    println!("示例4: 复利投资");
    println!("本金: ¥20,000, 年利率: 6%, 按月复利, 投资期: 5年");

    match InvestmentCalculator::compound_interest_future_value(20000.0, 6.0, 12, 5.0) {
        Ok(fv) => {
            let roi_result = InvestmentCalculator::calculate_roi(20000.0, fv, 5.0).unwrap();
            println!("最终价值: ¥{:.2}", fv);
            println!("{}", roi_result);
        }
        Err(e) => println!("计算错误: {}", e),
    }
}
