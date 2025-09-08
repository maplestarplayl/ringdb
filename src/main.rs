use std::sync::Arc;

use ringdb::{storage::disk::DiskManager, Database};
use rustyline::{DefaultEditor, error::ReadlineError};

#[monoio::main]
async fn main() {
    let mut r1 = DefaultEditor::new().unwrap();
    let db = Database::new("database.db".into(), 10).await.unwrap();
    let disk_manager = Arc::new(DiskManager::new("database.db").await.unwrap());
    loop {
        let readline = r1.readline("ringdb>> ");
        match readline {
            Ok(line) => {
                r1.add_history_entry(&line).unwrap();
                if line.trim() == "exit" {
                    println!("Exiting...");
                    break;
                }

                match db.run_statement(&line, disk_manager.clone()).await {
                    Ok(res) => println!("{:?}", res),
                    Err(e) => println!("Error executing statement: {:?}", e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Exiting...");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
