# lykia-db
Lykia is a toy document database basically written for educational purposes. It is planned to be a modest mixture of popular kids in town (e.g. SurrealDB).

## Overview
- Written in Rust
- A weird scripting and query language, combination of PHP and SQL. Built based on the language "Lox" which is explained in the famous book, Crafting Interpreters.
- A subset of JSON data types in both scripting language itself and storage
- In-disk and in-memory storage
- ACID compliance
- Replication
- A handy scripting language that is easy to use and provides a rich set of functions. 

## Roadmap

- [x] Core scripting language
- [-] A minimal standard library
- [-] SQL parsing
- [ ] Query planning
- [ ] Event loop for communication
- [ ] In-memory storage engine
- [ ] Bitcask storage engine
- [ ] Transaction management with MVCC
- [ ] B-Tree implementation for indexes
- [ ] Basic replication with Raft

## Getting Started
To use Lykia, you can download the latest release from the GitHub releases page.

REPL:

```shell
$ lykia
```
Alternatively, you can run a Lykia script by passing its name as the first argument.

```shell 
$ lykia hello.ly
```

## License
Lykia is licensed under the Apache License, Version 2.0. See LICENSE for the full license text.