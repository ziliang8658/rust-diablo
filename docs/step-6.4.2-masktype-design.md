# Step 6.4.2 MaskType透明度掩码系统 - 详细设计文档

## 文档信息
- **创建时间**: 2024-12-04
- **版本**: v1.0
- **状态**: 设计阶段
- **前置**: Step 6.4.1 光照系统（已完成）
- **后续**: Step 6.4.3 八方向动画

---

## 1. 功能概述

### 1.1 目标
实现MaskType透明度掩码系统，使不同地形（河流、土路、草地等）能够平滑混合，而不是硬边缘切换。

### 1.2 当前问题
- **硬边缘切换**: 不同地形之间是直接的硬边缘，没有过渡
- **视觉突兀**: 河流与土路、草地与石板等交界处非常明显
- **缺少混合**: 没有实现透明度混合机制

### 1.3 预期效果
- **平滑过渡**: 地形之间使用透明度渐变混合
- **自然融合**: 河流边缘与土路自然融合
- **匹配原版**: 达到原版Diablo的地形混合效果

---

## 2. 原版实现深度分析

### 2.1 MaskType枚举定义

**参考**: `Source/engine/render/dun_render.hpp` 第28-90行

```cpp
enum class MaskType : uint8_t {
    Solid,       // 完全不透明 - 所有像素直接绘制
    Transparent, // 完全透明 - 所有像素使用alpha混合
    Left,        // 左上三角透明混合
    Right,       // 右上三角透明混合
};
```

#### MaskType::Left 掩码图案
```
下半部分（16行）：完全不透明
上半部分（16行）：从左到右渐变（🮆=不透明, 🮐=透明混合）

🮐🮐🮐...🮐🮐🮐🮐🮐🮐🮆🮆  ← 顶部（2个不透明像素）
🮐🮐🮐...🮐🮐🮐🮐🮆🮆🮆🮆  ← +2个不透明
🮐🮐🮐...🮐🮐🮆🮆🮆🮆🮆🮆  ← +2个不透明
...
🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆...🮆🮆  ← 底部（完全不透明）
```

#### MaskType::Right 掩码图案
```
下半部分（16行）：完全不透明
上半部分（16行）：从右到左渐变

🮆🮆🮐🮐🮐...🮐🮐🮐🮐🮐🮐  ← 顶部（2个不透明像素）
🮆🮆🮆🮆🮐🮐...🮐🮐🮐🮐🮐🮐  ← +2个不透明
🮆🮆🮆🮆🮆🮆...🮐🮐🮐🮐🮐🮐  ← +2个不透明
...
🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆🮆...🮆🮆  ← 底部（完全不透明）
```

### 2.2 掩码算法原理

**参考**: `Source/engine/render/dun_render.cpp` 第93-109行

#### 核心参数
```cpp
template <MaskType Mask>
constexpr int8_t PrefixIncrement = 0;

template <>
constexpr int8_t PrefixIncrement<MaskType::Left> = 2;   // 每行增加2个不透明像素

template <>
constexpr int8_t PrefixIncrement<MaskType::Right> = -2; // 每行减少2个不透明像素

template <MaskType Mask>
int8_t InitialPrefix = PrefixIncrement<Mask> >= 0 ? -32 : 64;
```

#### 算法说明
- **渲染方向**: 从底部到顶部（bottom-to-top）
- **prefix**: 表示当前行有多少像素是不透明的
- **Left掩码**:
  - 初始prefix = -32（底部以下，即全透明）
  - 每行prefix += 2（逐渐增加不透明部分）
  - 到第16行时：prefix = -32 + 2*16 = 0（开始有不透明像素）
  - 到第32行时：prefix = 32（完全不透明）
- **Right掩码**:
  - 初始prefix = 64（超出宽度，即全不透明）
  - 每行prefix -= 2（逐渐减少不透明部分）
  - 到第16行时：prefix = 64 - 2*16 = 32（仍然完全不透明）
  - 到第32行时：prefix = 0（开始出现透明部分）

### 2.3 MaskType选择逻辑

**参考**: `Source/engine/render/scrollrt.cpp` 第547-579行

