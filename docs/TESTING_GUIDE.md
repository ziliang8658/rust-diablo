# 测试指南 - Rust Diablo 项目

## 📋 测试原则

### 测试金字塔

```
        /\
       /  \      E2E测试 (5%)
      /____\     - 完整游戏流程测试
     /      \    
    /________\   集成测试 (15%)
   /          \  - 系统间交互测试
  /____________\ 
 /              \ 单元测试 (80%)
/________________\ - 函数/模块级测试
```

### 每个Step的测试要求

1. **单元测试**：核心逻辑必须有单元测试
2. **集成测试**：系统间交互需要集成测试
3. **手动测试**：可玩性测试（用测试场景）
4. **性能测试**：关键路径需要性能基准测试

---

## 🧪 测试类型

### 1. 单元测试 (Unit Tests)

**位置：** 在模块文件底部的 `#[cfg(test)]` 块中

**示例：**
```rust
// src/math/point.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_addition() {
        let p1 = Point::new(10, 20);
        let p2 = Point::new(5, 3);
        let result = p1 + p2;
        assert_eq!(result, Point::new(15, 23));
    }

    #[test]
    fn test_distance_calculation() {
        let p1 = Point::new(0, 0);
        let p2 = Point::new(3, 4);
        assert_eq!(p1.distance_to(&p2), 5);
    }
}
```

**运行：**
```bash
cargo test
cargo test test_point_addition  # 运行特定测试
cargo test --package rust-diablo --lib math::point  # 运行特定模块
```

---

### 2. 集成测试 (Integration Tests)

**位置：** `tests/` 目录

**示例：**
```rust
// tests/game_integration_test.rs

use rust_diablo::{Game, Engine};

#[test]
fn test_game_initialization() {
    let result = Game::new();
    assert!(result.is_ok());
}

#[test]
fn test_player_movement() {
    let mut game = Game::new().unwrap();
    let initial_pos = game.player_position();
    
    game.move_player(1, 0);
    game.update(0.016).unwrap();
    
    let new_pos = game.player_position();
    assert_ne!(initial_pos, new_pos);
}
```

**运行：**
```bash
cargo test --test game_integration_test
cargo test --test '*'  # 运行所有集成测试
```

---

### 3. 文档测试 (Doc Tests)

**位置：** 在文档注释中

**示例：**
```rust
/// Point represents a 2D coordinate
/// 
/// # Examples
/// 
/// ```
/// use rust_diablo::math::Point;
/// 
/// let p = Point::new(10, 20);
/// assert_eq!(p.x, 10);
/// assert_eq!(p.y, 20);
/// ```
pub struct Point {
    pub x: i32,
    pub y: i32,
}
```

**运行：**
```bash
cargo test --doc
```

---

### 4. 基准测试 (Benchmark Tests)

**位置：** `benches/` 目录

**示例：**
```rust
// benches/pathfinding_benchmark.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_diablo::pathfinding::a_star;

fn benchmark_pathfinding(c: &mut Criterion) {
    c.bench_function("a_star_10x10", |b| {
        let map = create_test_map(10, 10);
        b.iter(|| {
            a_star(
                black_box(&map),
                black_box((0, 0)),
                black_box((9, 9))
            )
        });
    });
}

criterion_group!(benches, benchmark_pathfinding);
criterion_main!(benches);
```

**Cargo.toml 配置：**
```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "pathfinding_benchmark"
harness = false
```

**运行：**
```bash
cargo bench
cargo bench --bench pathfinding_benchmark
```

---

### 5. 手动测试场景 (Manual Test Scenarios)

**位置：** `examples/` 目录

**示例：**
```rust
// examples/test_combat.rs

use rust_diablo::*;

