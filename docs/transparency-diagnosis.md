# 透明度系统诊断文档

## 问题描述

### 视觉问题
- **C++原版**：桥体清晰可见，与水体分离正确
- **Rust版本**：桥体与周围水体混合不好，桥体没有正确展示
- **Rust渲染异常**：有大量深蓝色/紫色实心区域，形成菱形图案和锯齿状的河流

### 日志发现
- **关键发现**：整个Town地图没有任何`transparency=true`的瓦片输出
- **C++和Rust**：都没有输出`transparency=true`
- **这意味着**：透明度系统没有被触发

---

## 透明度触发条件

### C++逻辑
**文件**: `Source/engine/render/scrollrt.cpp:543`

```cpp
bool transparency = TileHasAny(tilePosition, TileProperties::Transparent) 
                     && TransList[dTransVal[tilePosition.x][tilePosition.y]];
```

**需要两个条件同时满足**：
1. `TileHasAny(pos, Transparent)` - 瓦片在SOL数据中有TRANSPARENT属性
2. `TransList[dTransVal[x][y]]` - TransList中对应索引为true

### Rust逻辑
**文件**: `rust-diablo/src/world/mod.rs:1295`

```rust
fn check_transparency(&self, tile_position: (i32, i32), sol_data: Option<&SolData>) -> bool {
    // 1. Check if tile has TRANSPARENT property in SOL
    let has_transparent_prop = self.tile_has_property(
        tile_position, 
        TileProperties::TRANSPARENT, 
        sol_data
    );
    
    if !has_transparent_prop {
        return false;  // No TRANSPARENT property
    }
    
    // 2. Check TransList using dTransVal index
    let trans_value = self.trans_val.get(tile_position.0, tile_position.1);
    let trans_list_active = trans_value > 0 && self.trans_list.get(trans_value as u8);
    
    trans_list_active
}
```

**逻辑相同**，需要两个条件。

---

## 诊断步骤

### 步骤1: 检查Town SOL数据中的TRANSPARENT属性

**目标**: 找出Town SOL文件中哪些piece_id有TRANSPARENT属性

#### C++端添加诊断日志

**位置**: `Source/levels/gendung.cpp::LoadLevelSOLData()` 后面

```cpp
// After loading SOL data
void DiagnoseTransparentTiles() {
    Log("[C++ SOL DIAGNOSIS] Checking SOL data for TRANSPARENT properties:");
    int transparentCount = 0;
    std::vector<int> transparentPieces;
    
    for (int i = 0; i < MAXTILES; i++) {
        if (HasAnyOf(SOLData[i], TileProperties::Transparent)) {
            transparentCount++;
            transparentPieces.push_back(i);
            
            // Log details for first 20
            if (transparentCount <= 20) {
                Log("  Piece {} has Transparent property (flags={:08x})", i, static_cast<uint32_t>(SOLData[i]));
            }
        }
    }
    
    Log("[C++ SOL DIAGNOSIS] Total pieces with TRANSPARENT: {} out of {}", transparentCount, MAXTILES);
    
    if (transparentCount > 20) {
        std::string summary;
        for (size_t i = 0; i < std::min(size_t(10), transparentPieces.size()); i++) {
            if (!summary.empty()) summary += ", ";
            summary += std::to_string(transparentPieces[i]);
        }
        Log("[C++ SOL DIAGNOSIS] First 10 transparent pieces: {}", summary);
    }
}
```

调用位置：在`LoadLevelSOLData()`返回后调用此函数。

#### Rust端添加诊断日志

**位置**: `rust-diablo/src/tiles/sol.rs` 添加诊断方法

```rust
impl SolData {
    pub fn diagnose_transparent_tiles(&self) {
        use crate::tiles::TileProperties;
        
        println!("[RUST SOL DIAGNOSIS] Checking SOL data for TRANSPARENT properties:");
        let mut transparent_count = 0;
        let mut transparent_pieces = Vec::new();
        
        for (i, props) in self.tiles.iter().enumerate() {
            if props.contains(TileProperties::TRANSPARENT) {
                transparent_count += 1;
                transparent_pieces.push(i);
                
                if transparent_count <= 20 {
                    println!("  Piece {} has Transparent property (flags={:?})", i, props);
                }
            }
        }
        
        println!("[RUST SOL DIAGNOSIS] Total pieces with TRANSPARENT: {} out of {}", 
                 transparent_count, self.tiles.len());
        
        if transparent_count > 20 {
            let summary: Vec<String> = transparent_pieces.iter()
                .take(10)
                .map(|p| p.to_string())
                .collect();
            println!("[RUST SOL DIAGNOSIS] First 10 transparent pieces: {}", summary.join(", "));
        }
    }
}
```

调用位置：在`World::load_town_sector`中加载SOL后调用。

---

### 步骤2: 检查TransVal数据

**目标**: 确认TransVal数组的实际值

#### C++端添加诊断

**位置**: `Source/levels/gendung.cpp::LoadTransparency()` 已有统计

现有输出：
```
[C++ TRANSPARENCY] Loaded transparency data: X tiles, Y non-zero values, Z unique values
[C++ TRANSPARENCY] Unique values: [...]
```

#### Rust端已有输出

```
TransList initialized: N active indices
```

**对比**: 检查两边的非零值数量和唯一值是否一致。

