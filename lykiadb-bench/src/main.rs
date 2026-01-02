use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "lykiadb-bench")]
#[command(about = "Benchmark result manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Save current benchmark results with a name
    Save {
        /// Name for this snapshot
        name: String,
    },
    /// Compare two snapshots
    Compare {
        /// First snapshot name
        baseline: String,
        /// Second snapshot name
        current: String,
    },
    /// List saved snapshots
    List,
}

#[derive(Serialize, Deserialize)]
struct Snapshot {
    name: String,
    timestamp: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Estimates {
    mean: Option<PointEstimate>,
    median: Option<PointEstimate>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PointEstimate {
    point_estimate: f64,
}

fn bench_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn criterion_dir() -> PathBuf {
    bench_dir().join("target/criterion")
}

fn snapshots_dir() -> PathBuf {
    bench_dir().join("snapshots")
}

fn save_snapshot(name: &str) -> Result<()> {
    let src = criterion_dir();
    let dest = snapshots_dir().join(name);

    if !src.exists() {
        anyhow::bail!("No criterion results found. Run benchmarks first with: cargo bench -p lykiadb-bench");
    }

    fs::create_dir_all(&dest)?;
    copy_dir_all(&src, &dest)?;

    let meta = Snapshot {
        name: name.to_string(),
        timestamp: chrono::Local::now().to_rfc3339(),
    };
    fs::write(
        dest.join("meta.json"),
        serde_json::to_string_pretty(&meta)?,
    )?;

    println!("✓ Saved snapshot: {}", name);
    Ok(())
}

fn compare_snapshots(baseline: &str, current: &str) -> Result<()> {
    let baseline_path = snapshots_dir().join(baseline);
    if !baseline_path.exists() {
        anyhow::bail!("Snapshot '{}' not found", baseline);
    }

    let current_path = snapshots_dir().join(current);
    if !current_path.exists() {
        anyhow::bail!("Snapshot '{}' not found", current);
    }

    // Set up comparison by copying baseline and current data to criterion's expected locations
    let crit_dir = criterion_dir();
    let _ = fs::remove_dir_all(&crit_dir);
    fs::create_dir_all(&crit_dir)?;
    
    // Copy the main report directory from current snapshot
    let current_report = current_path.join("report");
    if current_report.exists() {
        let crit_report = crit_dir.join("report");
        copy_dir_all(&current_report, &crit_report)?;
    }
    
    // Find all benchmark groups and set up comparison
    for entry in fs::read_dir(&current_path)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        
        // Skip non-benchmark directories
        if name_str == "report" || name_str == "meta.json" || name_str.starts_with('.') {
            continue;
        }
        
        let baseline_group = baseline_path.join(&name);
        let current_group = current_path.join(&name);
        
        if !baseline_group.exists() || !current_group.exists() {
            continue;
        }
        
        // Copy current data to criterion dir
        let crit_group = crit_dir.join(&name);
        copy_dir_all(&current_group, &crit_group)?;
        
        // Copy baseline data as "base"
        let base_dest = crit_group.join("base");
        copy_dir_all(&baseline_group, &base_dest)?;
    }

    println!("✓ Comparison prepared: '{}' vs '{}'", baseline, current);
    println!("\nView results:");
    println!("  open lykiadb-bench/target/criterion/report/index.html");
    
    Ok(())
}

fn list_snapshots() -> Result<()> {
    let dir = snapshots_dir();
    if !dir.exists() {
        println!("No snapshots saved yet");
        return Ok(());
    }

    let mut entries: Vec<_> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let name = entry.file_name();
        let meta_path = entry.path().join("meta.json");
        if let Ok(meta) = fs::read_to_string(meta_path) {
            if let Ok(snapshot) = serde_json::from_str::<Snapshot>(&meta) {
                println!("{} ({})", name.to_string_lossy(), snapshot.timestamp);
                continue;
            }
        }
        println!("{}", name.to_string_lossy());
    }

    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Save { name } => save_snapshot(&name),
        Commands::Compare { baseline, current } => compare_snapshots(&baseline, &current),
        Commands::List => list_snapshots(),
    }
}
