# RingDB: A High-Performance Learning Database Built with `io_uring` and Rust

**RingDB** is a relational database prototype built from scratch for learning and research purposes. It explores the application of modern systems programming techniques in core database components, with a special focus on Linux's cutting-edge asynchronous I/O interface, **`io_uring`**.

This project was built through a series of progressive stages, constructing a complete database system from low-level I/O up to high-level SQL interaction. It culminates in an OLTP server architecture that adopts the **Thread-per-Core** model and supports network access via TCP.

## ✨ Features

  * **Blazing-Fast I/O Layer:**

      * Fully based on the `monoio` runtime for deep **`io_uring`** integration, enabling truly asynchronous I/O.
      * Implements a sequential file scan executor with prefetch capabilities to optimize read throughput.

  * **Modern Concurrency Architecture:**

      * Employs a **Thread-per-Core** server model, spawning an independent worker thread and `monoio` runtime for each CPU core to achieve true parallelism.
      * Efficiently dispatches client connections using a main listener thread + worker thread pattern.
      * Safely shares core state (Buffer Pool, System Catalog) across threads using `Arc` and internal locks (`Mutex`/`RwLock`).

  * **Complete Core Components:**

      * **Buffer Pool Manager:** A high-efficiency in-memory cache implementing the **Clock-Sweep** replacement policy.
      * **Persistent Catalog:** Serializes and stores schema metadata (table definitions, etc.) in the database file, giving the database a "memory."
      * **SQL Parser:** A hand-written recursive descent parser supporting basic SQL statements like `CREATE TABLE`, `INSERT`, and `SELECT`, which transforms SQL text into an Abstract Syntax Tree (AST).
      * **Query Executor:** Based on the classic Volcano/Iterator model to execute parsed ASTs and manipulate data.
      * **Client/Server Model:** Supports remote access via a TCP network protocol.

## 🏗️ Project Structure

The project is organized as a Rust **library** core with two **binaries** (`server` and `client`).

```
.
├── Cargo.toml
└── src/
    ├── bin/
    │   ├── client.rs      # A simple command-line client to connect to the server
    │   └── server.rs      # The main database server program (listener/worker model)
    ├── storage_layer.rs   # [Consolidated File] Core Storage: DiskManager, BufferPool, etc.
    ├── catalog.rs       # System catalog for managing table metadata
    ├── db.rs            # Top-level database instance, encapsulating core APIs
    ├── executor/        # Query executor module
    │   ├── mod.rs
    │   ├── create_table.rs
    │   ├── insert.rs
    │   └── seq_scan.rs
    ├── parser/          # SQL parser module
    │   ├── mod.rs
    │   ├── ast.rs
    │   ├── lexer.rs
    │   ├── parser.rs
    │   └── token.rs
    └── lib.rs           # Declares all modules, defines the library's public interface
```

## 🚀 Getting Started

**Prerequisites:**

  * Rust toolchain installed.
  * A Linux environment (as `io_uring` and `monoio` are Linux-specific).

**1. Compile the project:**

```bash
cargo build --release
```

**2. Run the database server:**
*In one terminal window:*

```bash
./target/release/server
```

You should see the server start up and begin listening on `127.0.0.1:5432`.

**3. Run the client and interact:**
*In a **second** terminal window:*

```bash
./target/release/client
```

You will be greeted with a `ring-db>` prompt. You can now enter SQL commands to interact with your database\!

```sql
ring-db> CREATE TABLE users (id INT, name VARCHAR);
Message("Table 'users' created.")

ring-db> INSERT INTO users VALUES (1, 'Alice');
Message("1 row inserted.")

ring-db> SELECT id, name FROM users;
Data([Tuple { values: [Integer(1), String("Alice")] }])

ring-db> .exit
```

## 🗺️ Roadmap

This project has laid a solid foundation for a powerful database system. The following is a roadmap of features and improvements that can be explored to make it more complete and robust.

  - [ ] **Indexing:**

      - [ ] Implement a disk-friendly **B+ Tree** index structure to accelerate `WHERE` clause lookups.

  - [ ] **Concurrency Control:**

      - [ ] Implement **Transactions** (`BEGIN`, `COMMIT`, `ROLLBACK`).
      - [ ] Implement a **Lock Manager** (based on 2PL) or a more advanced **MVCC** (Multi-Version Concurrency Control) protocol.

  - [ ] **Recovery:**

      - [ ] Implement **Write-Ahead Logging (WAL)** to ensure atomicity and durability in the face of crashes.

  - [ ] **Query Optimizer:**

      - [ ] Develop a cost-based query optimizer to choose the most efficient execution plan (e.g., choosing between an index scan and a table scan).

  - [ ] **Expanded SQL Support:**

      - [ ] Support for the `WHERE` clause (requires a `FilterExecutor`).
      - [ ] Support for `UPDATE` and `DELETE` statements.
      - [ ] Support for `JOIN` operations (`HashJoinExecutor`, `NestedLoopJoinExecutor`).
      - [ ] Support for aggregate functions (`GROUP BY`) and sorting (`ORDER BY`).

  - [ ] **Storage Layer Enhancements:**

      - [ ] Support for **multi-page tables** that can grow beyond a single page.
      - [ ] Implement more granular **free space management** within pages.
      - [ ] Implement a persistent, table-based system catalog.
  - [ ] **IO Enhancements:**

      - [ ] Replace `Vec<u8>` with a custom **aligned buffer** type to ensure proper memory alignment for `O_DIRECT` I/O.
      - [ ] Implement a more sophisticated **prefetching strategy** in the sequential scan executor.
  - [ ] **Networking Layer Optimizations:**

      - [ ] Use **`SO_REUSEPORT`** on the server listener to eliminate the single-listener bottleneck and allow all worker threads to accept connections directly.