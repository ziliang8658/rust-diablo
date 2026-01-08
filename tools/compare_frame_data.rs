/// Tool to compare frame data between Rust and C++ implementations
///
/// This tool extracts frame data from Rust's TileTextureManager and outputs it
/// in a format that can be compared with C++'s GetDunFrame output.
///
/// Usage:
///   cargo run --bin compare_frame_data -- <frame_index> [tile_type] [output_file]
///
/// Example:
///   cargo run --bin compare_frame_data -- 1 LeftTriangle frame_1_left.bin
///   cargo run --bin compare_frame_data -- 1 RightTriangle frame_1_right.bin
///   cargo run --bin compare_frame_data -- 100 TransparentSquare frame_100_trans.bin

use anyhow::{Context, Result};
use rust_diablo::resources::mpq::MpqManager;
use rust_diablo::tiles::texture_manager::TileTextureManager;
use rust_diablo::tiles::types::{DungeonType, TileType};
use std::env;
use std::fs::File;
use std::io::Write;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <frame_index> [tile_type] [output_file]", args[0]);
        eprintln!("  frame_index: 1-based frame index (as used in MIN data)");
        eprintln!("  tile_type: Optional - LeftTriangle, RightTriangle, Square, TransparentSquare, LeftTrapezoid, RightTrapezoid");
        eprintln!("  output_file: Optional - binary file to save the data");
        eprintln!("\nExample:");
        eprintln!("  {} 1 LeftTriangle frame_1_left.bin", args[0]);
        eprintln!("  {} 100 TransparentSquare frame_100_trans.bin", args[0]);
        return Ok(());
    }

    let frame_idx: usize = args[1]
        .parse()
        .context("Invalid frame index")?;

    let tile_type = if args.len() >= 3 {
        match args[2].as_str() {
            "LeftTriangle" => TileType::LeftTriangle,
            "RightTriangle" => TileType::RightTriangle,
            "Square" => TileType::Square,
            "TransparentSquare" => TileType::TransparentSquare,
            "LeftTrapezoid" => TileType::LeftTrapezoid,
            "RightTrapezoid" => TileType::RightTrapezoid,
            _ => {
                eprintln!("Unknown tile type: {}", args[2]);
                eprintln!("Valid types: LeftTriangle, RightTriangle, Square, TransparentSquare, LeftTrapezoid, RightTrapezoid");
                return Ok(());
            }
        }
    } else {
        // Default to LeftTriangle for testing
        TileType::LeftTriangle
    };

    println!("=== Frame Data Comparison Tool ===");
    println!("Frame index: {} (1-based)", frame_idx);
    println!("Tile type: {:?}", tile_type);

    // Initialize MPQ manager
    let mut mpq_manager = MpqManager::new();
    
    // Load texture manager (it will load palette and MIN data internally)
    let mut texture_mgr = TileTextureManager::load_for_dungeon(DungeonType::Town, &mut mpq_manager)?;

    println!("\n=== Frame Information ===");
    
    // Get frame (raw, before decoding)
    let frame = texture_mgr.get_frame(frame_idx)?;
    println!("Frame {} (1-based) / {} (0-based):", frame_idx, frame_idx - 1);
    println!("  Raw data size: {} bytes", frame.raw_data.len());
    println!("  Is decoded: {}", frame.is_decoded);
    
    // Print first 32 bytes of raw data
    let preview_len = frame.raw_data.len().min(32);
    println!("  First {} bytes (hex):", preview_len);
    for (i, chunk) in frame.raw_data[..preview_len].chunks(16).enumerate() {
        let hex_str: Vec<String> = chunk.iter().map(|b| format!("{:02X}", b)).collect();
        println!("    {:04X}: {}", i * 16, hex_str.join(" "));
    }

    // Get decoded data
    println!("\n=== Decoded Data ===");
    let decoded_data = texture_mgr.get_indexed_tile(frame_idx, tile_type)?;
    println!("Decoded data size: {} bytes", decoded_data.len());
    
    // Print first 32 bytes of decoded data
    let preview_len = decoded_data.len().min(32);
    println!("  First {} bytes (hex):", preview_len);
    for (i, chunk) in decoded_data[..preview_len].chunks(16).enumerate() {
        let hex_str: Vec<String> = chunk.iter().map(|b| format!("{:02X}", b)).collect();
        println!("    {:04X}: {}", i * 16, hex_str.join(" "));
    }

    // Print statistics
    println!("\n=== Data Statistics ===");
    println!("Raw data:");
    println!("  Size: {} bytes", frame.raw_data.len());
    println!("  Expected for {:?}:", tile_type);
    match tile_type {
        TileType::LeftTriangle | TileType::RightTriangle => {
            println!("    Original (with padding): 544 bytes");
            println!("    Re-encoded (compact): 512 bytes");
            println!("  Actual: {} bytes", frame.raw_data.len());
        }
        TileType::Square => {
            println!("    Expected: 1024 bytes (32x32)");
            println!("  Actual: {} bytes", frame.raw_data.len());
        }
        TileType::TransparentSquare => {
            println!("    Expected: Variable (RLE encoded)");
            println!("  Actual: {} bytes", frame.raw_data.len());
        }
        TileType::LeftTrapezoid | TileType::RightTrapezoid => {
            println!("    Expected: ~800 bytes (variable)");
            println!("  Actual: {} bytes", frame.raw_data.len());
        }
    }
    
    println!("\nDecoded data:");
    println!("  Size: {} bytes", decoded_data.len());
    match tile_type {
        TileType::LeftTriangle | TileType::RightTriangle => {
            println!("  Expected: 512 bytes (compact, no padding)");
            if decoded_data.len() == 512 {
                println!("  ✅ Size matches expected!");
            } else {
                println!("  ⚠️ Size mismatch! Expected 512, got {}", decoded_data.len());
            }
        }
        TileType::Square => {
            println!("  Expected: 1024 bytes (32x32)");
            if decoded_data.len() == 1024 {
                println!("  ✅ Size matches expected!");
            } else {
                println!("  ⚠️ Size mismatch! Expected 1024, got {}", decoded_data.len());
            }
        }
        _ => {
            println!("  Variable size (RLE or trapezoid)");
        }
    }

    // Save to file if requested
    if args.len() >= 4 {
        let output_file = &args[3];
        println!("\n=== Saving Data ===");
        
        // Save raw data
        let raw_file = format!("{}_raw.bin", output_file.trim_end_matches(".bin"));
        let mut f = File::create(&raw_file)?;
        f.write_all(&frame.raw_data)?;
        println!("Saved raw data to: {}", raw_file);
        
        // Save decoded data
        let mut f = File::create(output_file)?;
        f.write_all(decoded_data)?;
        println!("Saved decoded data to: {}", output_file);
        
        println!("\nTo compare with C++:");
        println!("  1. In C++, call GetDunFrame(pDungeonCels.get(), {})", frame_idx);
        println!("  2. Save the data to a file");
        println!("  3. Compare the files using:");
        println!("     fc /b {} <cpp_output_file>", output_file);
    } else {
        println!("\n=== Usage ===");
        println!("To save data for comparison, specify output file:");
        println!("  {} {} {:?} frame_{}_decoded.bin", args[0], frame_idx, tile_type, frame_idx);
    }

    Ok(())
}

