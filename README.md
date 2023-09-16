# lykia-db
Lykia is a toy document database basically written for educational purposes. It is planned to be a modest mixture of popular kids in town (e.g. SurrealDB).

## Overview
- Written in Rust
- A weird scripting and query language, combination of PHP and SQL. Built based on the language "Lox" which is explained in the famous book, Crafting Interpreters.
- A subset of JSON data types in both scripting language itself and storage
- In-disk and in-memory storage
- ACID compliance
- Replication

## Roadmap

- [x] Core scripting language
- [ ] A minimal standard library (in progress)
- [ ] SQL parsing (in progress)
- [ ] Query planning
- [ ] Async runtime/event loop
- [ ] In-memory storage engine
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
$ cargo run examples/fib.ly
```

## License
Lykia is licensed under the Apache License, Version 2.0. See LICENSE for the full license text.