#### 前置条件检查
```cpp
// 1. 检查是否启用transparency
bool transparency = TileHasAny(tilePosition, TileProperties::Transparent) 
                    && TransList[dTransVal[tilePosition.x][tilePosition.y]];

// 如果transparency=false，所有tile都使用MaskType::Solid
```

#### Block 0 (左半部分) 掩码选择
```cpp
const auto getFirstTileMaskLeft = [=](TileType tile) -> MaskType {
    if (transparency) {
        switch (tile) {
        case TileType::LeftTrapezoid:      // 左梯形
        case TileType::TransparentSquare:  // 透明方块
            return TileHasAny(tilePosition, TileProperties::TransparentLeft)
                ? MaskType::Left      // 如果有TransparentLeft属性，使用Left掩码
                : MaskType::Solid;    // 否则不透明
        case TileType::LeftTriangle:       // 左三角（floor）
            return MaskType::Solid;        // 永远不透明
        default:
            return MaskType::Transparent;  // 其他类型完全透明
        }
    }
    return MaskType::Solid;  // transparency关闭时，全部不透明
};
```

#### Block 1 (右半部分) 掩码选择
```cpp
const auto getFirstTileMaskRight = [=](TileType tile) -> MaskType {
    if (transparency) {
        switch (tile) {
        case TileType::RightTrapezoid:     // 右梯形
        case TileType::TransparentSquare:  // 透明方块
            return TileHasAny(tilePosition, TileProperties::TransparentRight)
                ? MaskType::Right     // 如果有TransparentRight属性，使用Right掩码
                : MaskType::Solid;    // 否则不透明
        case TileType::RightTriangle:      // 右三角（floor）
            return MaskType::Solid;        // 永远不透明
        default:
            return MaskType::Transparent;  // 其他类型完全透明
        }
    }
    return MaskType::Solid;  // transparency关闭时，全部不透明
};
```

#### Blocks 2+ 掩码选择
```cpp
// 墙壁块（blocks 2及以上）使用简化逻辑
MaskType mask = transparency ? MaskType::Transparent : MaskType::Solid;
```

### 2.4 渲染函数调用

**参考**: `Source/engine/render/scrollrt.cpp` 第595-608行

```cpp
// Block 0
RenderTile(out, lightmap, targetBufferPosition,
    pDungeonCels.get(), levelCelBlock, 
    getFirstTileMaskLeft(tileType),  // 动态获取掩码类型
    tbl);

// Block 1
RenderTile(out, lightmap, targetBufferPosition + RightFrameDisplacement,
    pDungeonCels.get(), levelCelBlock, 
    getFirstTileMaskRight(tileType),  // 动态获取掩码类型
    tbl);
```

### 2.5 逐行渲染实现

**参考**: `Source/engine/render/dun_render.cpp` 第216-237行

```cpp
template <LightType Light, MaskType Mask>
void RenderLine(uint8_t *dst, const uint8_t *src, uint_fast8_t n, 
                const uint8_t *tbl, const Lightmap &lightmap, int8_t prefix)
{
    if constexpr (Mask == MaskType::Solid || Mask == MaskType::Transparent) {
        // 简单情况：全不透明或全透明
        RenderLineTransparentOrOpaque<Light, Mask == MaskType::Transparent>(dst, src, n, tbl, &lightmap);
        
    } else if (prefix >= static_cast<int8_t>(n)) {
        // prefix超出行宽：整行处理
        if constexpr (Mask == MaskType::Right) {
            RenderLineOpaque<Light>(dst, src, n, tbl, &lightmap);
        } else {
            RenderLineTransparent<Light>(dst, src, n, tbl, &lightmap);
        }
        
    } else if (prefix <= 0) {
        // prefix为负或零：整行处理
        if constexpr (Mask == MaskType::Left) {
            RenderLineOpaque<Light>(dst, src, n, tbl, &lightmap);
        } else {
            RenderLineTransparent<Light>(dst, src, n, tbl, &lightmap);
        }
        
    } else {
        // 正常情况：分两部分渲染
        RenderLineTransparentAndOpaque<Light, 
            /*OpaqueFirst=*/Mask == MaskType::Right>(dst, src, prefix, n, tbl, lightmap);
    }
}
```

