mod projection;

use std::fmt;

/// 执行器错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum ExecError {
    OpenError(String),
    ExecuteError(String),
    CloseError(String),
    TypeError(String),
    Other(String),
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecError::OpenError(msg) => write!(f, "Open error: {}", msg),
            ExecError::ExecuteError(msg) => write!(f, "Execute error: {}", msg),
            ExecError::CloseError(msg) => write!(f, "Close error: {}", msg),
            ExecError::TypeError(msg) => write!(f, "Type error: {}", msg),
            ExecError::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl std::error::Error for ExecError {}

/// 值类型
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Text(String),
    Null,
}

/// 数据行
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub values: Vec<Value>,
}

impl Row {
    pub fn new(values: Vec<Value>) -> Self {
        Row { values }
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }
}

/// 字段类型
#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Int,
    Float,
    Text,
}

/// 字段定义
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub data_type: DataType,
}

impl Field {
    pub fn new(name: &str, data_type: DataType) -> Self {
        Field {
            name: name.to_string(),
            data_type,
        }
    }
}

/// 表结构
#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    pub fields: Vec<Field>,
}

impl Schema {
    pub fn new(fields: Vec<Field>) -> Self {
        Schema { fields }
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    pub fn field_names(&self) -> Vec<String> {
        self.fields.iter().map(|f| f.name.clone()).collect()
    }
}

/// 结果集
#[derive(Debug, Clone)]
pub struct RecordBatch {
    pub schema: Schema,
    pub rows: Vec<Row>,
}

impl RecordBatch {
    pub fn new(schema: Schema, rows: Vec<Row>) -> Self {
        RecordBatch { schema, rows }
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
}

/// 执行器接口（火山模型）
pub trait Executor: Send {
    fn open(&mut self) -> Result<(), ExecError>;
    fn next(&mut self) -> Result<Option<RecordBatch>, ExecError>;
    fn close(&mut self) -> Result<(), ExecError>;
    fn schema(&self) -> &Schema;
}

pub mod table_scan;
pub mod filter;

pub use table_scan::TableScanExecutor;
pub use filter::{FilterExecutor, FilterPredicate};
pub use projection::ProjectionExecutor;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_person_table() -> (Schema, Vec<Row>) {
        let schema = Schema::new(vec![
            Field::new("id", DataType::Int),
            Field::new("name", DataType::Text),
            Field::new("age", DataType::Int),
        ]);
        let rows = vec![
            Row::new(vec![Value::Int(1), Value::Text("Alice".to_string()), Value::Int(25)]),
            Row::new(vec![Value::Int(2), Value::Text("Bob".to_string()), Value::Int(30)]),
            Row::new(vec![Value::Int(3), Value::Text("Charlie".to_string()), Value::Int(35)]),
            Row::new(vec![Value::Int(4), Value::Text("David".to_string()), Value::Int(20)]),
            Row::new(vec![Value::Int(5), Value::Text("Eve".to_string()), Value::Int(28)]),
        ];
        (schema, rows)
    }

    #[test]
    fn test_table_scan() {
        let (schema, data) = create_person_table();
        let mut executor = TableScanExecutor::new("users", schema, data);
        executor.open().unwrap();
        let mut count = 0;
        while let Some(batch) = executor.next().unwrap() {
            count += batch.row_count();
        }
        executor.close().unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_filter_executor() {
        let (schema, data) = create_person_table();
        let scan = TableScanExecutor::new("users", schema, data);
        let filter = FilterExecutor::new(
            Box::new(scan),
            FilterPredicate::Gt("age".to_string(), Value::Int(25)),
        );
        let mut executor = filter;
        executor.open().unwrap();
        let mut rows = Vec::new();
        while let Some(batch) = executor.next().unwrap() {
            rows.extend(batch.rows);
        }
        executor.close().unwrap();
        assert_eq!(rows.len(), 3);
    }

    #[test]
    fn test_projection_executor() {
        let (schema, data) = create_person_table();
        let scan = TableScanExecutor::new("users", schema, data);
        let projection = ProjectionExecutor::new(
            Box::new(scan),
            vec!["name".to_string(), "age".to_string()],
        ).unwrap();
        let mut executor = projection;
        executor.open().unwrap();
        let mut rows = Vec::new();
        while let Some(batch) = executor.next().unwrap() {
            rows.extend(batch.rows);
        }
        executor.close().unwrap();
        assert_eq!(executor.schema().field_count(), 2);
        assert_eq!(rows.len(), 5);
    }

    #[test]
    fn test_filter_and_projection() {
        let (schema, data) = create_person_table();
        let scan = TableScanExecutor::new("users", schema, data);
        let filter = FilterExecutor::new(
            Box::new(scan),
            FilterPredicate::Gt("age".to_string(), Value::Int(25)),
        );
        let projection = ProjectionExecutor::new(
            Box::new(filter),
            vec!["name".to_string()],
        ).unwrap();
        let mut executor = projection;
        executor.open().unwrap();
        let mut names = Vec::new();
        while let Some(batch) = executor.next().unwrap() {
            for row in batch.rows {
                if let Some(Value::Text(name)) = row.get(0) {
                    names.push(name.clone());
                }
            }
        }
        executor.close().unwrap();
        assert_eq!(names, vec!["Bob", "Charlie", "Eve"]);
    }
}