use crate::sql::ast::Column;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

// #[derive(Debug, Clone)]
// pub struct Column {
//     name: String,
//     data_type: DataType,
// }

#[derive(Debug, Clone)]
pub struct Schema {
    pub columns: Vec<Column>,
}

impl Schema {
    pub fn get_col_idx(&self, col_name: &str) -> Option<usize> {
        self.columns.iter().position(|col| col.name == col_name)
    }
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub schema: Schema,
}

#[derive(Debug, Default)]
pub struct Catalog {
    tables: HashMap<String, TableInfo>,
    next_table_id: usize,
}

impl Catalog {

    pub fn create_table(&mut self, name: String, columns: Vec<Column>) -> Result<(), String> {
        if self.tables.contains_key(&name) {
            return Err(format!("Table '{}' already exists", name));
        }
        let schema = Schema { columns };
        let table_info = TableInfo {
            name: name.clone(),
            schema,
        };
        self.tables.insert(name, table_info);
        self.next_table_id += 1;
        Ok(())
    }

    pub fn get_table(&self, name: &str) -> Option<&TableInfo> {
        self.tables.get(name)
    }
}

pub type CatalogRef = Arc<Mutex<Catalog>>;
