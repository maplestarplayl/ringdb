use bincode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
pub enum DataType {
    Int,
    Varchar,
}
#[derive(Debug, Clone, Encode, Decode)]
pub enum Value {
    Integer(i64),
    String(String),
}
#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
}

#[derive(Debug)]
pub enum Statement {
    CreateTable {
        table_name: String,
        columns: Vec<Column>,
    },
    Insert {
        table_name: String,
        values: Vec<Value>,
    },
    Select {
        table_name: String,
        columns: Vec<String>,
    },
}