fn main() -> anyhow::Result<()> {
    // 初始化游戏
    let mut game = Game::new()?;
    
    // 设置测试场景
    game.spawn_player_at(Point::new(100, 100));
    game.spawn_monster("zombie", Point::new(150, 100));
    game.set_player_health(50);
    game.give_player_weapon("sword");
    
    // 运行游戏循环
    game.run()
}
```

**运行：**
```bash
cargo run --example test_combat
cargo run --example test_pathfinding
```

---

## 📊 按Step的测试要求

### Step 1-3: 基础框架 (已完成，需补充测试)

#### Step 1: 基础游戏框架
**单元测试：**
- [x] Engine初始化测试
- [x] 事件处理测试
- [x] 游戏循环基础测试

**集成测试：**
- [ ] 完整窗口创建和关闭测试

**测试文件：**
```
tests/
  └── step1_foundation_test.rs
src/
  ├── engine.rs (内含 #[cfg(test)])
  └── game.rs (内含 #[cfg(test)])
```

---

#### Step 2: 渲染和坐标系统
**单元测试：**
- [x] Point 加减运算
- [x] Point 距离计算
- [x] Rect 碰撞检测
- [x] Rect 包含判断
- [x] Color RGB转换

**集成测试：**
- [ ] 实体渲染测试
- [ ] 世界更新测试

**性能测试：**
- [ ] 大量实体渲染性能

**测试文件：**
```
tests/
  └── step2_rendering_test.rs
benches/
  └── rendering_benchmark.rs
src/math/
  ├── point.rs (内含测试)
  └── rect.rs (内含测试)
```

---

#### Step 3: 精灵系统
**单元测试：**
- [ ] 纹理加载测试（模拟）
- [ ] AnimationController 状态切换
- [ ] 帧更新逻辑

**集成测试：**
- [ ] 精灵渲染测试
- [ ] 动画播放测试

**手动测试：**
- [x] 玩家精灵显示

**测试文件：**
```
tests/
  └── step3_sprite_test.rs
src/sprite/
  ├── texture.rs (内含测试)
  ├── animation.rs (内含测试)
  └── sprite.rs (内含测试)
```

---

### Step 4: 动画系统和碰撞检测

**单元测试：**
```rust
// src/sprite/animation.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_animation_frame_advance() {
        let mut anim = Animation::new(vec![
            Rect::new(0, 0, 32, 32),
            Rect::new(32, 0, 32, 32),
        ], 0.1, true);
        
        // 第一帧
        assert_eq!(anim.current_frame(), 0);
        
        // 更新时间
        anim.update(0.15);
        
        // 应该前进到第二帧
        assert_eq!(anim.current_frame(), 1);
    }

    #[test]
    fn test_animation_loop() {
        let mut anim = Animation::new(vec![
            Rect::new(0, 0, 32, 32),
            Rect::new(32, 0, 32, 32),
        ], 0.1, true);
        
        anim.update(0.25);  // 超过总时长
        assert_eq!(anim.current_frame(), 0);  // 应该循环
        assert!(!anim.is_finished());
    }

    #[test]
    fn test_animation_no_loop() {
        let mut anim = Animation::new(vec![
            Rect::new(0, 0, 32, 32),
        ], 0.1, false);
        
        anim.update(0.15);
        assert!(anim.is_finished());
    }
}

// src/world/collision.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_tile_collision() {
        let world = World::new(10, 10, 32);
        // 设置障碍物
        world.set_tile(5, 5, TileType::Wall);
        
        // 测试碰撞
        assert!(!world.can_move_to(Point::new(5*32, 5*32)));
        assert!(world.can_move_to(Point::new(6*32, 6*32)));
    }

    #[test]
    fn test_entity_collision() {
        let mut world = World::new(10, 10, 32);
        let player = Entity::create_player(Point::new(100, 100));
        let monster = Entity::new(Point::new(110, 100), 32, 32);
        
        assert!(player.bounds().intersects(&monster.bounds()));
    }
}
```

**集成测试：**
```rust
// tests/step4_animation_test.rs

