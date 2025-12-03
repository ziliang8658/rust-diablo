# MPQ-Rust 补丁分析

## 问题根源

### mpq-rust 的 read() 函数

```rust
pub fn read(&self, archive: &mut Archive, buf: &mut [u8]) -> Result<usize, Error> {
    if self.block.flags & FILE_PATCH_FILE != 0 {
        Err(Error::new(ErrorKind::Other, "Patch file not supported"))
    } else if self.block.flags & FILE_SINGLE_UNIT != 0 {
        self.read_single_unit_file(...)
    } else {
        self.read_sector_file(...)
    }
}
```

### Flag 定义冲突

**现代 MPQ (WoW, SC2):**
```
FILE_PATCH_FILE  = 0x00010000  (bit 16)
FILE_SINGLE_UNIT = 0x00020000  (bit 17)
```

**Diablo 1 MPQ:**
```
FLAG_EXISTS      = 0x80000000  (bit 31)
FLAG_COMPRESS    = 0x00000100  (bit 8)
```

### 为什么会冲突？

Diablo 1 的 flags 可能在 bit 16/17 有值，导致 mpq-rust 误判为 "patch file"。

## 解决方案

### 方案 A: Fork mpq-rust 并修改

```toml
# Cargo.toml
[dependencies]
mpq = { git = "https://github.com/YOUR-FORK/mpq-rust.git", branch = "diablo1-compat" }
```

修改内容：
```rust
// 移除或修改这个检查
// if self.block.flags & FILE_PATCH_FILE != 0 { ... }

// 改为：
if self.block.flags == 0x00010000 {  // 严格匹配
    Err(Error::new(ErrorKind::Other, "Patch file not supported"))
} else if self.block.flags & FILE_SINGLE_UNIT != 0 {
    ...
}
```

### 方案 B: 使用本地修改版本

1. 克隆 mpq-rust
2. 修改源码
3. 作为本地依赖

```toml
[dependencies]
mpq = { path = "../mpq-rust" }
```

### 方案 C: 完全原生实现（推荐）

优势：
- 完全控制
- 无第三方依赖问题
- 学习价值高
- 已有 70% 代码

## 下一步

决定使用哪个方案并实施。

