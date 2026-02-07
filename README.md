# gpt-toolbox
[![crates.io](https://img.shields.io/crates/v/gpt-toolbox.svg)](https://crates.io/crates/gpt-toolbox)
![minimum rust 1.65](https://img.shields.io/badge/rust-1.65%2B-orange.svg)
[![Documentation](https://docs.rs/gpt-toolbox/badge.svg)](https://docs.rs/gpt-toolbox)

Fork of [gpt](https://github.com/Quyzi/gpt) by Quyzi that adds:

- Support for arbitrary sector size (8K,16K, etc.)
- Multi-platform function to get sector size allowing devs to use any drive with weird sector size
- Other QoL features

## Example

```rust
use std::error::Error;
use gpt_toolbox::{GptConfig, LogicalBlockSize};

fn main() {
    // Inspect disk image, handling errors.
    if let Err(e) = run() {
        eprintln!("Failed to inspect image: {}", e);
        std::process::exit(1)
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    // First parameter is target disk image (optional, default: fixtures sample)
    let sample = "tests/fixtures/gpt-disk.img".to_string();
    let input = std::env::args().nth(1).unwrap_or(sample);

    // Open disk image with 16K sector size.
    let cfg = GptConfig::new()
        .writable(false)
        .logical_block_size(LogicalBlockSize::Other(16384));
    
    let disk = cfg.open(input)?;

    // Print GPT layout.
    println!("Disk (primary) header: {:#?}", disk.primary_header());
    println!("Partition layout: {:#?}", disk.partitions());

    Ok(())
}
```