#### 分段渲染函数
```cpp
template <LightType Light, bool OpaqueFirst>
void RenderLineTransparentAndOpaque(uint8_t *dst, const uint8_t *src, 
                                     uint_fast8_t prefixWidth, uint_fast8_t width,
                                     const uint8_t *tbl, const Lightmap &lightmap)
{
    if constexpr (OpaqueFirst) {
        // Right掩码：先不透明，后透明
        RenderLineOpaque<Light>(dst, src, prefixWidth, tbl, &lightmap);
        RenderLineTransparent<Light>(dst + prefixWidth, src + prefixWidth, 
                                      width - prefixWidth, tbl, &lightmap);
    } else {
        // Left掩码：先透明，后不透明
        RenderLineTransparent<Light>(dst, src, prefixWidth, tbl, &lightmap);
        RenderLineOpaque<Light>(dst + prefixWidth, src + prefixWidth, 
                                 width - prefixWidth, tbl, &lightmap);
    }
}
```

---

## 3. Rust实现设计

### 3.1 数据结构

#### 3.1.1 MaskType枚举
**文件**: `rust-diablo/src/tiles/types.rs`

```rust
/// 瓦片透明度掩码类型
///
/// 用于实现不同地形的平滑混合效果
///
/// # Reference
/// Original: Source/engine/render/dun_render.hpp Line 28-90
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaskType {
    /// 完全不透明 - 所有像素直接绘制
    Solid,
    
    /// 完全透明 - 所有像素使用alpha混合
    Transparent,
    
    /// 右上三角透明混合
    /// 
    /// 用于RightTrapezoid和TransparentSquare类型
    /// - 下半部分（16行）：完全不透明
    /// - 上半部分（16行）：从右到左渐变透明
    Right,
    
    /// 左上三角透明混合
    /// 
    /// 用于LeftTrapezoid和TransparentSquare类型
    /// - 下半部分（16行）：完全不透明
    /// - 上半部分（16行）：从左到右渐变透明
    Left,
}

impl MaskType {
    /// 获取prefix增量
    /// 
    /// - Left: +2 (每行增加2个不透明像素)
    /// - Right: -2 (每行减少2个不透明像素)
    /// - Solid/Transparent: 0 (不使用prefix)
    pub const fn prefix_increment(self) -> i8 {
        match self {
            MaskType::Left => 2,
            MaskType::Right => -2,
            _ => 0,
        }
    }
    
    /// 获取初始prefix值
    /// 
    /// - Left: -32 (从完全透明开始)
    /// - Right: 64 (从完全不透明开始)
    pub const fn initial_prefix(self) -> i8 {
        match self {
            MaskType::Left => -32,
            MaskType::Right => 64,
            _ => 0,
        }
    }
    
    /// 计算指定行的prefix值（从底部开始计数）
    pub const fn get_prefix_for_row(self, row_from_bottom: i8) -> i8 {
        self.initial_prefix() + self.prefix_increment() * row_from_bottom
    }
}
```

#### 3.1.2 扩展TileProperties
**文件**: `rust-diablo/src/tiles/types.rs`

```rust
bitflags! {
    pub struct TileProperties: u8 {
        const NONE = 0;
        const SOLID = 1 << 0;
        const BLOCK_MISSILE = 1 << 1;
        const BLOCK_LIGHT = 1 << 2;
        const TRANSPARENT = 1 << 3;        // 新增：启用透明度混合
        const TRANSPARENT_LEFT = 1 << 4;   // 新增：左侧透明
        const TRANSPARENT_RIGHT = 1 << 5;  // 新增：右侧透明
        const TRAP = 1 << 6;
    }
}
```

### 3.2 核心函数

#### 3.2.1 MaskType选择函数
**文件**: `rust-diablo/src/world/mod.rs`

