#!/usr/bin/env python3
"""
Visual Tile Verification Script

Automatically verifies that decoded tiles have correct geometric characteristics
by analyzing the exported PNG images.

Usage:
    python tests/verify_decoded_tiles.py
"""

import os
import sys
from pathlib import Path
from PIL import Image
import re

class TileVerifier:
    """Verifies decoded tile images"""
    
    # Magenta color used for transparent pixels (with some tolerance for PNG compression)
    TRANSPARENT_COLOR_TOLERANCE = 50
    
    def __init__(self, output_dir="tests/output/visual_samples"):
        self.output_dir = Path(output_dir)
        self.results = []
        
    def is_transparent_pixel(self, pixel):
        """Check if a pixel is transparent (magenta/pink)"""
        r, g, b = pixel[:3]
        # Check if it's close to magenta (255, 0, 255)
        return (r > 200 and g < self.TRANSPARENT_COLOR_TOLERANCE and b > 200)
    
    def is_opaque_pixel(self, pixel):
        """Check if a pixel is NOT transparent"""
        return not self.is_transparent_pixel(pixel)
    
    def count_row_pixels(self, img, row):
        """Count non-transparent pixels in a row"""
        count = 0
        for x in range(img.width):
            pixel = img.getpixel((x, row))
            if self.is_opaque_pixel(pixel):
                count += 1
        return count
    
    def count_leading_transparent(self, img, row):
        """Count leading transparent pixels in a row"""
        count = 0
        for x in range(img.width):
            pixel = img.getpixel((x, row))
            if self.is_transparent_pixel(pixel):
                count += 1
            else:
                break
        return count
    
    def verify_square(self, img, filename):
        """Verify Square tile (32×32, all rows full)"""
        errors = []
        
        if img.width != 32 or img.height != 32:
            errors.append(f"Wrong size: {img.width}×{img.height} (expected 32×32)")
            return errors
        
        for row in range(32):
            count = self.count_row_pixels(img, row)
            if count != 32:
                errors.append(f"Row {row}: {count} pixels (expected 32)")
        
        return errors
    
    def verify_left_triangle(self, img, filename):
        """Verify LeftTriangle tile (32×31, left-aligned)"""
        errors = []
        
        if img.width != 32 or img.height != 31:
            errors.append(f"Wrong size: {img.width}×{img.height} (expected 32×31)")
            return errors
        
        # Expected widths for all 31 rows
        expected_widths = [
            2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,  # rows 0-15
            30, 28, 26, 24, 22, 20, 18, 16, 14, 12, 10, 8, 6, 4, 2,      # rows 16-30
        ]
        
        for row, expected_width in enumerate(expected_widths):
            actual = self.count_row_pixels(img, row)
            if actual != expected_width:
                errors.append(f"Row {row}: width={actual} (expected {expected_width})")
            
            # Verify left-aligned (no leading transparent pixels)
            leading = self.count_leading_transparent(img, row)
            if leading != 0:
                errors.append(f"Row {row}: not left-aligned ({leading} leading transparent)")
        
        return errors
    
    def verify_right_triangle(self, img, filename):
        """Verify RightTriangle tile (32×31, right-aligned)"""
        errors = []
        
        if img.width != 32 or img.height != 31:
            errors.append(f"Wrong size: {img.width}×{img.height} (expected 32×31)")
            return errors
        
        # Expected widths for all 31 rows
        expected_widths = [
            2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,  # rows 0-15
            30, 28, 26, 24, 22, 20, 18, 16, 14, 12, 10, 8, 6, 4, 2,      # rows 16-30
        ]
        
        for row, expected_width in enumerate(expected_widths):
            actual = self.count_row_pixels(img, row)
            if actual != expected_width:
                errors.append(f"Row {row}: width={actual} (expected {expected_width})")
            
            # Verify right-aligned (leading transparent = 32 - width)
            leading = self.count_leading_transparent(img, row)
            expected_leading = 32 - expected_width
            if leading != expected_leading:
                errors.append(f"Row {row}: not right-aligned (leading={leading}, expected {expected_leading})")
        
        return errors
    
    def verify_left_trapezoid(self, img, filename):
        """Verify LeftTrapezoid tile (32×32, lower triangle + upper rectangle)"""
        errors = []
        
        if img.width != 32 or img.height != 32:
            errors.append(f"Wrong size: {img.width}×{img.height} (expected 32×32)")
            return errors
        
        # Lower half (rows 0-15): LeftTriangle lower half
        expected_widths_lower = [
            2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,
        ]
        
        for row, expected_width in enumerate(expected_widths_lower):
            actual = self.count_row_pixels(img, row)
            if actual != expected_width:
                errors.append(f"Row {row}: width={actual} (expected {expected_width})")
        
        # Upper half (rows 16-31): 32×16 rectangle
        for row in range(16, 32):
            actual = self.count_row_pixels(img, row)
            if actual != 32:
                errors.append(f"Row {row}: width={actual} (expected 32)")
        
        return errors
    
    def verify_right_trapezoid(self, img, filename):
        """Verify RightTrapezoid tile (32×32, lower triangle + upper rectangle)"""
        errors = []
        
        if img.width != 32 or img.height != 32:
            errors.append(f"Wrong size: {img.width}×{img.height} (expected 32×32)")
            return errors
        
        # Lower half (rows 0-15): RightTriangle lower half
        expected_widths_lower = [
            2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,
        ]
        
        for row, expected_width in enumerate(expected_widths_lower):
            actual = self.count_row_pixels(img, row)
            if actual != expected_width:
                errors.append(f"Row {row}: width={actual} (expected {expected_width})")
            
            # Verify right-aligned
            leading = self.count_leading_transparent(img, row)
            expected_leading = 32 - expected_width
            if leading != expected_leading:
                errors.append(f"Row {row}: not right-aligned (leading={leading}, expected {expected_leading})")
        
        # Upper half (rows 16-31): 32×16 rectangle
        for row in range(16, 32):
            actual = self.count_row_pixels(img, row)
            if actual != 32:
                errors.append(f"Row {row}: width={actual} (expected 32)")
        
        return errors
    
    def verify_transparent_square(self, img, filename):
        """Verify TransparentSquare tile (32×32 with transparent pixels)"""
        errors = []
        
        if img.width != 32 or img.height != 32:
            errors.append(f"Wrong size: {img.width}×{img.height} (expected 32×32)")
            return errors
        
        # Count transparent pixels
        transparent_count = 0
        for y in range(img.height):
            for x in range(img.width):
                pixel = img.getpixel((x, y))
                if self.is_transparent_pixel(pixel):
                    transparent_count += 1
        
        if transparent_count == 0:
            errors.append("No transparent pixels found")
        
        return errors
    
    def verify_tile(self, filepath):
        """Verify a single tile image"""
        filename = filepath.name
        
        # Parse filename: frame_XXXX_type_TileType.png
        match = re.match(r'frame_(\d+)_type_(\w+)\.png', filename)
        if not match:
            return {
                'filename': filename,
                'tile_type': 'Unknown',
                'status': 'SKIP',
                'errors': ['Invalid filename format']
            }
        
        frame_idx = int(match.group(1))
        tile_type = match.group(2)
        
        try:
            img = Image.open(filepath)
            img = img.convert('RGB')  # Ensure RGB mode
        except Exception as e:
            return {
                'filename': filename,
                'tile_type': tile_type,
                'status': 'ERROR',
                'errors': [f'Failed to open image: {e}']
            }
        
        # Verify based on tile type
        if tile_type == 'Square':
            errors = self.verify_square(img, filename)
        elif tile_type == 'LeftTriangle':
            errors = self.verify_left_triangle(img, filename)
        elif tile_type == 'RightTriangle':
            errors = self.verify_right_triangle(img, filename)
        elif tile_type == 'LeftTrapezoid':
            errors = self.verify_left_trapezoid(img, filename)
        elif tile_type == 'RightTrapezoid':
            errors = self.verify_right_trapezoid(img, filename)
        elif tile_type == 'TransparentSquare':
            errors = self.verify_transparent_square(img, filename)
        else:
            errors = [f'Unknown tile type: {tile_type}']
        
        status = 'PASS' if not errors else 'FAIL'
        
        return {
            'filename': filename,
            'tile_type': tile_type,
            'frame_idx': frame_idx,
            'status': status,
            'errors': errors
        }
    
    def verify_all(self):
        """Verify all tiles in the output directory"""
        if not self.output_dir.exists():
            print(f"Error: Output directory not found: {self.output_dir}")
            return False
        
        png_files = sorted(self.output_dir.glob('*.png'))
        
        if not png_files:
            print(f"Error: No PNG files found in {self.output_dir}")
            return False
        
        print(f"\n{'='*70}")
        print(f"Visual Tile Verification")
        print(f"{'='*70}\n")
        print(f"Directory: {self.output_dir}")
        print(f"Found {len(png_files)} PNG files\n")
        
        # Verify each tile
        for filepath in png_files:
            result = self.verify_tile(filepath)
            self.results.append(result)
        
        # Print results
        self.print_results()
        
        # Return overall success
        failed = sum(1 for r in self.results if r['status'] == 'FAIL')
        return failed == 0
    
    def print_results(self):
        """Print verification results"""
        # Group by tile type
        by_type = {}
        for result in self.results:
            tile_type = result['tile_type']
            if tile_type not in by_type:
                by_type[tile_type] = []
            by_type[tile_type].append(result)
        
        # Print results by type
        for tile_type, results in sorted(by_type.items()):
            print(f"\n{tile_type}:")
            print(f"{'-'*70}")
            
            passed = sum(1 for r in results if r['status'] == 'PASS')
            failed = sum(1 for r in results if r['status'] == 'FAIL')
            
            for result in results:
                status_symbol = '✓' if result['status'] == 'PASS' else '✗'
                print(f"  {status_symbol} {result['filename']}")
                
                if result['errors']:
                    for error in result['errors'][:3]:  # Show first 3 errors
                        print(f"      Error: {error}")
                    if len(result['errors']) > 3:
                        print(f"      ... and {len(result['errors'])-3} more errors")
            
            print(f"\n  Summary: {passed} passed, {failed} failed")
        
        # Overall summary
        total = len(self.results)
        total_passed = sum(1 for r in self.results if r['status'] == 'PASS')
        total_failed = sum(1 for r in self.results if r['status'] == 'FAIL')
        
        print(f"\n{'='*70}")
        print(f"Overall: {total_passed}/{total} passed, {total_failed}/{total} failed")
        print(f"{'='*70}\n")
        
        if total_failed == 0:
            print("🎉 All tiles verified successfully!")
        else:
            print(f"⚠️  {total_failed} tile(s) failed verification")

def main():
    """Main entry point"""
    verifier = TileVerifier()
    success = verifier.verify_all()
    
    sys.exit(0 if success else 1)

if __name__ == '__main__':
    main()