#[test]
fn test_player_walk_animation() {
    let mut game = Game::new().unwrap();
    let player_index = game.get_player_index();
    
    // 初始应该是Idle动画
    assert_eq!(game.get_entity_animation_state(player_index), AnimationState::Idle);
    
    // 移动玩家
    game.handle_player_move(1, 0);
    game.update(0.016).unwrap();
    
    // 应该切换到Walk动画
    assert_eq!(game.get_entity_animation_state(player_index), AnimationState::Walk);
}

#[test]
fn test_collision_blocks_movement() {
    let mut game = Game::new().unwrap();
    let initial_pos = game.player_position();
    
    // 在玩家前方放置墙
    game.set_tile(initial_pos.x + 32, initial_pos.y, TileType::Wall);
    
    // 尝试向前移动
    game.handle_player_move(1, 0);
    game.update(0.016).unwrap();
    
    // 位置应该没变
    assert_eq!(game.player_position(), initial_pos);
}
```

**手动测试场景：**
```rust
// examples/test_animation.rs

fn main() -> anyhow::Result<()> {
    let mut game = Game::new()?;
    
    // 测试场景：显示所有动画状态
    game.enable_debug_mode();
    game.spawn_player_at(Point::new(200, 200));
    
    println!("控制：");
    println!("WASD - 移动 (测试Walk动画)");
    println!("Space - 攻击 (测试Attack动画)");
    println!("1-9 - 切换动画状态");
    
    game.run()
}
```

**性能测试：**
```rust
// benches/animation_benchmark.rs

fn benchmark_animation_update(c: &mut Criterion) {
    c.bench_function("update_100_animations", |b| {
        let mut animations = vec![];
        for _ in 0..100 {
            animations.push(create_test_animation());
        }
        
        b.iter(|| {
            for anim in &mut animations {
                anim.update(0.016);
            }
        });
    });
}
```

---

### Step 5: 地图生成系统

**单元测试：**
```rust
#[test]
fn test_room_generation() {
    let mut generator = DungeonGenerator::new(50, 50);
    let room = generator.create_room(10, 10, 5, 5);
    
    assert_eq!(room.width, 5);
    assert_eq!(room.height, 5);
    assert!(generator.is_room_valid(&room));
}

#[test]
fn test_corridor_generation() {
    let room1 = Room::new(5, 5, 3, 3);
    let room2 = Room::new(15, 5, 3, 3);
    let corridor = create_corridor(&room1, &room2);
    
    assert!(!corridor.is_empty());
}

#[test]
fn test_resource_loading_pcx() {
    let loader = ResourceLoader::new();
    let result = loader.load_pcx("test_assets/test.pcx");
    assert!(result.is_ok());
}

#[test]
fn test_palette_loading() {
    let palette = Palette::from_file("test_assets/test.pal");
    assert!(palette.is_ok());
    assert_eq!(palette.unwrap().colors.len(), 256);
}
```

**集成测试：**
```rust
#[test]
fn test_dungeon_generation() {
    let generator = DungeonGenerator::new(50, 50);
    let dungeon = generator.generate(1234);  // 固定种子
    
    // 验证地牢属性
    assert!(dungeon.room_count() >= 5);
    assert!(dungeon.room_count() <= 10);
    assert!(dungeon.is_all_rooms_connected());
}

#[test]
fn test_mpq_resource_loading() {
    let loader = ResourceLoader::new();
    loader.load_mpq("DIABDAT.MPQ").unwrap();
    
    // 测试加载各种资源
    assert!(loader.has_resource("levels/towndata/town.dun"));
    assert!(loader.has_resource("gendata/cutstart.cel"));
}
```

---

### Step 6: 怪物系统

**单元测试：**
```rust
#[test]
fn test_monster_initialization() {
    let monster = Monster::new(MonsterType::Zombie, Point::new(100, 100));
    assert_eq!(monster.monster_type, MonsterType::Zombie);
    assert!(monster.hp > 0);
    assert_eq!(monster.state, MonsterState::Idle);
}

#[test]
fn test_monster_take_damage() {
    let mut monster = Monster::new(MonsterType::Zombie, Point::new(100, 100));
    let initial_hp = monster.hp;
    
    monster.take_damage(10);
    assert_eq!(monster.hp, initial_hp - 10);
}

