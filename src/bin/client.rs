//! client.rs - A CLI client to connect to the DB server, send SQL commands, and display results.
use std::io::prelude::*;
use bytes::{BufMut, BytesMut};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::io::{self, Write};
use std::net::TcpStream;

use ringdb::executor::ExecutionResult;


const LEN_BYTES: usize = 4;

fn main() -> io::Result<()> {
    println!("--- ringDB CLIENT ---");
    println!("connect to 127.0.0.1:5432...");

    let mut stream = TcpStream::connect("127.0.0.1:5432")?;
    println!("Connected successfully! Please enter SQL statements or .exit to quit.");

    let mut rl = DefaultEditor::new().unwrap();

    loop {
        let readline = rl.readline("ringDB>> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                if line.trim() == ".exit" {
                    break;
                }

                // 1. Prepare the request (length + SQL)
                let sql_bytes = line.as_bytes();
                let len_bytes = (sql_bytes.len() as u32).to_be_bytes();
                let mut request = BytesMut::new();
                request.put_slice(&len_bytes);
                request.put_slice(sql_bytes);

                // 2. Send the request
                if let Err(e) = stream.write_all(&request) {
                    eprintln!("Send request failed: {}", e);
                    break;
                }

                // 3. Read the response length and content
                let mut len_buffer = [0u8; LEN_BYTES];
                if let Err(e) = stream.read_exact(&mut len_buffer) {
                    eprintln!("Read response length failed: {}", e);
                    break;
                }
                let result_len = u32::from_be_bytes(len_buffer) as usize;

                let mut result_buffer = vec![0u8; result_len];
                if let Err(e) = stream.read_exact(&mut result_buffer) {
                    eprintln!("Read response content failed: {}", e);
                    break;
                }

                // 4. Decode and display the result
                let (result, _): (Result<ExecutionResult, String>, _) =
                    bincode::decode_from_slice(&result_buffer, bincode::config::standard())
                        .unwrap();

                match result {
                    Ok(exec_result) => println!("{:?}", exec_result),
                    Err(e) => println!("Server error: {}", e),
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    println!("Disconnecting. Goodbye!");
    Ok(())
}