---

### 步骤3: 检查特定瓦片（桥和水）

**目标**: 找出桥和水的piece_id，检查它们的属性和透明度值

#### 增强C++日志

在`DrawCell`中，针对特定piece_id输出详细信息：

```cpp
// After getting levelPieceId
if (levelPieceId == 某个值) {  // 替换为桥/水的piece_id
    int8_t transVal = dTransVal[tilePosition.x][tilePosition.y];
    bool hasProp = TileHasAny(tilePosition, TileProperties::Transparent);
    bool hasTransLeft = TileHasAny(tilePosition, TileProperties::TransparentLeft);
    bool hasTransRight = TileHasAny(tilePosition, TileProperties::TransparentRight);
    bool listActive = TransList[transVal];
    
    Log("[C++ BRIDGE/WATER] piece={} at ({},{}) transVal={} hasProp={} hasLeft={} hasRight={} listActive={} SOLData={:08x}",
        levelPieceId, tilePosition.x, tilePosition.y, 
        transVal, hasProp, hasTransLeft, hasTransRight, listActive,
        static_cast<uint32_t>(SOLData[levelPieceId]));
}
```

#### 增强Rust日志

在`render_micro_tile`中，针对特定piece_id输出详细信息（类似C++）。

---

### 步骤4: 对比dPiece数组

**目标**: 确认两边加载的地图数据完全一致

```bash
# 已有输出文件
diff dpiece_cpp.txt dpiece_rust.txt
```

如果不一致，说明地图加载有问题。

---

## 可能的问题假设

### 假设1: Town SOL没有TRANSPARENT属性
**现象**: SOL诊断输出显示0个或很少的piece有TRANSPARENT属性

**原因**: Town地图可能不需要透明度系统（Town是室外，没有地下城的透明墙体）

**解决**: 
- 如果Town确实不需要透明度，那这是正常的
- 桥的渲染问题可能与透明度无关，需要检查其他渲染逻辑
- 需要测试地下城地图（Cathedral/Catacombs）来验证透明度系统

### 假设2: TransVal全是0
**现象**: 
```
[C++ TRANSPARENCY] Loaded transparency data: X tiles, 0 non-zero values, 0 unique values
TransList initialized: 0 active indices
```

**原因**: Town DUN文件的透明层全是0（没有透明度区域）

**解决**: 同假设1，需要测试地下城地图

### 假设3: 桥渲染问题与透明度无关
**现象**: 桥的piece_id在SOL中没有TRANSPARENT属性

**原因**: 桥可能使用其他渲染逻辑（如特殊图层、Z-order等）

**解决**: 需要检查：
- 桥的piece_id具体是什么
- 桥使用什么TileType (LeftTriangle, RightTriangle, TransparentSquare?)
- 桥是floor还是wall
- 桥的渲染顺序（Phase 1 floor还是Phase 2 wall）

### 假设4: Rust渲染的深蓝色区域是什么
**现象**: 大量深蓝色/紫色实心区域，形成菱形图案

**可能原因**:
1. **未渲染的瓦片**: 某些piece_id没有纹理数据，显示默认颜色
2. **解码失败**: CL2解码失败，显示默认颜色
3. **mask应用错误**: MaskType应用不正确，导致整个瓦片被遮罩
4. **光照问题**: 光照值为0，显示全黑/全蓝

**诊断**: 
- 在render_micro_tile中增加更多日志
- 检查哪些piece_id显示为深蓝色
- 检查这些piece的解码是否成功
- 检查光照值

---

## 下一步行动计划

### 阶段1: 数据验证（当前阶段）
1. ✅ 添加SOL诊断日志（C++和Rust）
2. ✅ 运行并对比SOL诊断输出
3. ✅ 对比TransVal统计数据
4. ✅ 对比dPiece数组

### 阶段2: 特定瓦片分析
1. 从截图中识别桥的大致位置
2. 从dPiece数组中找出桥的piece_id
3. 检查桥的SOL属性、TransVal值
4. 对比桥在C++和Rust的渲染参数

### 阶段3: 深蓝色区域诊断
1. 识别深蓝色区域的piece_id
2. 检查这些piece的纹理是否存在
3. 检查解码是否成功
4. 检查mask和光照是否正确

### 阶段4: 测试地下城地图
1. 切换到Cathedral (L1) 地图
2. 验证透明度系统在地下城是否工作
3. 对比C++和Rust的透明墙体渲染

---

## 实施优先级

**P0 (立即)**: 
- 添加SOL诊断日志
- 检查Town SOL是否有TRANSPARENT属性

**P1 (如果Town无透明度)**:
- 分析桥渲染问题（与透明度无关）
- 诊断深蓝色区域问题

**P2 (验证透明度系统)**:
- 测试地下城地图
- 验证透明墙体渲染

---

## 参考代码位置

### C++
- `Source/levels/gendung.cpp` - SOL加载和透明度加载
- `Source/engine/render/scrollrt.cpp` - 透明度检查和渲染

### Rust
- `rust-diablo/src/tiles/sol.rs` - SOL数据结构
- `rust-diablo/src/world/transparency.rs` - 透明度系统
- `rust-diablo/src/world/mod.rs` - 透明度检查和渲染

---

**创建日期**: 2024-12-04  
**目的**: 系统诊断透明度系统和桥渲染问题