```rust
impl World {
    /// 获取Block 0（左半部分）的MaskType
    /// 
    /// # Reference
    /// Original: Source/engine/render/scrollrt.cpp Line 547-562
    fn get_mask_type_left(
        &self,
        tile_position: (i32, i32),
        tile_type: TileType,
        sol_data: Option<&SOLData>,
    ) -> MaskType {
        // 检查是否启用transparency
        let transparency = self.check_transparency(tile_position, sol_data);
        
        if !transparency {
            return MaskType::Solid;
        }
        
        match tile_type {
            TileType::LeftTrapezoid | TileType::TransparentSquare => {
                // 检查TransparentLeft属性
                if self.tile_has_property(tile_position, TileProperties::TRANSPARENT_LEFT, sol_data) {
                    MaskType::Left
                } else {
                    MaskType::Solid
                }
            }
            TileType::LeftTriangle => MaskType::Solid,  // floor永远不透明
            _ => MaskType::Transparent,  // 其他类型完全透明
        }
    }
    
    /// 获取Block 1（右半部分）的MaskType
    /// 
    /// # Reference
    /// Original: Source/engine/render/scrollrt.cpp Line 564-579
    fn get_mask_type_right(
        &self,
        tile_position: (i32, i32),
        tile_type: TileType,
        sol_data: Option<&SOLData>,
    ) -> MaskType {
        let transparency = self.check_transparency(tile_position, sol_data);
        
        if !transparency {
            return MaskType::Solid;
        }
        
        match tile_type {
            TileType::RightTrapezoid | TileType::TransparentSquare => {
                if self.tile_has_property(tile_position, TileProperties::TRANSPARENT_RIGHT, sol_data) {
                    MaskType::Right
                } else {
                    MaskType::Solid
                }
            }
            TileType::RightTriangle => MaskType::Solid,
            _ => MaskType::Transparent,
        }
    }
    
    /// 检查tile是否启用transparency
    /// 
    /// # Reference
    /// Original: Source/engine/render/scrollrt.cpp Line 540
    fn check_transparency(
        &self,
        tile_position: (i32, i32),
        sol_data: Option<&SOLData>,
    ) -> bool {
        // TODO: 实现TransList检查
        // 暂时简化为只检查TileProperties::TRANSPARENT
        self.tile_has_property(tile_position, TileProperties::TRANSPARENT, sol_data)
    }
    
    /// 检查tile是否有指定属性
    fn tile_has_property(
        &self,
        tile_position: (i32, i32),
        property: TileProperties,
        sol_data: Option<&SOLData>,
    ) -> bool {
        if let Some(sol) = sol_data {
            if let Some(dungeon_map) = self.dungeon_map.as_ref() {
                if dungeon_map.in_bounds(tile_position.0, tile_position.1) {
                    let piece_id = dungeon_map.get_piece(tile_position.0, tile_position.1);
                    if let Some(props) = sol.get(piece_id as usize) {
                        return props.contains(property);
                    }
                }
            }
        }
        false
    }
}
```

#### 3.2.2 掩码应用函数
**文件**: `rust-diablo/src/engine/mod.rs`

```rust
impl Engine {
    /// 对RGBA数据应用透明度掩码
    /// 
    /// # Reference
    /// Original: Source/engine/render/dun_render.cpp Line 216-237
    fn apply_mask_to_rgba(
        rgba_data: &mut [u8],
        width: u32,
        height: u32,
        mask_type: MaskType,
    ) {
        match mask_type {
            MaskType::Solid => {
                // 不做任何处理，保持所有像素不透明
            }
            MaskType::Transparent => {
                // 将所有非透明像素的alpha设置为128（50%透明）
                for pixel in rgba_data.chunks_exact_mut(4) {
                    if pixel[3] > 0 {
                        pixel[3] = 128;  // 半透明
                    }
                }
            }
            MaskType::Left => {
                Self::apply_left_mask(rgba_data, width, height);
            }
            MaskType::Right => {
                Self::apply_right_mask(rgba_data, width, height);
            }
        }
    }
    
    /// 应用Left掩码（左上三角透明）
    fn apply_left_mask(rgba_data: &mut [u8], width: u32, height: u32) {
        // 从底部到顶部处理
        for y in 0..height {
            let row_from_bottom = (height - 1 - y) as i8;
            let prefix = MaskType::Left.get_prefix_for_row(row_from_bottom);
            
            // 行起始位置
            let row_start = (y * width * 4) as usize;
            
            if prefix <= 0 {
                // 整行不透明（前16行）
                continue;
            } else if prefix >= width as i8 {
                // 整行透明（不应该发生，除非配置错误）
                for x in 0..width as usize {
                    let pixel_idx = row_start + x * 4;
                    if rgba_data[pixel_idx + 3] > 0 {
                        rgba_data[pixel_idx + 3] = 128;  // 半透明
                    }
                }
            } else {
                // 部分透明（前prefix个像素透明，后面不透明）
                for x in 0..(prefix as usize).min(width as usize) {
                    let pixel_idx = row_start + x * 4;
                    if rgba_data[pixel_idx + 3] > 0 {
                        rgba_data[pixel_idx + 3] = 128;  // 半透明
                    }
                }
            }
        }
    }
    
    /// 应用Right掩码（右上三角透明）
    fn apply_right_mask(rgba_data: &mut [u8], width: u32, height: u32) {
        for y in 0..height {
            let row_from_bottom = (height - 1 - y) as i8;
            let prefix = MaskType::Right.get_prefix_for_row(row_from_bottom);
            
            let row_start = (y * width * 4) as usize;
            
            if prefix >= width as i8 {
                // 整行不透明（前16行）
                continue;
            } else if prefix <= 0 {
                // 整行透明
                for x in 0..width as usize {
                    let pixel_idx = row_start + x * 4;
                    if rgba_data[pixel_idx + 3] > 0 {
                        rgba_data[pixel_idx + 3] = 128;
                    }
                }
            } else {
                // 部分透明（前prefix个像素不透明，后面透明）
                for x in (prefix as usize)..width as usize {
                    let pixel_idx = row_start + x * 4;
                    if rgba_data[pixel_idx + 3] > 0 {
                        rgba_data[pixel_idx + 3] = 128;
                    }
                }
            }
        }
    }
}
```

