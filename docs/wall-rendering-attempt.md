# 墙体渲染尝试记录

## 日期
2024年

## 目标
实现符合DevilutionX原版的等距墙体渲染

## 实现的内容

### Tile格式
- ✅ 创建32x32梯形/方形墙体tiles (tile_6-11)
- ✅ 左梯形 + 右梯形配对
- ✅ 光照效果：左暗（30%）右亮（85%）

### 数据结构
- ✅ MicroTiles支持LeftTrapezoid/RightTrapezoid
- ✅ MegaTiles配置多层堆叠（micro1-4）

### 渲染逻辑
- ✅ 区分地板和墙体渲染
- ✅ 墙体多层垂直堆叠（2层）
- ✅ 左右配对渲染（32px + 32px = 64px）

## 遇到的问题

### 1. 坐标计算复杂
等距投影下，左右trapezoid的精确拼接需要复杂的坐标计算。

### 2. 位置偏移问题
- 初始实现：左右重叠或有空隙
- 修复后：仍然出现锯齿状交错

### 3. 渲染顺序
多层堆叠的渲染顺序和Y坐标偏移需要精确匹配原版。

## 技术难点

### 等距投影的复杂性
```
完整墙体 (64px) = 左trapezoid (32px) + 右trapezoid (32px)
                           ↓
                  需要精确的水平位置计算
                           ↓
              left_x = final_x - 32
              right_x = final_x
```

### 原版MICROS结构
```
MICROS.mt[16]:
  mt[0-1]:  底层（左右配对）
  mt[2-3]:  第2层（左右配对）+ Y偏移32px
  mt[4-5]:  第3层（左右配对）+ Y偏移64px
  ...
  mt[14-15]: 第8层
```

完整实现需要渲染8层（16个microtiles），当前只实现了2层。

## 回退原因

1. **代码复杂度过高**：墙体渲染逻辑占据大量代码
2. **调试困难**：坐标计算错误难以定位
3. **不符合渐进开发原则**：应该先完善基础渲染，再处理复杂墙体

## 保留的成果

### Tile资源
- ✅ `assets/tiles/tile_0.png ~ tile_11.png`
- ✅ `assets/tiles/create_clean_wall.py`

### 格式解析
- ✅ `src/tiles/*` - 完整的MIN/TIL/SOL解析
- ✅ `src/resources/dungeon_cel.rs` - CEL格式解析
- ✅ TileType枚举（支持6种tile形状）

## 下一步计划

### 短期目标
1. 回退到简单的地板渲染
2. 完善基础渲染管线
3. 优化性能和代码结构

### 长期目标
1. 重新设计墙体渲染架构
2. 参考原版DrawCell函数的精确实现
3. 实现完整的16层microtiles堆叠

## 技术债务记录

- [ ] 墙体等距坐标计算需要重新设计
- [ ] 需要完整实现MICROS.mt[16]的8层渲染
- [ ] 光照上溢（bleed up）尚未实现
- [ ] 透明度mask（MaskType::Left/Right）尚未实现

## 参考资料

### 原版代码
- `Source/engine/render/scrollrt.cpp::DrawCell()`
- `Source/engine/render/dun_render.cpp`
- `Source/levels/dun_tile.hpp`

### 关键常量
- `TILE_WIDTH = 64`
- `TILE_HEIGHT = 32`
- `DunFrameWidth = 32`
- `DunFrameHeight = 32`
- `MicroTileLen = 16`














