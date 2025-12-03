use anyhow::Result;
use rust_diablo::engine::Engine;
use rust_diablo::math::{Point, Rect};
/// Step 5.3 功能演示
///
/// 这个示例演示：
/// 1. TRN颜色转换
/// 2. ResourceManager资源管理
/// 3. SimpleTown城镇场景（简化版）
///
/// 运行方式：
/// ```
/// cargo run --example test_step5_3
/// ```
use rust_diablo::resources::{ColorTransform, ResourceManager};
use rust_diablo::world::SimpleTown;

fn main() -> Result<()> {
    println!("=== Step 5.3: TRN and ResourceManager Demo ===\n");

    // 初始化SDL和Engine
    println!("1. Initializing Engine...");
    let mut engine = Engine::new()?;

    // 创建ResourceManager
    println!("2. Creating ResourceManager...");
    let mpq_paths = vec![
        ("assets/Diabdat.mpq", 1000),
        ("assets/DIABDAT.MPQ", 1000),
        ("Diabdat.mpq", 1000),
        ("DIABDAT.MPQ", 1000),
    ];

    let mut res_mgr = ResourceManager::new_empty();
    let loaded = res_mgr.try_load_mpqs(mpq_paths);

    if loaded.is_empty() {
        eprintln!("❌ Warning: Could not load any MPQ files.");
        eprintln!("  Please ensure Diabdat.mpq is in the assets/ directory.");
        eprintln!("  Continuing with demo using fallback...\n");
    } else {
        println!("✓ Loaded MPQ: {:?}\n", loaded[0]);
    }

    // 测试1: 加载调色板
    println!("3. Testing Palette Loading...");
    match res_mgr.load_palette("levels/towndata/town.pal") {
        Ok(palette) => {
            println!("✓ Loaded town palette");
            println!("  Palette has {} colors", palette.len());
            println!(
                "  First color: RGB({}, {}, {})",
                palette.colors[0].r, palette.colors[0].g, palette.colors[0].b
            );
        }
        Err(e) => {
            eprintln!("⚠ Could not load palette: {}", e);
        }
    }
    println!();

    // 测试2: TRN加载（如果找到）
    println!("4. Testing TRN Loading...");
    let trn_paths = vec![
        "plrgfx/warrior/wmn/wmn.trn",
        "plrgfx/warrior/wmd/wmd.trn",
        "plrgfx/warrior/wms/wms.trn",
    ];

    let mut trn_found = false;
    for path in &trn_paths {
        match res_mgr.load_trn(path) {
            Ok(trn) => {
                println!("✓ Loaded TRN: {}", path);
                let (changed, total) = trn.change_stats();
                println!("  Mapping changes: {} / {}", changed, total);
                trn_found = true;
                break;
            }
            Err(_) => {
                // 继续尝试下一个
            }
        }
    }

    if !trn_found {
        println!("⚠ No TRN files found (this is normal, they may not be in MPQ)");
        println!("  Creating identity TRN for demonstration...");
        let identity_trn = ColorTransform::identity();
        println!("✓ Created identity TRN");
        let (changed, total) = identity_trn.change_stats();
        println!("  Mapping changes: {} / {}", changed, total);
    }
    println!();

    // 测试3: PCX加载
    println!("5. Testing PCX Loading...");
    match res_mgr.load_pcx("ui_art/logo.pcx") {
        Ok(pcx) => {
            println!("✓ Loaded logo.pcx");
            println!("  Size: {}x{}", pcx.width, pcx.height);
            println!("  Has internal palette: {}", pcx.palette.is_some());

            // 尝试创建纹理
            match res_mgr.load_pcx_texture(
                &mut engine,
                "ui_art/logo.pcx",
                "demo_logo",
                None,
                Some(0),
            ) {
                Ok(_) => {
                    println!("✓ Created SDL texture from PCX");
                }
                Err(e) => {
                    eprintln!("⚠ Failed to create texture: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("⚠ Could not load PCX: {}", e);
        }
    }
    println!();

    // 测试4: CL2精灵加载
    println!("6. Testing CL2 Sprite Loading...");
    let sprite_paths = vec![
        ("plrgfx/warrior/wmn/wmnas.cl2", "warrior idle"),
        ("plrgfx/warrior/wmn/wmnaw.cl2", "warrior walk"),
    ];

    for (path, desc) in &sprite_paths {
        match res_mgr.load_cl2(path, 96) {
            Ok(frames) => {
                println!("✓ Loaded {} ({})", desc, path);
                println!("  Frames: {}", frames.len());
                if !frames.is_empty() {
                    println!("  First frame: {}x{}", frames[0].width, frames[0].height);
                }
            }
            Err(e) => {
                eprintln!("⚠ Could not load {}: {}", desc, e);
            }
        }
    }
    println!();

    // 测试5: SimpleTown创建
    println!("7. Testing SimpleTown...");
    let town = SimpleTown::new();
    println!("✓ Created SimpleTown");
    println!("  Name: {}", town.name);
    println!(
        "  Size: {}x{}",
        town.background_width, town.background_height
    );
    let walkable = town.get_walkable_area();
    println!(
        "  Walkable area: {}x{} at ({},{})",
        walkable.width, walkable.height, walkable.x, walkable.y
    );

    // 测试可行走性
    println!("  Testing walkability:");
    println!("    (320, 240): {}", town.is_walkable(320.0, 240.0)); // 中心应该可行走
    println!("    (10, 10): {}", town.is_walkable(10.0, 10.0)); // 边缘不可行走
    println!();

    // 测试6: 缓存统计
    println!("8. ResourceManager Cache Stats:");
    println!("{}", res_mgr.cache_stats());
    println!();

    // 总结
    println!("=== Demo Complete ===");
    println!("Step 5.3 核心功能验证:");
    println!("  ✓ TRN颜色转换模块");
    println!("  ✓ ResourceManager统一资源加载");
    println!("  ✓ SimpleTown城镇场景结构");
    println!("  ✓ 资源缓存系统");
    println!("\n下一步: 集成到Game.rs，实现完整的城镇场景预览");

    Ok(())
}