#### 3.2.3 修改渲染函数签名
**文件**: `rust-diablo/src/engine/mod.rs`

```rust
pub fn draw_rgba_texture(
    &mut self,
    _texture_id: &str,
    rgba_data: &[u8],
    width: u32,
    height: u32,
    dst_rect: Rect,
    flip_h: bool,
    mask_type: MaskType,  // 新增参数
) -> Result<bool> {
    // 复制数据以便应用掩码
    let mut rgba_data_mut = rgba_data.to_vec();
    
    // 应用掩码
    Self::apply_mask_to_rgba(&mut rgba_data_mut, width, height, mask_type);
    
    // ... 现有的纹理创建和渲染代码 ...
}
```

---

## 4. 实施计划

### 4.1 阶段划分

#### 阶段1：基础结构 (2-3小时)
**目标**: 创建MaskType类型和基础函数

**任务**:
1. ✅ 在`tiles/types.rs`添加MaskType枚举
2. ✅ 实现MaskType的辅助方法（prefix_increment等）
3. ✅ 扩展TileProperties添加透明度标志
4. ✅ 在`tiles/mod.rs`导出MaskType
5. ✅ 编译测试

**验收标准**:
- MaskType枚举定义正确
- prefix计算函数测试通过
- 代码编译无错误

#### 阶段2：掩码选择逻辑 (3-4小时)
**目标**: 实现动态MaskType选择

**任务**:
1. ✅ 在World中添加`get_mask_type_left()`
2. ✅ 在World中添加`get_mask_type_right()`
3. ✅ 实现`check_transparency()`
4. ✅ 实现`tile_has_property()`
5. ✅ 添加调试打印，输出MaskType选择结果
6. ✅ 测试不同TileType的掩码选择

**验收标准**:
- 不同tile类型返回正确的MaskType
- 调试输出显示掩码选择逻辑
- 代码编译无错误

#### 阶段3：掩码应用（简化版） (3-4小时)
**目标**: 实现基本的掩码渲染

**任务**:
1. ✅ 实现`apply_mask_to_rgba()`框架
2. ✅ 实现MaskType::Solid处理（无操作）
3. ✅ 实现MaskType::Transparent处理（全部半透明）
4. ✅ 实现简化的Left掩码（暂不考虑prefix）
5. ✅ 实现简化的Right掩码（暂不考虑prefix）
6. ✅ 修改`draw_rgba_texture()`签名
7. ✅ 测试基本效果

**验收标准**:
- Solid瓦片正常显示
- Transparent瓦片有透明效果
- Left/Right掩码有初步效果
- 代码编译运行

#### 阶段4：完整prefix算法 (4-5小时)
**目标**: 实现完整的渐变掩码

**任务**:
1. ✅ 实现完整的`apply_left_mask()`
2. ✅ 实现完整的`apply_right_mask()`
3. ✅ 优化掩码计算性能
4. ✅ 添加边界检查和错误处理
5. ✅ 测试不同高度的瓦片
6. ✅ 验证渐变效果

