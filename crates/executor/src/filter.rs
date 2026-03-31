use crate::*;

#[derive(Debug, Clone)]
pub enum FilterPredicate {
    Eq(String, Value),
    Gt(String, Value),
    Lt(String, Value),
    Ne(String, Value),
}

impl FilterPredicate {
    fn column_name(&self) -> &str {
        match self {
            FilterPredicate::Eq(name, _) => name,
            FilterPredicate::Gt(name, _) => name,
            FilterPredicate::Lt(name, _) => name,
            FilterPredicate::Ne(name, _) => name,
        }
    }

    pub fn evaluate(&self, row: &Row, schema: &Schema) -> bool {
        let field_idx = schema.fields.iter()
            .position(|f| f.name == self.column_name())
            .unwrap_or(usize::MAX);
        if field_idx == usize::MAX {
            return false;
        }
        let value = row.get(field_idx);
        match (self, value) {
            (FilterPredicate::Eq(_, expected), Some(v)) => v == expected,
            (FilterPredicate::Gt(_, expected), Some(Value::Int(v))) => {
                if let Value::Int(e) = expected { v > e } else { false }
            }
            (FilterPredicate::Lt(_, expected), Some(Value::Int(v))) => {
                if let Value::Int(e) = expected { v < e } else { false }
            }
            (FilterPredicate::Ne(_, expected), Some(v)) => v != expected,
            _ => false,
        }
    }
}

pub struct FilterExecutor {
    child: Box<dyn Executor>,
    predicate: FilterPredicate,
    is_open: bool,
}

impl FilterExecutor {
    pub fn new(child: Box<dyn Executor>, predicate: FilterPredicate) -> Self {
        FilterExecutor {
            child,
            predicate,
            is_open: false,
        }
    }
}

impl Executor for FilterExecutor {
    fn open(&mut self) -> Result<(), ExecError> {
        self.child.open()?;
        self.is_open = true;
        Ok(())
    }

    fn next(&mut self) -> Result<Option<RecordBatch>, ExecError> {
        if !self.is_open {
            return Err(ExecError::ExecuteError("Executor not open".to_string()));
        }
        while let Some(batch) = self.child.next()? {
            let filtered_rows: Vec<Row> = batch.rows.into_iter()
                .filter(|row| self.predicate.evaluate(row, &self.child.schema()))
                .collect();
            if !filtered_rows.is_empty() {
                return Ok(Some(RecordBatch::new(
                    self.child.schema().clone(),
                    filtered_rows,
                )));
            }
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<(), ExecError> {
        self.child.close()?;
        self.is_open = false;
        Ok(())
    }

    fn schema(&self) -> &Schema {
        self.child.schema()
    }
}