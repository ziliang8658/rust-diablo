/// Example: List all files in Diabdat.mpq
/// 
/// This example demonstrates how to:
/// 1. Open a MPQ archive
/// 2. Read the (listfile) to get all files
/// 3. Display file information with statistics

use anyhow::Result;
use rust_diablo::resources::MpqManager;
use std::collections::HashMap;

fn main() -> Result<()> {
    println!("=== MPQ File Lister ===\n");
    
    // Create MPQ manager
    let mut mpq_manager = MpqManager::new();
    
    // Try to load Diabdat.mpq from various locations
    let mpq_paths = vec![
        "assets/Diabdat.mpq",
        "assets/DIABDAT.MPQ",
        "Diabdat.mpq",
        "DIABDAT.MPQ",
        "rust-diablo/assets/Diabdat.mpq",
    ];
    
    let mut loaded = false;
    let searched_paths = mpq_paths.clone();
    
    for path in mpq_paths {
        match mpq_manager.load_mpq(&path, 1000) {
            Ok(_) => {
                loaded = true;
                println!("✓ Successfully loaded MPQ archive: {}\n", path);
                break;
            }
            Err(_) => {
                continue;
            }
        }
    }
    
    if !loaded {
        eprintln!("❌ Error: Could not load Diabdat.mpq from any of the following paths:");
        for path in searched_paths {
            eprintln!("  - {}", path);
        }
        eprintln!("\nPlease ensure Diabdat.mpq is in one of these locations.");
        return Ok(());
    }
    
    // List all files in the MPQ
    println!("=== Reading MPQ file list ===\n");
    
    // Try to read the special "(listfile)" file
    // This file contains a list of all files in the MPQ archive
    match mpq_manager.find_file("(listfile)") {
        Some(listfile_data) => {
            println!("✓ Found (listfile) - {} bytes\n", listfile_data.len());
            
            // Convert bytes to string (listfile is text format)
            match std::str::from_utf8(&listfile_data) {
                Ok(listfile_text) => {
                    // Split by lines to get individual file paths
                    let files: Vec<&str> = listfile_text
                        .lines()
                        .filter(|line| !line.trim().is_empty())
                        .collect();
                    
                    println!("=== Files in MPQ ({} total) ===\n", files.len());
                    
                    // Group files by directory for better readability
                    let mut by_directory: HashMap<String, Vec<String>> = HashMap::new();
                    
                    for file_path in &files {
                        // Extract directory
                        let dir = if let Some(last_slash) = file_path.rfind('/') {
                            file_path[..last_slash].to_string()
                        } else if let Some(last_backslash) = file_path.rfind('\\') {
                            file_path[..last_backslash].to_string()
                        } else {
                            "(root)".to_string()
                        };
                        
                        by_directory.entry(dir)
                            .or_insert_with(Vec::new)
                            .push(file_path.to_string());
                    }
                    
                    // Sort directories
                    let mut sorted_dirs: Vec<_> = by_directory.keys().cloned().collect();
                    sorted_dirs.sort();
                    
                    // Print files grouped by directory
                    for dir in &sorted_dirs {
                        let dir_files = &by_directory[dir];
                        println!("[{}] ({} files)", dir, dir_files.len());
                        
                        // Sort files in directory
                        let mut sorted_files = dir_files.clone();
                        sorted_files.sort();
                        
                        for file in &sorted_files {
                            // Get file size if possible
                            if let Some(data) = mpq_manager.find_file(file) {
                                let size = data.len();
                                let size_str = format_size(size);
                                
                                // Get file extension
                                let ext = if let Some(dot) = file.rfind('.') {
                                    &file[dot..]
                                } else {
                                    ""
                                };
                                
                                println!("  - {} ({}) {}", file, size_str, ext);
                            } else {
                                println!("  - {} (read error)", file);
                            }
                        }
                        println!();
                    }
                    
                    // Print summary statistics
                    println!("=== Summary Statistics ===");
                    println!("Total files: {}", files.len());
                    println!("Total directories: {}", sorted_dirs.len());
                    
                    // Calculate total size
                    let mut total_size = 0u64;
                    let mut readable_files = 0;
                    for file in &files {
                        if let Some(data) = mpq_manager.find_file(file) {
                            total_size += data.len() as u64;
                            readable_files += 1;
                        }
                    }
                    
                    println!("Readable files: {}/{}", readable_files, files.len());
                    println!("Total size: {}", format_size(total_size as usize));
                    
                    // Count files by extension
                    let mut by_extension: HashMap<String, usize> = HashMap::new();
                    
                    for file_path in &files {
                        let ext = if let Some(dot) = file_path.rfind('.') {
                            file_path[dot..].to_lowercase()
                        } else {
                            "(no ext)".to_string()
                        };
                        *by_extension.entry(ext).or_insert(0) += 1;
                    }
                    
                    println!("\n=== Files by extension ===");
                    let mut sorted_exts: Vec<_> = by_extension.iter().collect();
                    sorted_exts.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
                    
                    for (ext, count) in sorted_exts {
                        println!("  {}: {}", ext, count);
                    }
                    
                    // Show some interesting file types
                    println!("\n=== Interesting Files ===");
                    
                    let interesting_patterns = vec![
                        (".pal", "Palette files"),
                        (".pcx", "Image files (PCX)"),
                        (".cl2", "Sprite files (CL2)"),
                        (".cel", "Sprite files (CEL)"),
                        (".min", "Minimap files"),
                        (".til", "Tile files"),
                        (".sol", "Solid/collision files"),
                        (".dun", "Dungeon layout files"),
                        (".wav", "Audio files"),
                    ];
                    
                    for (ext, description) in interesting_patterns {
                        let count = by_extension.get(ext).unwrap_or(&0);
                        if *count > 0 {
                            println!("  {}: {} files", description, count);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Error: (listfile) is not valid UTF-8: {}", e);
                    eprintln!("The MPQ archive may be corrupted or use a different encoding.");
                }
            }
        }
        None => {
            eprintln!("❌ Error: (listfile) not found in MPQ");
            eprintln!("\nThis MPQ archive does not contain a listfile.");
            eprintln!("Trying to list known common files instead...\n");
            
            // Fallback: try some common file paths
            let common_paths = vec![
                "levels/towndata/town.pal",
                "levels/towndata/town.min",
                "levels/towndata/town.til",
                "ui_art/logo.pcx",
                "ui_art/title.pcx",
                "plrgfx/warrior/whs/whsas.cl2",
            ];
            
            println!("=== Common Files Found ===\n");
            let mut found_count = 0;
            
            for path in common_paths {
                if let Some(data) = mpq_manager.find_file(path) {
                    println!("  ✓ {} ({} bytes)", path, data.len());
                    found_count += 1;
                }
            }
            
            if found_count == 0 {
                println!("  No common files found.");
            } else {
                println!("\nFound {} common files.", found_count);
            }
        }
    }
    
    println!("\n=== Done ===");
    Ok(())
}

/// Format file size in human-readable format
fn format_size(size: usize) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.2} KB", size as f64 / 1024.0)
    } else {
        format!("{:.2} MB", size as f64 / (1024.0 * 1024.0))
    }
}
