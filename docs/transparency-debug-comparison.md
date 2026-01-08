# TransList透明度系统 - Rust vs C++ 调试对比指南

## 问题分析

### 问题1: Rust缺少transparency字段 ✅ 已修复

**修复**: 在 `render_micro_tile()` 调用 `check_transparency()` 并输出

### 问题2: micro坐标不一致 - 正常现象

## 为什么micro坐标不同？

**Rust输出**:
```
[RUST] Tile piece=84 block=0 at screen=(32,208) micro=(26,42) ...
[RUST] Tile piece=84 block=1 at screen=(64,208) micro=(27,42) ...
```

**C++输出**:
```
[C++] Tile piece=84 block=1 at screen=(746,607) micro=(44,48) ...
[C++] Tile piece=460 block=0 at screen=(778,607) micro=(45,47) ...
```

### 原因分析

1. **屏幕坐标完全不同**
   - Rust: `screen=(32,208)`
   - C++: `screen=(746,607)`
   - **结论**: 两者渲染的是不同位置的tile

2. **视口/摄像机位置不同**
   - Rust和C++的摄像机中心可能不同
   - 玩家位置可能不同
   - 窗口大小可能不同

3. **这是正常的！**
   - 我们不需要两边渲染相同的tile
   - 我们只需要验证**同一个tile的渲染逻辑一致**

---

## 如何正确对比

### 方法1: 对比相同的tile (推荐)

**找到一个共同的tile**，比如 `piece=84`:

**Rust**:
```
piece=84 block=0 micro=(26,42) type=LeftTriangle mask=Solid transparency=false
piece=84 block=1 micro=(27,42) type=RightTriangle mask=Solid transparency=false
```

**C++**:
```
piece=84 block=1 micro=(44,48) type=RightTriangle mask=Solid transparency=false
```

**对比重点**:
- ✅ 相同piece的tile_type一致（RightTriangle）
- ✅ 相同piece的mask一致（Solid）
- ✅ 相同piece的transparency一致（false）
- ✅ micro坐标不同是正常的（不同位置的tile）

### 方法2: 对比透明度数据加载

**Rust启动日志**:
```
✓ Loaded transparency data
TransList initialized: N active indices
Active transparency indices: [...]
```

**C++启动日志**:
```
[C++ TRANSPARENCY] Loaded transparency data: ... N non-zero values
[C++ TRANSPARENCY] Unique values: [...]
```

**对比**:
- [ ] active indices数量是否一致？
- [ ] unique values列表是否一致？

### 方法3: 对比特定位置的transparency值

**如果想对比同一位置**，需要：

1. **在Rust中打印玩家位置**:
```rust
println!("[RUST] Player at world=({}, {})", player.world_x, player.world_y);
```

2. **在C++中打印玩家位置**（已有）:
```cpp
LogVerbose("Player at world=({}, {})", MyPlayer->position.tile.x, MyPlayer->position.tile.y);
```

3. **手动移动到相同位置，再对比输出**

---

## 预期结果

### 场景A: Town透明度全为0（当前情况）

**Rust**:
```
TransList initialized: 0 active indices
[RUST] Tile piece=X ... transparency=false
```

**C++**:
```
[C++ TRANSPARENCY] 0 non-zero values
[C++] Tile piece=X ... transparency=false
```

**结论**: ✅ 两边一致，Town确实没有透明度数据

### 场景B: 有透明度数据

**Rust**:
```
TransList initialized: 5 active indices
Active transparency indices: [1, 2, 3, 4, 5]
[TRANSPARENCY] Active at (x, y): trans_val=3, prop=true, list=true
[MASK_LEFT] (x, y): tile_type=TransparentSquare, mask=Left
```

**C++**:
```
[C++ TRANSPARENCY] 100 non-zero values, 5 unique values
[C++ TRANSPARENCY] Unique values: [1, 2, 3, 4, 5]
[C++ TRANSPARENCY] Active at (x, y): trans_val=3, prop=1, list=1
[C++ MASK_LEFT] (x, y): tile_type=3, mask=1
```

**对比**:
- [ ] unique values是否一致
- [ ] 相同tile的trans_val是否一致
- [ ] 相同条件下的mask选择是否一致

---

## 快速验证清单

### 加载阶段
- [ ] Rust: TransList initialized数量
- [ ] C++: non-zero values数量
- [ ] 两者是否相等？

### 运行时透明度检查
- [ ] 是否有 `[TRANSPARENCY]` 日志？
- [ ] 如果有，trans_val值是否合理（1-255）？
- [ ] Rust和C++的trans_val列表是否一致？

### Mask选择逻辑
- [ ] 是否有 `[MASK_LEFT/RIGHT]` 日志？
- [ ] 如果有，是在transparency=true时触发吗？
- [ ] Rust和C++选择的MaskType是否一致？

### 渲染结果
- [ ] 相同piece的tile渲染参数是否一致？
- [ ] transparency=false时，mask都是Solid吗？
- [ ] transparency=true时，mask是否动态选择？

---

## 总结

**micro坐标不同是正常的**，因为：
1. 两边渲染的视口位置不同
2. 我们关注的是**逻辑一致性**，不是**渲染相同画面**

**真正要对比的是**:
1. ✅ 透明度数据加载统计（active indices数量）
2. ✅ 相同piece的tile_type、mask、transparency
3. ✅ 透明度检查逻辑（trans_val → trans_list → transparency）
4. ✅ Mask选择逻辑（tile_type + properties → mask）

**当前状态** (基于输出):
- ✅ transparency=false（符合Town无透明度的预期）
- ✅ mask=Solid（符合transparency=false的逻辑）
- ✅ Rust和C++行为一致

---

## 下一步建议

1. **检查加载统计**: 确认Rust和C++的active indices都是0
2. **测试Dungeon关卡**: Town可能没有透明度，试试Cathedral
3. **手动设置测试**: 修改DUN文件或手动设置TransList来测试透明度效果

**如果Town确实没有透明度数据，这个实现就是完全正确的！** ✅




