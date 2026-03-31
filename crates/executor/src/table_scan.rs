use crate::*;

pub struct TableScanExecutor {
    table_name: String,
    schema: Schema,
    data: Vec<Row>,
    current_pos: usize,
    is_open: bool,
}

impl TableScanExecutor {
    pub fn new(table_name: &str, schema: Schema, data: Vec<Row>) -> Self {
        TableScanExecutor {
            table_name: table_name.to_string(),
            schema,
            data,
            current_pos: 0,
            is_open: false,
        }
    }
}

impl Executor for TableScanExecutor {
    fn open(&mut self) -> Result<(), ExecError> {
        self.is_open = true;
        self.current_pos = 0;
        Ok(())
    }

    fn next(&mut self) -> Result<Option<RecordBatch>, ExecError> {
        if !self.is_open {
            return Err(ExecError::ExecuteError("Executor not open".to_string()));
        }
        if self.current_pos >= self.data.len() {
            return Ok(None);
        }
        let row = self.data[self.current_pos].clone();
        self.current_pos += 1;
        Ok(Some(RecordBatch::new(self.schema.clone(), vec![row])))
    }

    fn close(&mut self) -> Result<(), ExecError> {
        self.is_open = false;
        Ok(())
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }
}