/// MaskType functionality tests
///
/// Tests for the transparency mask system (Step 6.4.2)

use rust_diablo::tiles::MaskType;

#[test]
fn test_left_mask_prefix_progression() {
    // Test prefix values for Left mask across all rows
    
    // Bottom half (rows 0-15): should all be negative or 0
    assert_eq!(MaskType::Left.get_prefix_for_row(0), -32);
    assert_eq!(MaskType::Left.get_prefix_for_row(8), -16);
    assert_eq!(MaskType::Left.get_prefix_for_row(15), -2);
    
    // Transition point
    assert_eq!(MaskType::Left.get_prefix_for_row(16), 0);
    
    // Upper half (rows 17-31): should be positive
    assert_eq!(MaskType::Left.get_prefix_for_row(20), 8);
    assert_eq!(MaskType::Left.get_prefix_for_row(25), 18);
    assert_eq!(MaskType::Left.get_prefix_for_row(31), 30);
}

#[test]
fn test_right_mask_prefix_progression() {
    // Test prefix values for Right mask across all rows
    
    // Bottom half (rows 0-15): should all be >= 32 (entire row opaque)
    assert_eq!(MaskType::Right.get_prefix_for_row(0), 64);
    assert_eq!(MaskType::Right.get_prefix_for_row(8), 48);
    assert_eq!(MaskType::Right.get_prefix_for_row(15), 34);
    
    // Transition point
    assert_eq!(MaskType::Right.get_prefix_for_row(16), 32);
    
    // Upper half (rows 17-31): should be < 32 (partial transparency)
    assert_eq!(MaskType::Right.get_prefix_for_row(20), 24);
    assert_eq!(MaskType::Right.get_prefix_for_row(25), 14);
    assert_eq!(MaskType::Right.get_prefix_for_row(31), 2);
}

#[test]
fn test_mask_symmetry() {
    // Left and Right masks should be symmetric
    // At row R, Left has N transparent pixels, Right should have N opaque pixels
    
    for row in 0..32i8 {
        let left_prefix = MaskType::Left.get_prefix_for_row(row);
        let right_prefix = MaskType::Right.get_prefix_for_row(row);
        
        // left_prefix + (32 - right_prefix) should equal 32
        // Because: transparent_left + transparent_right = 0 (they don't overlap)
        let left_transparent = left_prefix.max(0);
        let right_transparent = (32 - right_prefix).max(0);
        
        // Sum should be <= 32 (no overlap in transparent regions)
        assert!(left_transparent + right_transparent <= 32,
            "Row {}: left_transparent={}, right_transparent={}, sum={}",
            row, left_transparent, right_transparent, left_transparent + right_transparent);
    }
}

