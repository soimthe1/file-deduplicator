use std::error::Error;
use std::fmt;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use xxhash_rust::xxh3::Xxh3;
use std::collections::HashMap;
use rayon::prelude::*;
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};

// Custom error type
#[derive(Debug)]
enum AppError {
    Io(std::io::Error),
    Dialoguer(dialoguer::Error),
    ThreadPool(rayon::ThreadPoolBuildError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(err) => write!(f, "I/O error: {}", err),
            AppError::Dialoguer(err) => write!(f, "Dialoguer error: {}", err),
            AppError::ThreadPool(err) => write!(f, "Thread pool error: {}", err),
        }
    }
}

impl Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<dialoguer::Error> for AppError {
    fn from(err: dialoguer::Error) -> Self {
        AppError::Dialoguer(err)
    }
}

impl From<rayon::ThreadPoolBuildError> for AppError {
    fn from(err: rayon::ThreadPoolBuildError) -> Self {
        AppError::ThreadPool(err)
    }
}

// CLI structure
#[derive(Parser)]
#[command(name = "file-deduplicator", version = "1.0", about = "A fast CLI tool to find and remove duplicate files")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Scan {
        #[arg(short, long, default_value = ".")]
        dir: String,
        #[arg(short, long)]
        interactive: bool,
    },
}

// File hashing with partial hashing for large files
const SAMPLE_SIZE: usize = 65536; // 64KB
const LARGE_FILE_THRESHOLD: u64 = 1_048_576; // 1MB

fn hash_file(path: &PathBuf) -> Result<Vec<u8>, AppError> {
    let metadata = fs::metadata(path)?;
    let size = metadata.len();
    let mut file = fs::File::open(path)?;
    let mut hasher = Xxh3::new();

    if size <= LARGE_FILE_THRESHOLD {
        let mut buffer = [0; 16384]; // 16KB buffer for small files
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
    } else {
        let mut head = vec![0; SAMPLE_SIZE];
        file.read_exact(&mut head)?;
        hasher.update(&head);

        file.seek(SeekFrom::End(-(SAMPLE_SIZE as i64).min(size as i64)))?;
        let mut tail = vec![0; SAMPLE_SIZE];
        file.read_exact(&mut tail)?;
        hasher.update(&tail);

        hasher.update(&size.to_le_bytes()); // Include size to reduce false positives
    }

    Ok(hasher.digest().to_le_bytes().to_vec())
}

// Directory scanning with fixed duplicate detection
fn scan_directory(dir: &str) -> Result<HashMap<Vec<u8>, Vec<PathBuf>>, AppError> {
    let mut paths = Vec::new();
    let mut stack = vec![PathBuf::from(dir)];

    // Collect all file paths
    while let Some(current_dir) = stack.pop() {
        for entry in fs::read_dir(current_dir)? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                let size = fs::metadata(&path)?.len();
                if size > 1024 { // Skip <1KB
                    paths.push(path);
                }
            }
        }
    }

    let pb = ProgressBar::new(paths.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} files ({eta})")
            .unwrap()
    );

    // Hash all files in parallel and collect into HashMap
    let duplicates: HashMap<Vec<u8>, Vec<PathBuf>> = paths
        .par_iter()
        .filter_map(|path| {
            match hash_file(path) {
                Ok(hash) => {
                    pb.inc(1);
                    Some((hash, path.clone()))
                }
                Err(e) => {
                    println!("Error hashing {}: {:?}", path.display(), e);
                    pb.inc(1);
                    None
                }
            }
        })
        .fold(
            || HashMap::new(),
            |mut acc, (hash, path)| {
                acc.entry(hash).or_insert_with(Vec::new).push(path);
                acc
            },
        )
        .reduce(
            || HashMap::new(),
            |mut acc, map| {
                for (hash, paths) in map {
                    acc.entry(hash).or_insert_with(Vec::new).extend(paths);
                }
                acc
            },
        );

    pb.finish_with_message("Scan complete");
    Ok(duplicates)
}

fn main() -> Result<(), AppError> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { dir, interactive } => {
            let duplicates = scan_directory(&dir)?;
            let duplicate_sets: Vec<_> = duplicates
                .into_iter()
                .filter(|(_, paths)| paths.len() > 1)
                .collect();

            if duplicate_sets.is_empty() {
                println!("No duplicates found.");
                return Ok(());
            }

            for (hash, paths) in &duplicate_sets {
                println!("Duplicates (hash: {:x?}):", hash);
                for path in paths {
                    println!("  - {}", path.display());
                }

                if interactive {
                    let choices: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
                    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
                        .with_prompt("Select files to DELETE (space to toggle, enter to confirm):")
                        .items(&choices)
                        .interact()?;

                    for idx in selections {
                        let path = &paths[idx];
                        fs::remove_file(path)?;
                        println!("Deleted: {}", path.display());
                    }
                }
            }
        }
    }

    Ok(())
}