#[test]
fn test_monster_death() {
    let mut monster = Monster::new(MonsterType::Zombie, Point::new(100, 100));
    monster.hp = 5;
    
    monster.take_damage(10);
    assert!(monster.is_dead());
}
```

**集成测试：**
```rust
#[test]
fn test_monster_spawning() {
    let mut game = Game::new().unwrap();
    let initial_count = game.monster_count();
    
    game.spawn_monster(MonsterType::Zombie, Point::new(200, 200));
    assert_eq!(game.monster_count(), initial_count + 1);
}

#[test]
fn test_monster_ai_idle() {
    let mut game = Game::new().unwrap();
    let monster_id = game.spawn_monster(MonsterType::Zombie, Point::new(200, 200));
    let initial_pos = game.get_monster_position(monster_id);
    
    // 玩家远离，怪物应该待机
    game.set_player_position(Point::new(1000, 1000));
    
    for _ in 0..60 {  // 1秒
        game.update(0.016).unwrap();
    }
    
    // 位置变化应该很小（只是随机漫步）
    let new_pos = game.get_monster_position(monster_id);
    assert!(initial_pos.distance_to(&new_pos) < 50);
}
```

---

### Step 7: 战斗系统

**单元测试：**
```rust
#[test]
fn test_damage_calculation() {
    let attacker = create_test_player();
    let defender = create_test_monster();
    
    let damage = calculate_damage(&attacker, &defender);
    assert!(damage >= attacker.min_damage);
    assert!(damage <= attacker.max_damage);
}

#[test]
fn test_hit_chance() {
    let attacker = create_test_player();
    let defender = create_test_monster();
    
    let hit_chance = calculate_hit_chance(&attacker, &defender);
    assert!(hit_chance >= 0.0);
    assert!(hit_chance <= 1.0);
}

#[test]
fn test_armor_reduction() {
    let armor = 50;
    let damage = 100;
    
    let reduced = apply_armor_reduction(damage, armor);
    assert!(reduced < damage);
    assert!(reduced > 0);
}
```

**集成测试：**
```rust
#[test]
fn test_player_attacks_monster() {
    let mut game = Game::new().unwrap();
    let monster_id = game.spawn_monster(MonsterType::Zombie, Point::new(110, 100));
    let initial_hp = game.get_monster_hp(monster_id);
    
    game.player_attack(monster_id);
    game.update(0.016).unwrap();
    
    let new_hp = game.get_monster_hp(monster_id);
    assert!(new_hp < initial_hp);
}

#[test]
fn test_monster_dies_and_drops_loot() {
    let mut game = Game::new().unwrap();
    let monster_id = game.spawn_monster(MonsterType::Zombie, Point::new(110, 100));
    
    // 杀死怪物
    game.set_monster_hp(monster_id, 1);
    game.player_attack(monster_id);
    game.update(0.016).unwrap();
    
    // 验证怪物死亡
    assert!(!game.monster_exists(monster_id));
    
    // 验证有物品掉落
    let items = game.get_items_near(Point::new(110, 100), 50);
    assert!(!items.is_empty());
}
```

---

## 🎯 测试覆盖率目标

### 总体目标
- **单元测试覆盖率：** ≥ 80%
- **集成测试覆盖率：** ≥ 60%
- **关键路径覆盖：** 100%

### 工具
```bash
# 安装 tarpaulin (Linux/macOS)
cargo install cargo-tarpaulin

# 运行覆盖率测试
cargo tarpaulin --out Html --output-dir coverage

# 查看报告
open coverage/index.html
```

### Windows上使用
```powershell
# 使用 llvm-cov
cargo install cargo-llvm-cov

# 运行覆盖率
cargo llvm-cov --html