**验收标准**:
- Left掩码显示正确的左上到右下渐变
- Right掩码显示正确的右上到左下渐变
- 32x32和32x31瓦片都正确处理
- 性能可接受

#### 阶段5：集成到渲染流程 (3-4小时)
**目标**: 更新所有渲染调用点

**任务**:
1. ✅ 修改`render_micro_tile()`添加MaskType参数
2. ✅ 在`draw_floor_at()`中传递MaskType::Solid
3. ✅ 在`draw_cell_at()`中获取并传递正确的MaskType
4. ✅ 在`render_floor_foliage()`中传递MaskType::Solid
5. ✅ 更新blocks 2+的渲染逻辑
6. ✅ 全面测试

**验收标准**:
- 所有渲染调用编译通过
- Floor瓦片正常显示
- Wall瓦片使用正确的掩码
- 地形混合效果可见

#### 阶段6：测试和优化 (3-4小时)
**目标**: 完善功能和性能

**任务**:
1. ✅ 测试不同地形组合
2. ✅ 调整半透明alpha值（可能需要不同的值）
3. ✅ 性能profiling
4. ✅ 添加可选的掩码可视化（调试用）
5. ✅ 文档化踩坑点
6. ✅ 编写总结文档

**验收标准**:
- 河流与土路平滑混合
- 草地与石板平滑混合
- 性能无明显下降
- 调试工具可用

### 4.2 时间估算
- **阶段1**: 2-3小时
- **阶段2**: 3-4小时
- **阶段3**: 3-4小时
- **阶段4**: 4-5小时
- **阶段5**: 3-4小时
- **阶段6**: 3-4小时
- **总计**: 18-24小时

### 4.3 里程碑
- **M1** (阶段1完成): MaskType基础结构就绪
- **M2** (阶段2完成): 掩码选择逻辑可用
- **M3** (阶段4完成): 掩码渲染核心完成
- **M4** (阶段6完成): 功能完整，可发布

---

## 5. 技术要点

### 5.1 关键算法

#### 5.1.1 Prefix计算
```rust
// Left掩码：从-32开始，每行+2
// Row 0 (底部): prefix = -32 + 2*0 = -32 (整行不透明)
// Row 15:        prefix = -32 + 2*15 = -2 (整行不透明)
// Row 16:        prefix = -32 + 2*16 = 0 (开始出现透明)
// Row 31 (顶部): prefix = -32 + 2*31 = 30 (30像素透明，2像素不透明)

// Right掩码：从64开始，每行-2
// Row 0 (底部): prefix = 64 - 2*0 = 64 (整行不透明)
// Row 15:        prefix = 64 - 2*15 = 34 (整行不透明)
// Row 16:        prefix = 64 - 2*16 = 32 (刚好全不透明)
// Row 31 (顶部): prefix = 64 - 2*31 = 2 (2像素不透明，30像素透明)
```

#### 5.1.2 Alpha混合策略
原版使用palette lookup table实现透明混合，Rust实现有两种选择：

**选项A**: 使用SDL2 alpha混合（推荐）
- 设置BlendMode::Blend
- 修改alpha通道值
- 简单高效

**选项B**: 手动实现palette混合
- 需要paletteTransparencyLookup表
- 更接近原版
- 实现复杂

**建议**: 先用选项A实现，如果效果不佳再考虑选项B

### 5.2 性能考虑

#### 5.2.1 掩码应用时机
```rust
// 方案1：在CPU端应用掩码（当前设计）
// - 优点：实现简单，易于调试
// - 缺点：每帧都要修改RGBA数据

// 方案2：缓存应用掩码后的纹理
// - 优点：性能更好
// - 缺点：内存占用增加

// 建议：先实现方案1，如果性能不足再优化为方案2
```

#### 5.2.2 优化策略
1. **Early exit**: Solid掩码直接跳过处理
2. **批处理**: 整行透明/不透明时批量处理
3. **SIMD**: 考虑使用SIMD加速alpha修改（高级优化）

### 5.3 踩坑预警

#### 5.3.1 坐标系统
- **渲染方向**: 原版从底到顶，Rust可能需要转换
- **Y轴**: 注意SDL的Y轴方向（top-to-bottom）

