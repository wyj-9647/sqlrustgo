# SQLRustGo 设计文档

## 设计目标
- 模块化设计，便于维护和扩展
- 清晰的接口定义，支持单元测试
- 性能优先，关键路径优化

## 模块设计

### Parser 模块
- **文件位置**：`src/parser/`
- **主要结构**：`Lexer`、`Parser`、`Token`、`Statement`
- **设计思路**：递归下降解析

### Planner 模块
- **文件位置**：`src/planner/`
- **主要结构**：`LogicalPlan`、`PhysicalPlan`、`Optimizer`
- **优化规则**：谓词下推、投影裁剪、常量折叠

### Executor 模块
- **文件位置**：`src/executor/`
- **主要结构**：`Executor` trait、`TableScan`、`Filter`、`Projection`
- **设计思路**：火山模型（逐行处理数据）

### Storage 模块
- **文件位置**：`src/storage/`
- **主要结构**：`Page`、`BufferPool`、`PageManager`
- **设计思路**：LRU缓存策略

## 技术选型
| 组件 | 选择 | 原因 |
|------|------|------|
| 编程语言 | Rust | 内存安全、高性能、零成本抽象 |
| 解析器 | 手写递归下降 | 学习目的，便于理解原理 |
| 存储 | 基于页的文件存储 | 简单高效，符合教学目的 |

## 接口定义

### Parser 接口
```rust
pub trait Parser {
    fn parse(&mut self, sql: &str) -> Result<Statement, ParseError>;
}