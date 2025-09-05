# 配置文件说明

本目录包含留学成本估算器的配置文件，基于 [27spring 文档](../docs/src/all_schools.md) 中的学校数据生成。

## 配置文件列表

### `us_schools.toml` - 美国学校配置
包含美国CS硕士项目的学校配置，包括：
- Texas A&M University (排名48)
- University of Texas at Dallas (排名52) 
- University of Central Florida (排名60)
- Arizona State University (排名48)
- San Jose State University (硅谷位置)
- The University of Texas at Austin (排名8，顶级院校)

### `canada_schools.toml` - 加拿大学校配置
包含加拿大CS硕士项目的学校配置，包括：

**基础院校选项：**
- University of Calgary (推荐，排名前10)
- University of Saskatchewan (匹配，排名前15)
- Lakehead University (保底，GPA要求2.4)
- Laurentian University (保底，支持双录取)

**大城市院校选项：**
- Concordia University (蒙特利尔，接受工作经验)
- York University (多伦多，工作经验加分)
- Carleton University (渥太华，政府实习机会)
- University of Ottawa (渥太华，政府科技实习)

## 使用方法

### CLI 命令行模式

```bash
# 查看帮助
cargo run -- --help

# 列出美国学校
cargo run -- list --country us

# 计算Texas A&M大学成本
cargo run -- calculate --country us --school "TAMU" --years 2

# 包含机会成本的详细报告
cargo run -- calculate --country us --school "TAMU" --include-opportunity-cost --format report

# 比较所有美国学校
cargo run -- compare --country us

# 启动TUI界面
cargo run -- tui
```

### TUI 交互界面模式

```bash
# 启动图形化交互界面
cargo run -- tui
```

在TUI界面中：
- 使用方向键导航
- Tab键切换标签页
- Enter键确认选择
- h键显示帮助
- q键退出

## 配置文件结构

### 汇率设置
```toml
[exchange_rates]
usd_to_cny = 7.2  # 美元兑人民币
cad_to_usd = 0.73  # 加元兑美元（仅加拿大配置）
last_updated = "2024-01-01"
```

### 应用设置
```toml
[app_settings]
default_study_years = 2
default_study_months = 0
shanghai_reference_salary = 375000.0  # 人民币
tax_rate = 0.282  # 28.2% 综合税负率
shanghai_living_cost = 91200.0  # 人民币年生活成本
```

### 学校配置
```toml
[[schools]]
name = "学校名称"
tuition_per_year = 26000.0  # 年学费
living_cost_per_month = 1800.0  # 月生活费
location = "地理位置"
ranking = 48  # CSRankings排名（可选）
```

### 默认费用配置
包含以下费用类别的默认值：
- `application` - 申请阶段费用
- `visa_legal` - 签证法律费用
- `insurance_medical` - 保险医疗费用
- `transportation` - 交通搬迁费用
- `accommodation` - 住宿安置费用
- `financial_services` - 金融服务费用
- `communication` - 通讯网络费用
- `study` - 学习相关费用
- `job_search` - 求职费用
- `emergency` - 应急储备费用

## 修改配置

1. **更新学费和生活费**：编辑对应学校的 `tuition_per_year` 和 `living_cost_per_month`
2. **更新汇率**：修改 `exchange_rates` 部分的汇率和更新日期
3. **调整默认费用**：根据实际情况修改 `default_costs` 各类别的费用
4. **添加新学校**：在 `[[schools]]` 部分添加新的学校配置

## 注意事项

- 美国配置费用单位为美元 (USD)
- 加拿大配置费用单位为加元 (CAD)
- 所有默认费用基于2024年市场数据
- 建议定期更新汇率和市场数据
- 配置修改后重启程序生效

## 数据来源

配置数据基于以下来源：
- [CSRankings.org](https://csrankings.org/) - 计算机科学专业排名
- [27spring 文档](../docs/src/all_schools.md) - 详细学校分析
- 实际留学生经验和市场调研数据
