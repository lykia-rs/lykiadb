# Benchmark Management

## Usage

```bash
# Run benchmarks
cargo bench -p lykiadb-bench

# Save results with a name
cargo run -p lykiadb-bench -r -- save <name>

# List saved snapshots
cargo run -p lykiadb-bench -r -- list

# Compare two snapshots
cargo run -p lykiadb-bench -r -- compare <baseline> <current>
```

## Example Workflow

```bash
# Run initial benchmarks and save
cargo bench -p lykiadb-bench
cargo run -p lykiadb-bench -r -- save v1.0

# After code changes, run benchmarks again and save
cargo bench -p lykiadb-bench
cargo run -p lykiadb-bench -r -- save v1.1

# Compare the two snapshots
cargo run -p lykiadb-bench -r -- compare v1.0 v1.1

# View HTML report
open lykiadb-bench/target/criterion/report/index.html
```