/// Tool to compare SOL data between C++ and Rust versions
/// 
/// This helps diagnose why Rust has missing floor tiles
/// 
/// Usage:
/// 1. Run C++ version and export SOL data
/// 2. Run Rust version and export SOL data  
/// 3. Compare the two files

use std::fs::File;
use std::io::Write;

fn main() -> anyhow::Result<()> {
    println!("SOL Data Comparison Tool");
    println!("========================");
    
    // TODO: Load SOL data from both versions
    // For now, we'll output diagnostic info
    
    println!("\nTo use this tool:");
    println!("1. Add diagnostic output to C++ version (already done)");
    println!("2. Add diagnostic output to Rust version (see below)");
    println!("3. Compare dpiece_cpp.txt and dpiece_rust.txt");
    println!("4. Look for differences in piece IDs that are floor vs solid");
    
    Ok(())
}



