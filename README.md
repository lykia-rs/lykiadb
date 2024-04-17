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
    <img alt="LykiaDB logo" height="200" src="https://vcankeklik.com/assets/img/logo.svg?v=051223">
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
- [x] Data manipulation language ("SELECT", "INSERT", "UPDATE", "DELETE")
- [x] Event loop, client-server communication
- [ ] Data definition language ("CREATE COLLECTION", etc.) (in progress)
- [ ] Query binding and planning (in progress)
- [ ] LSM storage engine (based on [mini-lsm](https://github.com/lykia-rs/mini-lsm)) (in progress)
- [ ] MVCC for transaction management (based on [mini-lsm](https://github.com/lykia-rs/mini-lsm))
- [ ] B-Tree implementation for indexing
- [ ] Plan optimization
- [ ] Basic replication with Raft

## Getting Started
To use Lykia, you can download the latest release from the GitHub releases page.

Run the server:

```shell
$ cargo run --release --bin lykiadb-server
```
Run the client:

```shell 
$ cargo run --release --bin lykiadb-shell cli/examples/fib.ly
```

Client transmits the fib.ly in an infinite loop.

## License
Lykia is licensed under the Apache License, Version 2.0. See LICENSE for the full license text.