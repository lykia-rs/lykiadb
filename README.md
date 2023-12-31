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


Lykia is a toy document database basically written for educational purposes. It is planned to be a modest mixture of popular kids in town (e.g. SurrealDB).

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
- [ ] SQL parsing
    - [x] "SELECT" expressions (the most complex part of the SQL syntax)
    - [ ] "INSERT" statements (in progress)
    - [ ] "UPDATE" statements (in progress)
    - [ ] "DELETE" statements (in progress)
- [ ] Query planning (in progress)
- [ ] Plan optimization
- [ ] Async runtime/event loop
- [ ] In-memory storage engine
- [ ] Type checker
- [ ] Persistent storage engine (Bitcask)
- [ ] Transaction management with MVCC
- [ ] B-Tree implementation for indexes
- [ ] Basic replication with Raft

## Getting Started
To use Lykia, you can download the latest release from the GitHub releases page.

REPL:

```shell
$ cargo run
```
Alternatively, you can run a Lykia script by passing its name as the first argument.

```shell 
$ cargo run server/examples/fib.ly
```

## License
Lykia is licensed under the Apache License, Version 2.0. See LICENSE for the full license text.