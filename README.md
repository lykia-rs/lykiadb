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
    <img alt="LykiaDB logo" height="200" src="https://github.com/lykia-rs/lykiadb/tree/main/assets/logo.svg">
</p>


Lykia is a toy multi-model database basically written for educational purposes.

## Overview
- Written in Rust
- A weird scripting and query language, combination of JavaScript and SQL. Built based on the language "Lox" which is explained in the famous book, Crafting Interpreters.
- A subset of JSON data types in both scripting language itself and storage
- In-disk and in-memory storage
- ACID compliance
- Replication

## Roadmap

- [x] Core scripting language
- [x] A minimal standard library
- [x] DML/DDL SQL
- [x] Event loop, client-server communication
- [x] Minimal playground app
- [ ] Query binding and planning (in progress)
- [ ] Bitcask storage engine
- [ ] MVCC for transaction management (based on [mini-lsm](https://github.com/lykia-rs/mini-lsm))
- [ ] Plan optimization
-----------------------------------------
- [ ] LSM storage engine (based on [mini-lsm](https://github.com/lykia-rs/mini-lsm)) 
- [ ] Basic replication with Raft

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
Run the playground:

```shell 
$ cd lykiadb-playground
$ pnpm dev
```

## License
Lykia is licensed under the Apache License, Version 2.0. See LICENSE for the full license text.