/// Debug tool to verify tile data loading and rendering
///
/// Run with: cargo run --example debug_tiles
use rust_diablo::resources::MpqManager;
use rust_diablo::tiles::types::DungeonType;
use rust_diablo::tiles::{texture_manager::TileTextureManager, MinData, TilData};

fn main() -> anyhow::Result<()> {
    println!("=== Tile Data Debug Tool ===\n");

    // Load MPQ - try multiple paths
    let mut mpq = MpqManager::new();
    let paths = [
        "diabdat.mpq",
        "../diabdat.mpq",
        "Diabdat.mpq",
        "../Diabdat.mpq",
    ];
    let mut loaded = false;
    for path in &paths {
        if mpq.load_mpq(path, 0).is_ok() {
            println!("Loaded MPQ from: {}", path);
            loaded = true;
            break;
        }
    }
    if !loaded {
        return Err(anyhow::anyhow!("Could not find diabdat.mpq"));
    }
    println!("✓ MPQ loaded\n");

    // Load Cathedral data
    let dungeon_type = DungeonType::Cathedral;

    // Load TIL
    let til_data = TilData::load_for_dungeon(&mut mpq, dungeon_type)?;
    println!("TIL: {} MegaTiles", til_data.len());

    // Print first 5 MegaTiles
    println!("\nFirst 5 MegaTiles (TIL data):");
    for i in 0..5.min(til_data.len()) {
        if let Some(mega) = til_data.get(i) {
            println!(
                "  MegaTile[{}]: micro1={}, micro2={}, micro3={}, micro4={}",
                i, mega.micro1, mega.micro2, mega.micro3, mega.micro4
            );
        }
    }

    // Load MIN
    let min_data = MinData::load_for_dungeon(&mut mpq, dungeon_type)?;
    println!(
        "\nMIN: {} pieces, {} blocks/piece",
        min_data.len(),
        min_data.blocks_per_piece
    );

    // Print first 3 pieces
    println!("\nFirst 3 Pieces (MIN data):");
    for i in 0..3.min(min_data.len()) {
        if let Some(piece) = min_data.get(i) {
            println!("  Piece[{}]:", i);
            for (block_idx, block) in piece.mt.iter().enumerate().take(4) {
                println!(
                    "    Block[{}]: data={:#06x}, frame={}, type={:?}",
                    block_idx,
                    block.data,
                    block.frame(),
                    block.tile_type()
                );
            }
        }
    }

    // Load texture manager
    let mut tex_mgr = TileTextureManager::load_for_dungeon(dungeon_type, &mut mpq)?;
    println!(
        "\nTexture Manager: {} pieces, {} frames",
        tex_mgr.len(),
        tex_mgr.total_frames()
    );

    // Check specific tiles
    println!("\n=== Checking Floor tile (MegaTile index 12, Tile::Floor=13) ===");
    if let Some(mega) = til_data.get(12) {
        println!(
            "MegaTile[12]: micro1={}, micro2={}, micro3={}, micro4={}",
            mega.micro1, mega.micro2, mega.micro3, mega.micro4
        );

        // Try to decode the floor tile
        let piece_id = mega.micro1 as usize;
        if piece_id < min_data.len() {
            if let Some(piece) = min_data.get(piece_id) {
                println!("  -> Piece[{}] (from micro1):", piece_id);
                for (block_idx, block) in piece.mt.iter().enumerate().take(2) {
                    println!(
                        "    Block[{}]: frame={}, type={:?}",
                        block_idx,
                        block.frame(),
                        block.tile_type()
                    );

                    // Try to decode
                    match tex_mgr.get_decoded_tile(piece_id, block_idx) {
                        Ok(rgba) => {
                            let non_transparent = rgba.chunks(4).filter(|p| p[3] != 0).count();
                            println!(
                                "      Decoded: {} bytes, {} non-transparent pixels",
                                rgba.len(),
                                non_transparent
                            );
                        }
                        Err(e) => println!("      Decode error: {}", e),
                    }
                }
            }
        }
    }

    println!("\n=== Checking Wall tile (MegaTile index 0, Tile::VWall=1) ===");
    if let Some(mega) = til_data.get(0) {
        println!(
            "MegaTile[0]: micro1={}, micro2={}, micro3={}, micro4={}",
            mega.micro1, mega.micro2, mega.micro3, mega.micro4
        );
    }

    println!("\n=== Checking Dirt tile (MegaTile index 21, Tile::Dirt=22) ===");
    if let Some(mega) = til_data.get(21) {
        println!(
            "MegaTile[21]: micro1={}, micro2={}, micro3={}, micro4={}",
            mega.micro1, mega.micro2, mega.micro3, mega.micro4
        );
    }

    // Check ALL blocks of Dirt pieces
    println!("\n=== Checking ALL blocks of Dirt pieces ===");
    for piece_id in [39usize, 30, 35, 32] {
        if let Some(piece) = min_data.get(piece_id) {
            println!("Piece[{}] ({} blocks):", piece_id, piece.mt.len());
            for (idx, block) in piece.mt.iter().enumerate() {
                if block.has_value() {
                    println!(
                        "  mt[{}]: frame={}, type={:?}, HAS_VALUE=TRUE!",
                        idx,
                        block.frame(),
                        block.tile_type()
                    );
                }
            }
            if piece.mt.iter().all(|b| !b.has_value()) {
                println!("  All blocks are empty (has_value=false)");
            }
        }
    }

    // Test dungeon generation
    println!("\n=== Testing Dungeon Generation ===");
    use rust_diablo::levels::drlg_l1;

    let generator = drlg_l1::create_l1_dungeon(12345, &til_data);

    // Print d_piece array at the boundary (around offset 16)
    println!("d_piece array at boundary (rows 14-18, cols 14-22):");
    for j in 14..18 {
        print!("  row {:2}: ", j);
        for i in 14..22 {
            print!("{:4}", generator.d_piece[i][j]);
        }
        println!();
    }

    // Check what piece IDs appear in the visible area
    println!("\nUnique piece IDs in d_piece[16..56][16..56]:");
    let mut piece_ids: std::collections::HashSet<u16> = std::collections::HashSet::new();
    for j in 16..56 {
        for i in 16..56 {
            piece_ids.insert(generator.d_piece[i][j]);
        }
    }
    let mut sorted: Vec<_> = piece_ids.iter().collect();
    sorted.sort();
    println!("  {:?}", sorted);

    // Print a small section of d_piece
    println!("\nd_piece array (center 10x10 at offset 16):");
    for j in 46..56 {
        print!("  ");
        for i in 46..56 {
            print!("{:4}", generator.d_piece[i][j]);
        }
        println!();
    }

    // Check what pieces are used
    println!("\nPiece details for Floor tile:");
    for piece_id in [22u16, 1, 6, 3] {
        if let Some(piece) = min_data.get(piece_id as usize) {
            println!("  Piece[{}]:", piece_id);
            for (idx, block) in piece.mt.iter().enumerate().take(4) {
                println!(
                    "    mt[{}]: frame={}, type={:?}, has_value={}",
                    idx,
                    block.frame(),
                    block.tile_type(),
                    block.has_value()
                );
            }
        }
    }

    // Test Dirt pieces (should be empty/black)
    println!("\n=== Checking Dirt Pieces (should all be has_value=false) ===");
    for piece_id in [39usize, 30, 35, 32] {
        if let Some(piece) = min_data.get(piece_id) {
            println!("Piece[{}]:", piece_id);
            for (idx, block) in piece.mt.iter().enumerate().take(2) {
                println!(
                    "  mt[{}]: frame={}, type={:?}, has_value={}",
                    idx,
                    block.frame(),
                    block.tile_type(),
                    block.has_value()
                );
            }
        }
    }

    println!("\n=== Done ===");
    Ok(())
}
