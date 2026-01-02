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
    /// Save current benchmark results
    Save {
        /// Name for this snapshot
        name: String,
    },
    /// Generate HTML report comparing saved results
    Report {
        /// Baseline snapshot name
        baseline: String,
        /// Current snapshot name (optional, uses latest criterion results if omitted)
        #[arg(short, long)]
        current: Option<String>,
    },
    /// List saved snapshots
    List,
}

#[derive(Serialize, Deserialize)]
struct Snapshot {
    name: String,
    timestamp: String,
}

fn bench_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn criterion_dir() -> PathBuf {
    bench_dir().parent().unwrap().join("target/criterion")
}

fn snapshots_dir() -> PathBuf {
    bench_dir().join("snapshots")
}

fn report_dir() -> PathBuf {
    bench_dir().join("reports")
}

fn save_snapshot(name: &str) -> Result<()> {
    let src = criterion_dir();
    let dest = snapshots_dir().join(name);

    if !src.exists() {
        anyhow::bail!("No criterion results found. Run benchmarks first with: cargo bench");
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

    println!("Saved snapshot: {}", name);
    Ok(())
}

fn generate_report(baseline: &str, current: Option<&str>) -> Result<()> {
    let baseline_path = snapshots_dir().join(baseline);
    if !baseline_path.exists() {
        anyhow::bail!("Baseline snapshot '{}' not found", baseline);
    }

    let current_path = if let Some(c) = current {
        let p = snapshots_dir().join(c);
        if !p.exists() {
            anyhow::bail!("Current snapshot '{}' not found", c);
        }
        p
    } else {
        criterion_dir()
    };

    if !current_path.exists() {
        anyhow::bail!("Current results not found");
    }

    let report_dir = report_dir();
    fs::create_dir_all(&report_dir)?;

    // Copy baseline to report/base
    let base_dest = report_dir.join("base");
    fs::create_dir_all(&base_dest)?;
    copy_dir_all(&baseline_path, &base_dest)?;

    // Copy current to report/change
    let change_dest = report_dir.join("change");
    fs::create_dir_all(&change_dest)?;
    copy_dir_all(&current_path, &change_dest)?;

    // Symlink for criterion compatibility
    let _ = fs::remove_file(report_dir.join("both"));
    #[cfg(unix)]
    std::os::unix::fs::symlink(".", report_dir.join("both"))?;

    // Generate index
    let current_name = current.unwrap_or("latest");
    let index = format!(
        r#"<!DOCTYPE html>
<html>
<head><title>Benchmark Comparison</title></head>
<body>
<h1>{} vs {}</h1>
<p>Browse base and change directories for detailed reports.</p>
<ul>
<li><a href="base/report/index.html">Baseline: {}</a></li>
<li><a href="change/report/index.html">Current: {}</a></li>
</ul>
</body>
</html>"#,
        baseline, current_name, baseline, current_name
    );

    fs::write(report_dir.join("index.html"), index)?;

    println!("Report generated: {}", report_dir.join("index.html").display());
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
        Commands::Report { baseline, current } => generate_report(&baseline, current.as_deref()),
        Commands::List => list_snapshots(),
    }
}
