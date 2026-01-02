# Benchmark Management

## Usage

```bash
# Run benchmarks
cargo bench -p lykiadb-bench

# Save current results
cargo run -p lykiadb-bench -r -- save <name>

# List saved snapshots
cargo run -p lykiadb-bench -r -- list

# Generate comparison report (baseline vs latest)
cargo run -p lykiadb-bench -r -- report <baseline>

# Compare two snapshots
cargo run -p lykiadb-bench -r -- report <baseline> --current <name>
```

## Example Workflow

```bash
# Initial baseline
cargo bench -p lykiadb-bench
cargo run -p lykiadb-bench -r -- save v1.0

# After changes
cargo bench -p lykiadb-bench
cargo run -p lykiadb-bench -r -- report v1.0

# Save new baseline
cargo run -p lykiadb-bench -r -- save v1.1

# Compare two snapshots
cargo run -p lykiadb-bench -r -- report v1.0 --current v1.1
```

Reports: `lykiadb-bench/reports/<comparison>/index.html`  
Snapshots: `lykiadb-bench/snapshots/<name>`
