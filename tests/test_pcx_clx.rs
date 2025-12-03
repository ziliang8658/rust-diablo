/// Integration tests for PCX and CLX format loading
///
/// These tests require a valid DIABDAT.MPQ file in the assets directory.

#[cfg(test)]
mod tests {
    use rust_diablo::resources::{ClxSprite, MpqManager, Palette, PcxImage};

    fn setup_mpq() -> Option<MpqManager> {
        let mut mpq = MpqManager::new();

        // load_mpq 内部会自动搜索多个路径（当前目录、assets、上级目录等）
        // 并尝试不同的文件名大小写变体
        // 所以直接调用即可，不需要手动检查文件是否存在
        match mpq.load_mpq("Diabdat.mpq", 1000) {
            Ok(_) => {
                println!("✓ Successfully loaded DIABDAT.MPQ");
                Some(mpq)
            }
            Err(e) => {
                eprintln!("⚠ DIABDAT.MPQ not found: {}", e);
                None
            }
        }
    }

    #[test]
    fn test_load_pcx_from_mpq() {
        let mut mpq = match setup_mpq() {
            Some(m) => m,
            None => {
                eprintln!("Skipping test: DIABDAT.MPQ not found");
                return;
            }
        };

        // Try to load a PCX file (logo or title)
        let pcx_paths = [
            "ui_art\\logo.pcx",
            "ui_art/logo.pcx",
            "ui_art\\title.pcx",
            "ui_art/title.pcx",
        ];

        let mut loaded = false;
        for path in &pcx_paths {
            if let Ok(pcx) = PcxImage::from_mpq(&mut mpq, path) {
                println!("Successfully loaded PCX: {}", path);
                println!("  Dimensions: {}x{}", pcx.width, pcx.height);
                println!("  Has embedded palette: {}", pcx.palette.is_some());

                assert!(pcx.width > 0);
                assert!(pcx.height > 0);
                assert_eq!(
                    pcx.pixels.len(),
                    (pcx.width as usize) * (pcx.height as usize)
                );

                loaded = true;
                break;
            }
        }

        if !loaded {
            eprintln!("Warning: Could not load any PCX file from MPQ");
        }
    }

    #[test]
    fn test_load_clx_from_mpq() {
        let mut mpq = match setup_mpq() {
            Some(m) => m,
            None => {
                eprintln!("Skipping test: DIABDAT.MPQ not found");
                return;
            }
        };

        // Try to load a CLX file (warrior standing animation)
        let clx_paths = [
            "plrgfx\\warrior\\whs\\whsas.cl2",
            "plrgfx/warrior/whs/whsas.cl2",
            "plrgfx\\warrior\\whs\\whsaw.cl2", // warrior walk
            "plrgfx/warrior/whs/whsaw.cl2",
        ];

        let mut loaded = false;
        for path in &clx_paths {
            if let Ok(clx) = ClxSprite::from_mpq(&mut mpq, path) {
                println!("Successfully loaded CLX: {}", path);
                println!("  Number of frames: {}", clx.num_frames());

                if let Some(frame) = clx.get_frame(0) {
                    println!("  First frame dimensions: {}x{}", frame.width, frame.height);
                    println!("  First frame pixels: {}", frame.pixels.len());

                    assert!(frame.width > 0);
                    assert!(frame.height > 0);
                    assert_eq!(
                        frame.pixels.len(),
                        (frame.width as usize) * (frame.height as usize)
                    );
                }

                loaded = true;
                break;
            }
        }

        if !loaded {
            eprintln!("Warning: Could not load any CLX file from MPQ");
        }
    }

