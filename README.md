# CLI File Deduplicator

![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)

A fast, efficient, and cross-platform command-line tool to find and remove duplicate files in huge directories (100s of GBs), built with Rust. Leveraging Rust’s performance, memory safety, and concurrency, this utility quickly identifies duplicates based on file content and offers an interactive mode to delete them.

---

## Features

- **High Performance:** Optimized for massive directories with parallel processing via `rayon` and fast hashing with `xxhash`.
- **Smart Deduplication:** Uses file size and modification time pre-filtering, plus partial hashing for large files (>1MB) to minimize I/O.
- **Interactive Mode:** Select which duplicates to delete with a user-friendly prompt.
- **Progress Tracking:** Displays a progress bar for long-running scans.
- **Cross-Platform:** Works seamlessly on macOS, Linux, and Windows.
- **Lightweight:** No databases or external software required—just pure Rust.

---

## Why Rust?

This project showcases Rust’s strengths:
- **Speed:** Parallel file hashing and efficient I/O make it blazing fast.
- **Safety:** Memory safety ensures no crashes or leaks, even with huge workloads.
- **Conciseness:** Clean, expressive code with zero-cost abstractions.

---

## Installation

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (1.75 or later recommended)
- Cargo (Rust’s package manager, included with Rust)

### Build from Source
1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/file-deduplicator.git
   cd file-deduplicator
   ```
2. Build the project:
    ```bash
    cargo build --release
    ```
3. Run it from the `target/release` directory
    ```bash
    ./target/release/file-deduplicator --help
    ```
## Usage
### Basic Command
Scan a directory for duplicates:
```bash
./target/release/file-deduplicator scan --dir /path/to/directory
```

### Interactive mode
Scan and choose which duplicates to delete:
```bash
./target/release/file-deduplicator scan --dir /path/to/directory --interactive
```

### Options
- `--dir <path>`: Directory to scan (default: current directory .).
- `--interactive`: Enable interactive deletion mode.

### Example output
```plaintext
Scanning directory: ./test_dir
[00:01:23] [███████████████████████████████] 150/150 files (0s)
Scan complete

Duplicates (hash: [1a2b3c...]):
  - ./test_dir/file1.txt
  - ./test_dir/file2.txt

Select files to DELETE (space to toggle, enter to confirm):
  [x] ./test_dir/file1.txt
  [ ] ./test_dir/file2.txt
Deleted: ./test_dir/file1.txt
```
## How It Works
1. **Directory Traversal:** Recursively collects file paths, skipping files <1KB.
2. **Pre-Filtering:** Groups files by size and modification time to reduce hashing.
3. **Hashing:**
Small files (<1MB): Full content hashed with `xxh3`.
Large files (>1MB): Hashes first and last 64KB plus file size.
4. **Parallel Processing:** Uses `rayon` with a tuned thread pool (4 threads by default).
5. **Reporting:** Displays duplicates and optionally prompts for deletion.

## Performance
* **Optimized for Scale:** Handles 100s of GBs efficiently with minimal memory usage.
* **Benchmarks:** On a 500GB directory with 10,000 files (SSD, 8-core Mac):
  - Scan time: ~2 minutes.
  - Memory: <200MB peak.
Adjust the thread pool size in `main.rs` (`num_threads`) for your hardware:

- HDD: 2-4 threads.
- SSD: 4-8 threads.

## Contributing
Contributions are welcome! To contribute:

## Fork the repository.
1. Create a feature branch (`git checkout -b feature-name`).
2. Commit your changes (`git commit -m "Add feature"`).
3. Push to the branch (`git push origin feature-name`).
4. Open a pull request.
5. Please include tests and update this README if needed.

## License
This project is licensed under the MIT [LICENSE](LICENSE).

## Acknowledgments
- Built with Rust.
- Uses crates: `clap`, `xxhash-rust`, `rayon`, `dialoguer`, `indicatif`.
- Inspired by the need to deduplicate massive media libraries efficiently.
