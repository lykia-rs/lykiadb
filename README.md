# ReverieDB
Reverie is a vector database written in Rust. 
It provides a convenient scripting language with SQL-like syntax supports both in-memory and in-disk storage. It is distributed by default, also handles concurrency with MVCC (Multi-Version Concurrency Control) mechanism.

## Features
- Written in Rust for performance, reliability, and safety.
- A handy scripting language that is easy to use and provides a rich set of functions.
- SQL-like syntax that is familiar to database developers.
- Distributed using the Raft consensus algorithm, which ensures fault-tolerance and consistency.
- Both in-memory and in-disk storage support, allowing for faster read/write performance and durability.
- MVCC concurrency control mechanism, which enables multiple transactions to access the same data simultaneously without interfering with each other.

## Getting Started
To use Reverie, you can download the latest release from the GitHub releases page. The database server can be started with the following command:

```shell
$ reverie-server
```
To interact with the database, you can use the Reverie command-line interface (CLI), which can be started with the following command:

```shell 
$ reverie-cli
```
Alternatively, you can use the Reverie API to interact with the database programmatically. The API is well-documented and easy to use.

## Scripting Language
Reverie's scripting language provides a set of built-in functions for working with the database, as well as the ability to define custom functions. The language is designed to be easy to use, with a syntax that is similar to other popular scripting languages.

## SQL-like Syntax
Reverie's SQL-like syntax allows database developers to use familiar syntax and commands to interact with the database. The syntax includes support for common SQL commands such as SELECT, INSERT, UPDATE, DELETE, and more.

## Distributed by design
Reverie is built using the Raft consensus algorithm, which ensures that the database is fault-tolerant and consistent. This means that even in the event of a node failure or network partition, the database can continue to operate and provide consistent data.

## In-memory and in-disk storage support
Reverie provides both in-memory and in-disk storage support, allowing for faster read/write performance and durability. In-memory storage is great for applications that require low-latency access to data, while in-disk storage provides durability and persistence.

## Concurrency Control
Reverie's MVCC concurrency control mechanism allows multiple transactions to access the same data simultaneously without interfering with each other. This means that applications can scale to handle large amounts of concurrent traffic without sacrificing performance or data consistency.

## Contributing
Reverie is an open-source project, and contributions are welcome! If you would like to contribute to the project, please read our contributing guidelines.

## License
Reverie is licensed under the Apache License, Version 2.0. See LICENSE for the full license text.