# 🏫 留学成本估算器

> 基于真实数据的智能留学成本计算工具，支持美国和加拿大留学项目

## ✨ 特性

- 🎯 **精确计算**: 基于2024年市场数据和实际留学生经验
- 🌍 **多国支持**: 美国和加拿大CS硕士项目
- 📊 **详细分析**: 包含隐性成本和机会成本分析
- 🎮 **多种界面**: CLI命令行和TUI交互界面
- ⚙️ **灵活配置**: YAML配置文件，支持自定义
- 📈 **成本档位**: 节约型、标准型、舒适型三种消费水平

## 📦 安装

```bash
git clone https://github.com/your-username/27spring.git
cd 27spring
cargo build --release
```

## 🚀 快速开始

### 1. 生成配置文件

```bash
cargo run config generate
```

### 2. 查看可选学校

```bash
# 查看美国学校
cargo run list --country us

# 查看加拿大学校  
cargo run list --country canada

# 查看所有学校
cargo run list
```

### 3. 计算留学成本

```bash
# 计算Texas A&M大学成本
cargo run calculate --country us --school "Texas A&M University" --years 2

# 包含机会成本分析
cargo run calculate --country us --school "TAMU" --include-opportunity-cost

# 节约型消费水平
cargo run calculate --country us --school "TAMU" --cost-level budget
```

### 4. 比较学校成本

```bash
# 比较美国所有学校
cargo run compare --country us

# 比较加拿大学校，输出JSON格式
cargo run compare --country canada --format json
```

### 5. 启动TUI界面

```bash
cargo run tui
```

## 📋 CLI 命令详解

### calculate - 计算成本

```bash
cargo run calculate [OPTIONS]

选项:
  -c, --country <COUNTRY>     目标国家 [us|canada|both]
  -s, --school <SCHOOL>       学校名称或简称
  -y, --years <YEARS>         学习年数 [默认: 2]
  -m, --months <MONTHS>       额外月数 [默认: 0]
  -l, --cost-level <LEVEL>    成本档位 [budget|standard|comfortable]
      --include-opportunity-cost  包含机会成本分析
  -f, --format <FORMAT>       输出格式 [table|json|yaml|report]
```

### compare - 比较学校

```bash
cargo run compare [OPTIONS]

选项:
  -c, --country <COUNTRY>     目标国家 [us|canada|both]
  -y, --years <YEARS>         学习年数 [默认: 2]
  -m, --months <MONTHS>       额外月数 [默认: 0]
  -l, --cost-level <LEVEL>    成本档位 [budget|standard|comfortable]
  -f, --format <FORMAT>       输出格式 [table|json|yaml|report]
```

### list - 列出学校

```bash
cargo run list [OPTIONS]

选项:
  -c, --country <COUNTRY>     目标国家 [us|canada]
  -t, --school-type <TYPE>    学校类型过滤
```

### config - 配置管理

```bash
# 显示当前配置
cargo run config show

# 生成配置文件
cargo run config generate [--force]

# 验证配置文件
cargo run config validate
```

## 🎮 TUI 界面操作

启动TUI界面：`cargo run tui`

### 快捷键

| 按键 | 功能 |
|------|------|
| `q` | 退出程序 |
| `h` / `F1` | 显示/隐藏帮助 |
| `Tab` / `←→` | 切换标签页 |
| `↑↓` | 选择学校 |
| `Enter` | 确认选择 |
| `1` | 切换到美国 |
| `2` | 切换到加拿大 |
| `3` | 两国对比模式 |
| `b` | 节约型档位 |
| `s` | 标准型档位 |
| `c` | 舒适型档位 |
| `+` | 增加学习年数 |
| `-` | 减少学习年数 |

### 界面说明

1. **学校选择**: 浏览和选择目标学校
2. **成本计算**: 查看详细的费用分解
3. **对比分析**: 多所学校成本对比
4. **设置**: 调整参数和查看配置

## ⚙️ 配置文件

配置文件位于 `config/` 目录：

- `schools.yaml` - 学校信息配置
- `regions.yaml` - 地区和汇率配置  
- `costs.yaml` - 费用项目配置

### 学校配置示例

```yaml
schools:
  us:
    target:
      - name: "Texas A&M University"
        short_name: "TAMU"
        tuition_per_year: 26000
        region: "texas"
        ranking:
          csrankings: 48
          category: "匹配院校"
```

### 地区配置示例

```yaml
regions:
  us:
    texas:
      name: "德克萨斯州"
      living_cost_per_month: 1800
      cost_factors:
        housing: 1.0
        food: 0.9
```

## 💰 成本计算说明

### 基础成本
- **学费**: 按年计算的国际学生学费
- **生活费**: 包含住宿、饮食、交通等基本开销

### 附加费用
- **申请阶段**: GRE、TOEFL、申请费、成绩单认证等
- **签证法律**: F1签证费、SEVIS费、面试费等
- **保险医疗**: 健康保险、牙科保险、体检费等
- **交通搬迁**: 机票、行李费、当地交通等
- **住宿安置**: 押金、中介费、家具电器等
- **金融服务**: 开户费、汇款费、汇率损失等
- **通讯网络**: 手机套餐、网络费、国际通话等
- **学习相关**: 教材、软件许可、实验材料等
- **求职费用**: 面试交通、职业装、培训等
- **应急储备**: 家庭紧急情况、意外支出等

### 机会成本
基于在上海工作的净储蓄损失，考虑：
- 年薪水平（入门级、中级、高级、管理层）
- 综合税负率（个税 + 社保公积金）
- 生活成本

## 📊 使用示例

### 计算Texas A&M大学2年成本

```bash
cargo run calculate --country us --school "TAMU" --years 2 --format report
```

输出：
```
📊 详细成本报告
================
🏫 学校: Texas A&M University
📍 地区: 德克萨斯州
📅 学习时长: 2年0个月
💡 成本档位: 标准型

💰 成本分解:
• 基础成本 (学费+生活费): $95,200
• 附加费用: $43,520
• 总留学成本: $138,720
```

### 比较所有美国学校

```bash
cargo run compare --country us --format table
```

输出：
```
┌─────┬─────────────────────────────────┬─────────────┐
│ 排名 │ 学校名称                        │ 估算成本     │
├─────┼─────────────────────────────────┼─────────────┤
│  1  │ Texas A&M University           │    $95,200 │
│  2  │ University of Central Florida   │   $112,332 │
│  3  │ University of Texas at Dallas   │   $111,060 │
└─────┴─────────────────────────────────┴─────────────┘
```

## 🔧 开发

### 项目结构

```
src/
├── main.rs          # 主程序入口
├── cli.rs           # CLI命令行解析
├── config.rs        # 配置文件处理
└── tui.rs           # TUI交互界面

config/
├── schools.yaml     # 学校配置
├── regions.yaml     # 地区配置
├── costs.yaml       # 费用配置
└── README.md        # 配置说明
```

### 添加新学校

1. 编辑 `config/schools.yaml`
2. 在对应国家和类别下添加学校信息
3. 确保地区配置存在于 `config/regions.yaml`

### 自定义费用

1. 编辑 `config/costs.yaml`
2. 修改对应国家的默认费用
3. 可以添加新的成本档位

## 📖 相关文档

- [配置文件说明](config/README.md)
- [学校选择指南](docs/src/all_schools.md)
- [ROI投资回报分析](docs/src/roi.md)

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License