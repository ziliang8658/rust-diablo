# 快速测试说明

## 已添加的诊断日志

### Triangle解码器内部日志

**文件**: `rust-diablo/src/tiles/decoder/triangle.rs`

**输出内容**（前5个decode调用）:
```
[DECODE_LEFT_TRIANGLE #1] raw_size=544 first_bytes=[xx xx xx xx]
[DECODE_LEFT_TRIANGLE #1] compact_size=??? first_10=[??,??,??,...]
[DECODE_LEFT_TRIANGLE #1] padded_size=992 first_10=[??,??,??,...]
```

**关键检查**:
1. `compact_size` - 应该是所有行宽度之和
2. `compact first_10` - 应该是 [57, 57, 45, 46, 47, 48, ...]（如果正确）
3. `padded first_10` - 应该是 [57, 57, 0, 0, ...]（Row0后padding）

### 桥piece的Alpha分析

**输出内容**:
```
🌉 [BRIDGE piece=38 block=0] Alpha analysis:
   First 10 palette indices: [???, ???, ...]
```

---

## 测试步骤

### 1. 运行程序

```bash
cargo run > test_output.log 2>&1
```

### 2. 查看解码日志

```powershell
Get-Content test_output.log | Select-String -Pattern "DECODE_LEFT_TRIANGLE" | Select-Object -First 10
```

### 3. 查看桥诊断

```powershell
Get-Content test_output.log | Select-String -Pattern "BRIDGE piece=38 block=0"
```

---

## 预期结果

### 场景A: compact_size正确，但padding有问题

```
[DECODE_LEFT_TRIANGLE #1] compact_size=496 first_10=[57, 57, 45, 46, 47, 48, ...]
[DECODE_LEFT_TRIANGLE #1] padded_size=992 first_10=[57, 57, 0, 0, ...]  ← padding后变回错的
```

**问题**: padding转换逻辑有bug  
**修复**: 检查widths数组和转换逻辑

### 场景B: compact_size就是错的

```
[DECODE_LEFT_TRIANGLE #1] compact_size=6 first_10=[57, 57, 0, 0, ...]  ← 只有2个非零
```

**问题**: compact输出本身就错了  
**修复**: 检查extend_from_slice是否正确执行

### 场景C: 根本没调用decode_left_triangle

```
(没有DECODE_LEFT_TRIANGLE日志)
```

**问题**: 使用了缓存数据，没有重新解码  
**修复**: 清除缓存或禁用缓存

---

## 如果还是不对

### 禁用缓存测试

在`TileTextureManager::get_indexed_tile`中，临时禁用缓存：

```rust
// 注释掉这行
// if self.indexed_cache.contains_key(&cache_key) {
//     return Ok(&self.indexed_cache[&cache_key]);
// }
```

强制每次都重新解码，确保使用最新的解码逻辑。

---

**请运行程序，把DECODE_LEFT_TRIANGLE的日志发给我！**




