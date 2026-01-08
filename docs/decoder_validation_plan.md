# Decoder验证方案

## 🎯 **问题定位**

**现象**: 大部分tile正确，少量黑色三角形artifacts
**结论**: Decoder大体正确，可能有小偏差

## 🔍 **可能的偏差点**

### **1. Row Width计算**
```rust
// 当前widths
[2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,
 30, 28, 26, 24, 22, 20, 18, 16, 14, 12, 10, 8, 6, 4, 2]
```

**验证**: 是否和C++一致？

### **2. Padding跳过**
```rust
// LeftTriangle: 偶数行前有2字节padding
src += 2;  // 跳过padding
```

**验证**: 是否所有偶数行都有padding？

### **3. 特殊tile类型**
- TransparentSquare: 可能有foliage数据
- Trapezoid: 未完全实现

## 📋 **验证步骤**

### **Step 1: 导出C++解码结果**

修改C++代码，导出解码后的数据：

```cpp
// Source/levels/reencode_dun_cels.cpp
void ReencodeDungeonCelsLeftTriangle(uint8_t *&dst, const uint8_t *&src) {
    const uint8_t *original_dst = dst;
    // ... existing code ...
    
    // Export first 10 tiles
    static int export_count = 0;
    if (export_count < 10) {
        char filename[256];
        sprintf(filename, "debug_cpp_triangle_%d.bin", export_count++);
        FILE *f = fopen(filename, "wb");
        fwrite(original_dst, 1, 512, f);
        fclose(f);
    }
}
```

### **Step 2: 导出Rust解码结果**

修改Rust decoder:

```rust
pub fn decode_left_triangle(raw_data: &[u8]) -> Result<Vec<u8>> {
    // ... existing decode ...
    
    static mut EXPORT_COUNT: usize = 0;
    unsafe {
        if EXPORT_COUNT < 10 {
            let filename = format!("debug_rust_triangle_{}.bin", EXPORT_COUNT);
            std::fs::write(&filename, &output).unwrap();
            EXPORT_COUNT += 1;
        }
    }
    
    Ok(output)
}
```

### **Step 3: 二进制对比**

```bash
# 对比前10个tile
for i in 0..9; do
    diff debug_cpp_triangle_$i.bin debug_rust_triangle_$i.bin
done
```

### **Step 4: 逐字节分析差异**

如果有差异，分析：
- 哪些位置不同？
- 差异模式（偏移、row width错误等）
- 是否只影响特定row？

---

## 🚀 **快速诊断方案**

**不修改C++**，只分析Rust输出：

### **Step A: 验证row width总和**

```rust
let widths = [2,4,6,8,10,12,14,16,18,20,22,24,26,28,30,32,
              30,28,26,24,22,20,18,16,14,12,10,8,6,4,2];
let sum: usize = widths.iter().sum();
assert_eq!(sum, 512);  // 应该等于512
```

### **Step B: 检查特定piece的解码**

```rust
// 对比piece=218 (河流)
// 1. 导出indexed data
// 2. 检查是否有异常值
// 3. 可视化为图片
```

### **Step C: 可视化解码数据**

将indexed data转换为PNG查看：

```rust
fn visualize_tile(data: &[u8], tile_type: TileType, filename: &str) {
    // 根据tile_type布局像素
    // 保存为PNG
    // 目视检查是否正常
}
```

---

## 🔧 **实施建议**

**最快诊断**: 
1. 导出前10个渲染的tile为PNG
2. 目视检查哪些有问题
3. 分析问题tile的特征

**代码**:
```rust
// In render_indexed_tile_to_framebuffer
if piece_id == 218 || piece_id == 856 {
    export_tile_as_png(&data, tile_type, piece_id, block_id);
}
```




