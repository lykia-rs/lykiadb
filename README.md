# λykiaDB
<div style="display: flex;">
<div>

[![CI](https://github.com/lykia-rs/lykiadb/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/lykia-rs/lykiadb/actions/workflows/ci.yml)

</div>
<div>

[![codecov](https://codecov.io/gh/lykia-rs/lykiadb/graph/badge.svg?token=DGIK7BE3K1)](https://codecov.io/gh/lykia-rs/lykiadb)

</div>
</div>

<p align="center">
    <img alt="LykiaDB logo" height="200" src="https://raw.githubusercontent.com/lykia-rs/lykiadb/refs/heads/main/assets/logo.svg">
</p>


Lykia is a document database management system built for educational purposes. The famous book, Crafting Interpreters, was the main source of inspiration for the project. It turned into a database, though.

## Overview
- 100% safe Rust, #BlazinglyFast
- A weird scripting and query language, combination of JavaScript and SQL. The language is a separate module thus can be used without the database.
- A subset of JSON data types in both scripting language itself and storage
- In-disk and in-memory storage
- ACID compliance
- Replication

## Primary goals

- [x] Core scripting language + DML/DDL SQL
- [x] A minimal standard library
- [x] Modular architecture for core and surrounding components
- [ ] Query planning and binding (in progress)
- [ ] Persistent storage engine
- [ ] Multi-version concurrency control
- [ ] Query optimization

## Secondary goals

- [x] Playground app

## Advanced 

- [ ] Replication with Raft
- [ ] JIT execution for expression evaluation

## Getting Started
To use Lykia, you can download the latest release from the GitHub releases page.

Run the server:

```shell
$ cargo run --release --bin lykiadb-server
```
Run the shell:

```shell 
$ cargo run --release --bin lykiadb-shell lykiadb-shell/examples/fib.ly
```

For playground, please visit [lykia-rs/playground](https://github.com/lykia-rs/playground)

## License
Lykia is licensed under the Apache License, Version 2.0. See LICENSE for the full license text.