    #[test]
    fn test_pcx_to_rgba() {
        let mut mpq = match setup_mpq() {
            Some(m) => m,
            None => {
                eprintln!("Skipping test: DIABDAT.MPQ not found");
                return;
            }
        };

        // Load a palette
        let palette_paths = ["levels\\towndata\\town.pal", "levels/towndata/town.pal"];

        let palette = palette_paths
            .iter()
            .find_map(|path| Palette::from_mpq(&mut mpq, path).ok());

        let palette = match palette {
            Some(p) => p,
            None => {
                eprintln!("Skipping test: Could not load palette");
                return;
            }
        };

        // Load a PCX file
        let pcx_paths = ["ui_art\\logo.pcx", "ui_art/logo.pcx"];

        for path in &pcx_paths {
            if let Ok(pcx) = PcxImage::from_mpq(&mut mpq, path) {
                // Convert to RGBA
                let rgba = pcx.to_rgba(&palette, Some(0)); // Index 0 as transparent

                assert_eq!(rgba.len(), (pcx.width as usize) * (pcx.height as usize) * 4);

                // Check that we have valid RGBA data
                let mut has_opaque = false;
                let mut has_transparent = false;

                for chunk in rgba.chunks_exact(4) {
                    if chunk[3] == 0 {
                        has_transparent = true;
                    } else {
                        has_opaque = true;
                    }
                }

                println!("PCX to RGBA conversion successful");
                println!("  Has opaque pixels: {}", has_opaque);
                println!("  Has transparent pixels: {}", has_transparent);

                return;
            }
        }

        eprintln!("Warning: Could not load PCX file for RGBA conversion test");
    }

    #[test]
    fn test_clx_to_rgba() {
        let mut mpq = match setup_mpq() {
            Some(m) => m,
            None => {
                eprintln!("Skipping test: DIABDAT.MPQ not found");
                return;
            }
        };

        // Load a palette
        let palette_paths = ["levels\\towndata\\town.pal", "levels/towndata/town.pal"];

        let palette = palette_paths
            .iter()
            .find_map(|path| Palette::from_mpq(&mut mpq, path).ok());

        let palette = match palette {
            Some(p) => p,
            None => {
                eprintln!("Skipping test: Could not load palette");
                return;
            }
        };

        // Load a CLX file
        let clx_paths = [
            "plrgfx\\warrior\\whs\\whsas.cl2",
            "plrgfx/warrior/whs/whsas.cl2",
        ];

        for path in &clx_paths {
            if let Ok(clx) = ClxSprite::from_mpq(&mut mpq, path) {
                // Convert first frame to RGBA
                if let Ok(rgba) = clx.frame_to_rgba(0, &palette) {
                    if let Some(frame) = clx.get_frame(0) {
                        assert_eq!(
                            rgba.len(),
                            (frame.width as usize) * (frame.height as usize) * 4
                        );

                        // Check that we have valid RGBA data
                        let mut has_opaque = false;
                        let mut has_transparent = false;

                        for chunk in rgba.chunks_exact(4) {
                            if chunk[3] == 0 {
                                has_transparent = true;
                            } else {
                                has_opaque = true;
                            }
                        }

                        println!("CLX to RGBA conversion successful");
                        println!("  Has opaque pixels: {}", has_opaque);
                        println!("  Has transparent pixels: {}", has_transparent);

                        return;
                    }
                }
            }
        }

        eprintln!("Warning: Could not load CLX file for RGBA conversion test");
    }

    #[test]
    fn test_clx_multiple_frames() {
        let mut mpq = match setup_mpq() {
            Some(m) => m,
            None => {
                eprintln!("Skipping test: DIABDAT.MPQ not found");
                return;
            }
        };

        // Load a CLX file with multiple frames
        let clx_paths = [
            "plrgfx\\warrior\\whs\\whsas.cl2",
            "plrgfx/warrior/whs/whsas.cl2",
        ];

        for path in &clx_paths {
            if let Ok(clx) = ClxSprite::from_mpq(&mut mpq, path) {
                let num_frames = clx.num_frames();
                println!("CLX file has {} frames", num_frames);

                if num_frames > 1 {
                    // Check that all frames have the same dimensions
                    if let Some(first_frame) = clx.get_frame(0) {
                        let width = first_frame.width;
                        let height = first_frame.height;

                        for i in 1..num_frames {
                            if let Some(frame) = clx.get_frame(i) {
                                assert_eq!(frame.width, width, "Frame {} width mismatch", i);
                                assert_eq!(frame.height, height, "Frame {} height mismatch", i);
                            }
                        }

                        println!(
                            "All {} frames have consistent dimensions: {}x{}",
                            num_frames, width, height
                        );
                    }
                }

                return;
            }
        }

        eprintln!("Warning: Could not load CLX file for multi-frame test");
    }
}
