use crate::*;

pub struct ProjectionExecutor {
    child: Box<dyn Executor>,
    columns: Vec<String>,
    output_schema: Schema,
    column_indices: Vec<usize>,
    is_open: bool,
}

impl ProjectionExecutor {
    pub fn new(child: Box<dyn Executor>, columns: Vec<String>) -> Result<Self, ExecError> {
        let input_schema = child.schema();
        let mut column_indices = Vec::new();
        let mut output_fields = Vec::new();

        for col in &columns {
            if col == "*" {
                for (idx, field) in input_schema.fields.iter().enumerate() {
                    column_indices.push(idx);
                    output_fields.push(field.clone());
                }
                break;
            } else {
                let idx = input_schema.fields.iter()
                    .position(|f| f.name == *col)
                    .ok_or_else(|| ExecError::TypeError(format!("Column '{}' not found", col)))?;
                column_indices.push(idx);
                output_fields.push(input_schema.fields[idx].clone());
            }
        }

        Ok(ProjectionExecutor {
            child,
            columns,
            output_schema: Schema::new(output_fields),
            column_indices,
            is_open: false,
        })
    }
}

impl Executor for ProjectionExecutor {
    fn open(&mut self) -> Result<(), ExecError> {
        self.child.open()?;
        self.is_open = true;
        Ok(())
    }

    fn next(&mut self) -> Result<Option<RecordBatch>, ExecError> {
        if !self.is_open {
            return Err(ExecError::ExecuteError("Executor not open".to_string()));
        }
        match self.child.next()? {
            Some(batch) => {
                let projected_rows: Vec<Row> = batch.rows.into_iter()
                    .map(|row| {
                        let values: Vec<Value> = self.column_indices.iter()
                            .filter_map(|&idx| row.get(idx).cloned())
                            .collect();
                        Row::new(values)
                    })
                    .collect();
                Ok(Some(RecordBatch::new(self.output_schema.clone(), projected_rows)))
            }
            None => Ok(None),
        }
    }

    fn close(&mut self) -> Result<(), ExecError> {
        self.child.close()?;
        self.is_open = false;
        Ok(())
    }

    fn schema(&self) -> &Schema {
        &self.output_schema
    }
}