#### 5.3.2 Alpha值选择
- **半透明度**: 原版使用lookup table，Rust使用固定alpha
- **建议值**: 128 (50%), 可能需要调整到96-160之间

#### 5.3.3 Tile尺寸
- **Triangle**: 32x31
- **Square**: 32x32
- **Foliage**: 32x16
- prefix计算需要适配不同高度

---

## 6. 测试方案

### 6.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mask_type_prefix_left() {
        assert_eq!(MaskType::Left.initial_prefix(), -32);
        assert_eq!(MaskType::Left.prefix_increment(), 2);
        assert_eq!(MaskType::Left.get_prefix_for_row(0), -32);
        assert_eq!(MaskType::Left.get_prefix_for_row(16), 0);
        assert_eq!(MaskType::Left.get_prefix_for_row(31), 30);
    }
    
    #[test]
    fn test_mask_type_prefix_right() {
        assert_eq!(MaskType::Right.initial_prefix(), 64);
        assert_eq!(MaskType::Right.prefix_increment(), -2);
        assert_eq!(MaskType::Right.get_prefix_for_row(0), 64);
        assert_eq!(MaskType::Right.get_prefix_for_row(16), 32);
        assert_eq!(MaskType::Right.get_prefix_for_row(31), 2);
    }
    
    #[test]
    fn test_apply_solid_mask() {
        let mut rgba = vec![255, 0, 0, 255]; // Red pixel
        Engine::apply_mask_to_rgba(&mut rgba, 1, 1, MaskType::Solid);
        assert_eq!(rgba[3], 255); // Alpha unchanged
    }
    
    #[test]
    fn test_apply_transparent_mask() {
        let mut rgba = vec![255, 0, 0, 255]; // Red pixel
        Engine::apply_mask_to_rgba(&mut rgba, 1, 1, MaskType::Transparent);
        assert_eq!(rgba[3], 128); // Alpha set to 128
    }
}
```

### 6.2 集成测试

**测试用例**:
1. **纯floor**: 所有瓦片MaskType::Solid
2. **河流边缘**: Left/Right掩码混合
3. **草地石板**: Transparent掩码测试
4. **复杂场景**: 多种掩码组合

**测试方法**:
1. 加载测试地图
2. 截图对比
3. 手动验证混合效果
4. 性能benchmark

### 6.3 调试工具

**可视化MaskType**:
```rust
fn visualize_mask_type(mask_type: MaskType) -> Color {
    match mask_type {
        MaskType::Solid => Color::new(0, 255, 0),       // 绿色
        MaskType::Transparent => Color::new(255, 0, 0), // 红色
        MaskType::Left => Color::new(0, 0, 255),        // 蓝色
        MaskType::Right => Color::new(255, 255, 0),     // 黄色
    }
}
```

---

## 7. 参考资料

### 7.1 原版代码文件
- `Source/engine/render/dun_render.hpp` - MaskType定义和接口
- `Source/engine/render/dun_render.cpp` - 掩码渲染实现
- `Source/engine/render/scrollrt.cpp` - MaskType选择逻辑
- `Source/levels/dun_tile.hpp` - TileType和TileProperties定义

### 7.2 关键函数
- `RenderTileFrame()` - 低级瓦片渲染
- `RenderLine()` - 逐行渲染with掩码
- `getFirstTileMaskLeft()` - Left掩码选择
- `getFirstTileMaskRight()` - Right掩码选择

### 7.3 相关常量
- `TILE_WIDTH = 32` - 瓦片宽度
- `TILE_HEIGHT = 32` - 方块高度
- `DunFrameTriangleHeight = 31` - 三角形高度

---

## 8. 下一步计划

### 8.1 Step 6.4.3 - 八方向动画
实现玩家和怪物的八方向动画系统

### 8.2 Step 6.5 - 完整渲染流程
整合所有渲染功能，优化性能

### 8.3 Step 7.0 - 游戏逻辑
开始实现游戏玩法逻辑

---

## 9. 更新日志

| 日期 | 版本 | 更新内容 |
|------|------|----------|
| 2024-12-04 | v1.0 | 初始设计文档，完成原版调研和实施计划 |

---

**文档结束**




