use std::{net::TcpListener, sync::Arc};

use bytes::{BufMut, BytesMut};
use crossbeam::channel;
use monoio::{
    io::{AsyncReadRentExt, AsyncWriteRentExt},
    net::TcpStream,
};
use ringdb::{Database, storage::disk::DiskManager};

const LEN_BYTES: usize = 4;

fn main() {
    println!("--- ringDB Server ---");

    let db = {
        let mut rt = monoio::RuntimeBuilder::<monoio::FusionDriver>::new()
            .build()
            .unwrap();
        rt.block_on(async { Arc::new(Database::new("database.db".to_string(), 64).await.unwrap()) })
    };

    let core_ids = core_affinity::get_core_ids().unwrap();
    let num_cores = core_ids.len();
    println!("Detected {} CPU cores.", num_cores);

    let (tx, rx) = channel::unbounded();

    let mut worker_threads = Vec::new();
    for (i, core_id) in core_ids.into_iter().enumerate() {
        let rx = rx.clone();
        let db = db.clone();

        let handle = std::thread::spawn(move || {
            if !core_affinity::set_for_current(core_id) {
                eprintln!("Failed to set core affinity for worker {}", i);
            } else {
                println!("Worker {} pinned to core {:?}", i, core_id.id);
            }

            let mut rt = monoio::RuntimeBuilder::<monoio::FusionDriver>::new()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                while let Ok(stream) = rx.recv() {
                    let stream = monoio::net::TcpStream::from_std(stream).unwrap();
                    let db = db.clone();
                    monoio::spawn(handle_connection(stream, db));
                }
            })
        });
        worker_threads.push(handle);
    }

    let listener_thread = std::thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:5432").unwrap();
        println!("Server is listening on: 127.0.0.1:5432");

        loop {
            match listener.accept() {
                Ok((stream, addr)) => {
                    println!("Accepted new connection from {}", addr);
                    // Send the connection to the worker thread
                    if tx.send(stream).is_err() {
                        eprintln!(
                            "Failed to distribute connection to worker thread, channel closed."
                        );
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }
    });

    listener_thread.join().unwrap();
    for handle in worker_threads {
        handle.join().unwrap();
    }
}

async fn handle_connection(mut stream: TcpStream, db: Arc<Database>) {
    let disk_manager = Arc::new(DiskManager::new("database.db").await.unwrap());
    let mut buffer = BytesMut::with_capacity(1024);
    loop {
        let disk_manager = disk_manager.clone();
        let len_buffer = vec![0u8; LEN_BYTES];
        let (res, len_buffer) = stream.read_exact(len_buffer).await;
        if res.is_err() {
            eprintln!("Failed to read length: {}", res.err().unwrap());
            break;
        }

        let len = u32::from_be_bytes(len_buffer.try_into().unwrap()) as usize;
        buffer.resize(len, 0);

        let (res, res_buffer) = stream.read_exact(buffer).await;
        if res.is_err() {
            eprintln!("Failed to read SQL: {}", res.err().unwrap());
            break;
        }

        let sql = match String::from_utf8(res_buffer.to_vec()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to read SQL, invalid UTF-8 sequence: {}", e);
                break;
            }
        };
        println!("Received SQL: {}", sql);

        let result = db.run_statement(&sql, disk_manager).await;

        let encoded_result = bincode::encode_to_vec(result, bincode::config::standard()).unwrap();
        let result_len = (encoded_result.len() as u32).to_be_bytes();

        let mut response = BytesMut::new();
        response.put_slice(&result_len);
        response.put_slice(&encoded_result);

        if let (Err(e), _) = stream.write_all(response).await {
            eprintln!("Failed to write response: {}", e);
            return;
        }

        buffer = res_buffer;
    }
}