# 查看报告
start target\llvm-cov\html\index.html
```

---

## 🔍 测试策略

### 关键系统测试重点

| 系统 | 测试重点 | 覆盖率目标 |
|------|---------|-----------|
| 数学库 | 所有运算符、边界值 | 95% |
| 碰撞检测 | 各种碰撞情况 | 90% |
| 动画系统 | 状态转换、循环 | 85% |
| 地图生成 | 连通性、种子确定性 | 70% |
| AI系统 | 状态转换、决策 | 75% |
| 战斗系统 | 伤害计算、状态效果 | 90% |
| 网络同步 | 数据一致性 | 95% |
| 存档系统 | 序列化/反序列化 | 90% |

---

## 🐛 测试驱动开发 (TDD) 流程

### Red-Green-Refactor

1. **Red（红）** - 编写失败的测试
```rust
#[test]
fn test_player_level_up() {
    let mut player = Player::new();
    player.experience = 1000;
    
    player.check_level_up();
    
    assert_eq!(player.level, 2);  // 这会失败，因为功能还没实现
}
```

2. **Green（绿）** - 编写最简代码使测试通过
```rust
impl Player {
    fn check_level_up(&mut self) {
        if self.experience >= 1000 {
            self.level = 2;
        }
    }
}
```

3. **Refactor（重构）** - 改进代码质量
```rust
impl Player {
    fn check_level_up(&mut self) {
        let required_exp = self.level * 1000;
        if self.experience >= required_exp {
            self.level += 1;
            self.experience -= required_exp;
        }
    }
}
```

---

## 📝 测试清单模板

每个Step完成时检查：

### 功能测试
- [ ] 所有公共API都有单元测试
- [ ] 边界条件都有测试
- [ ] 错误情况都有测试
- [ ] 测试覆盖率达标

### 集成测试
- [ ] 系统间交互有测试
- [ ] 关键流程有端到端测试
- [ ] 性能关键路径有基准测试

### 手动测试
- [ ] 创建了测试场景example
- [ ] 手动验证了可玩性
- [ ] 测试了边界情况

### 文档
- [ ] 测试用例有文档注释
- [ ] 复杂测试有说明
- [ ] 测试场景有使用说明

---

## 🚀 CI/CD 集成

### GitHub Actions 配置

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, nightly]
    
    steps:
      - uses: actions/checkout@v2
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
          override: true
      
      - name: Install dependencies (Ubuntu)
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y libsdl2-dev libsdl2-image-dev
      
      - name: Run tests
        run: cargo test --verbose
      
      - name: Run benchmarks
        if: matrix.rust == 'nightly'
        run: cargo bench --no-run
      
      - name: Check code coverage
        if: matrix.os == 'ubuntu-latest' && matrix.rust == 'stable'
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml
      
      - name: Upload coverage
        if: matrix.os == 'ubuntu-latest' && matrix.rust == 'stable'
        uses: codecov/codecov-action@v1
```

---

## 📚 测试资源

### 测试数据
```
tests/
  fixtures/
    ├── test_sprites/
    │   ├── player.png
    │   └── monster.png
    ├── test_maps/
    │   └── test_dungeon.json
    └── test_saves/
        └── test_save.dat
```

### Mock对象
```rust
// tests/mocks/mock_renderer.rs
pub struct MockRenderer {
    pub draw_calls: Vec<DrawCall>,
}

impl Renderer for MockRenderer {
    fn draw_sprite(&mut self, sprite: &Sprite, pos: Point) {
        self.draw_calls.push(DrawCall::Sprite(sprite.id.clone(), pos));
    }
}
```

---

## ✅ 最佳实践

1. **测试命名**：`test_<功能>_<场景>_<预期结果>`
2. **Arrange-Act-Assert**：清晰的测试结构
3. **单一职责**：每个测试只测一个东西
4. **可重复性**：测试结果必须一致
5. **独立性**：测试之间不应相互依赖
6. **快速运行**：单元测试应该快速完成

---

**最后更新：** 2025-11-17  
**适用范围：** 所有Steps  
**维护者：** Rust Diablo开发团队







