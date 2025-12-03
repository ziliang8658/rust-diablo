/// Debug CLX file format
use std::fs;

fn main() -> anyhow::Result<()> {
    let paths = vec![
        "../assets/gendata/cut2w.clx",
        "../assets/gendata/cutgatew.clx",
    ];

    for path in paths {
        println!("\n=== Analyzing {} ===", path);

        match fs::read(path) {
            Ok(data) => {
                println!("File size: {} bytes", data.len());

                if data.len() < 8 {
                    println!("File too small!");
                    continue;
                }

                // Read header
                let num_frames = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                println!("Num frames (first 4 bytes): {}", num_frames);

                // Print first 64 bytes as hex
                println!("First 64 bytes:");
                for i in 0..64.min(data.len()) {
                    if i % 16 == 0 {
                        print!("\n  {:04x}: ", i);
                    }
                    print!("{:02x} ", data[i]);
                }
                println!();

                // Try reading as CLX
                match rust_diablo::resources::ClxSprite::from_bytes(&data) {
                    Ok(sprite) => {
                        println!("✓ Successfully parsed as CLX");
                        println!("  Frames: {}", sprite.frames.len());
                    }
                    Err(e) => {
                        println!("✗ Failed to parse: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("Failed to read file: {}", e);
            }
        }
    }

    Ok(())
}
