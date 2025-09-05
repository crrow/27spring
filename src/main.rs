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

use anyhow::Result;
use dialoguer::Select;

mod models;
mod db;
mod calculator;
mod ui;

use db::DatabaseManager;
use calculator::ROICalculator;
use ui::ProfileCreator;

fn main() -> Result<()> {
    let calculator = ROICalculator::default();

    // 初始化数据库
    let db = DatabaseManager::new().map_err(|e| anyhow::Error::new(e))?;
    let mut profile_creator = ProfileCreator::new(db);

    println!("🎯 ROI Calculator - Profile版");
    println!("===========================");

    loop {
        let actions = vec![
            "创建新Profile",
            "查看已有Profile",
            "比较Profile",
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
