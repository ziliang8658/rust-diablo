# 原版C++代码的MPQ文件搜索和读取流程

> 分析DevilutionX C++代码如何搜索并读取MPQ文件

## 核心流程图

```
用户请求文件 "levels/towndata/town.pal"
           ↓
    FindAsset(filename)  [assets.cpp]
           ↓
    ┌──────────────────────────────────┐
    │ 1. 检查绝对路径                   │
    │    if (filename[0] == '/')        │
    │       尝试直接打开                │
    └──────────┬───────────────────────┘
               ↓ (未找到)
    ┌──────────────────────────────────┐
    │ 2. 检查Override路径               │
    │    遍历 OverridePaths             │
    │    尝试 overridePath + filename  │
    └──────────┬───────────────────────┘
               ↓ (未找到)
    ┌──────────────────────────────────┐
    │ 3. 在所有MPQ归档中搜索 ⭐         │
    │    FindMpqFile(filename)          │
    │    ├─ CalculateMpqFileHash()     │
    │    └─ 遍历所有MpqArchives        │
    │       按优先级从高到低            │
    └──────────┬───────────────────────┘
               ↓
    ┌──────────────────────────────────┐
    │ FindMpqFile详细流程               │
    │                                   │
    │ 3.1 计算文件名哈希                │
    │     hash = CalculateMpqFileHash() │
    │     得到 [h0, h1, h2]            │
    │                                   │
    │ 3.2 遍历所有已加载的MPQ归档       │
    │     for (archivePriority, archive) │
    │         按优先级降序 ⬇            │
    │                                   │
    │ 3.3 在当前归档中查找              │
    │     archive.GetFileNumber(hash)   │
    │     调用 libmpq 函数:            │
    │     libmpq__file_number_from_hash │
    │                                   │
    │ 3.4 如果找到                      │
    │     返回 archive 和 fileNumber   │
    │     停止搜索（优先级高的覆盖）    │
    │                                   │
    │ 3.5 如果未找到                    │
    │     继续下一个归档                │
    └──────────┬───────────────────────┘
               ↓ (在MPQ中找到)
    ┌──────────────────────────────────┐
    │ 4. 读取文件内容                   │
    │    archive.ReadFile(filename)     │
    └──────────┬───────────────────────┘
               ↓ (未在MPQ找到)
    ┌──────────────────────────────────┐
    │ 5. 尝试assets目录                 │
    │    AssetsPath() + filename        │
    └──────────┬───────────────────────┘
               ↓ (仍未找到)
    ┌──────────────────────────────────┐
    │ 6. 返回失败                       │
    │    AssetRef.archive = nullptr     │
    └──────────────────────────────────┘
```

## 关键代码分析

### 1. FindAsset - 文件搜索入口

**位置**: `Source/engine/assets.cpp:127-179`

```cpp
AssetRef FindAsset(std::string_view filename)
{
    AssetRef result;
    
    // 规范化路径分隔符
    std::string relativePath { filename };
    #ifndef _WIN32
    std::replace(relativePath.begin(), relativePath.end(), '\\', '/');
    #endif
    
    // 1. 检查绝对路径
    if (relativePath[0] == '/') {
        result.directHandle = SDL_IOFromFile(relativePath.c_str(), "rb");
        if (result.directHandle != nullptr) {
            return result;
        }
    }
    
    // 2. 检查Override路径（用于Mod和覆盖文件）
    for (const auto &overridePath : OverridePaths) {
        const std::string path = overridePath + relativePath;
        result.directHandle = OpenOptionalRWops(path);
        if (result.directHandle != nullptr) {
            LogVerbose("Loaded MPQ file override: {}", path);
            return result;
        }
    }
    
    // 3. ⭐ 在MPQ归档中搜索（最重要！）
    if (FindMpqFile(filename, &result.archive, &result.fileNumber)) {
        result.filename = filename;
        return result;  // 找到了！
    }
    
    // 4. 尝试assets目录
    result.directHandle = OpenOptionalRWops(paths::AssetsPath() + relativePath);
    if (result.directHandle != nullptr)
        return result;
    
    // 5. 未找到
    return result;
}
```

### 2. FindMpqFile - 在MPQ归档中搜索

**位置**: `Source/engine/assets.cpp:68-80`

```cpp
bool FindMpqFile(std::string_view filename, MpqArchive **archive, uint32_t *fileNumber)
{
    // 计算文件名哈希
    const MpqFileHash fileHash = CalculateMpqFileHash(filename);
    
    // 遍历所有已加载的MPQ归档（按优先级从高到低）
    // std::map<int, MpqArchive, std::greater<>> MpqArchives
    // greater<> 意味着优先级高的在前
    for (auto &[priority, mpqArchive] : MpqArchives) {
        // 在当前归档中查找文件
        if (mpqArchive.GetFileNumber(fileHash, *fileNumber)) {
            *archive = &mpqArchive;
            return true;  // 找到了，立即返回（优先级高的覆盖低的）
        }
    }
    
    return false;  // 所有归档都未找到
}
```

**关键点**:
- 使用 `std::map<int, MpqArchive, std::greater<>>` 存储归档
- `std::greater<>` 使得高优先级的归档排在前面
- **第一个匹配的文件就返回**，实现优先级覆盖

### 3. MpqArchive::GetFileNumber - 查找文件编号

**位置**: `Source/mpq/mpq_reader.cpp:55-58`

```cpp
bool MpqArchive::GetFileNumber(MpqFileHash fileHash, uint32_t &fileNumber)
{
    // 调用libmpq库函数
    // fileHash[0] = hash_index (用于Hash Table索引)
    // fileHash[1] = hash_a (用于匹配)
    // fileHash[2] = hash_b (用于匹配)
    return libmpq__file_number_from_hash(
        archive_, 
        fileHash[0],  // hash for table index
        fileHash[1],  // hashA
        fileHash[2],  // hashB
        &fileNumber
    ) == 0;
}
```

**libmpq内部实现** (推测):
```c
// libmpq内部大致流程
int libmpq__file_number_from_hash(archive, h0, h1, h2, fileNumber) {
    // 1. 读取Hash Table (已在Open时解密)
    HashEntry* hashTable = archive->hashTable;
    int hashTableSize = archive->hashTableSize;
    
    // 2. 计算起始索引
    int startIndex = h0 & (hashTableSize - 1);
    int currentIndex = startIndex;
    
    // 3. 线性探测查找
    do {
        HashEntry* entry = &hashTable[currentIndex];
        
        // 检查是否匹配
        if (entry->hashA == h1 && entry->hashB == h2) {
            // 找到了！
            if (entry->block == 0xFFFFFFFF || entry->block == 0xFFFFFFFE) {
                // 空槽位或已删除
                return ERROR_NOT_FOUND;
            }
            *fileNumber = entry->block;  // 文件在Block Table中的索引
            return SUCCESS;
        }
        
        // 空槽位，文件不存在
        if (entry->block == 0xFFFFFFFF) {
            return ERROR_NOT_FOUND;
        }
        
        // 继续下一个槽位（线性探测）
        currentIndex = (currentIndex + 1) % hashTableSize;
        
    } while (currentIndex != startIndex);
    
    return ERROR_NOT_FOUND;
}
```

### 4. MpqArchive::ReadFile - 读取文件内容

**位置**: `Source/mpq/mpq_reader.cpp:60-98`

```cpp
std::unique_ptr<std::byte[]> MpqArchive::ReadFile(
    std::string_view filename, 
    std::size_t &fileSize, 
    int32_t &error
) {
    std::unique_ptr<std::byte[]> result;
    std::uint32_t fileNumber;
    
    // 4.1 获取文件编号（再次调用，用于确认）
    error = libmpq__file_number_s(
        archive_, 
        filename.data(), 
        filename.size(), 
        &fileNumber
    );
    if (error != 0) return result;
    
    // 4.2 获取文件解压后的大小
    libmpq__off_t unpackedSize;
    error = libmpq__file_size_unpacked(archive_, fileNumber, &unpackedSize);
    if (error != 0) return result;
    
    // 4.3 打开Block Offset Table（用于多块文件）
    error = OpenBlockOffsetTable(fileNumber, filename);
    if (error != 0) return result;
    
    // 4.4 分配输出缓冲区
    result = std::make_unique<std::byte[]>(static_cast<size_t>(unpackedSize));
    
    // 4.5 获取块大小
    const std::size_t blockSize = GetBlockSize(fileNumber, 0, error);
    if (error != 0) return result;
    
    // 4.6 获取临时缓冲区
    std::vector<std::uint8_t> &tmp = GetTemporaryBuffer(blockSize);
    
    // 4.7 读取文件数据（libmpq处理解密和解压）
    error = libmpq__file_read_with_filename_and_temporary_buffer_s(
        archive_, 
        fileNumber, 
        filename.data(), 
        filename.size(), 
        reinterpret_cast<std::uint8_t *>(result.get()), 
        unpackedSize,
        tmp.data(),  // 临时缓冲区用于解压
        static_cast<libmpq__off_t>(blockSize), 
        nullptr
    );
    
    if (error != 0) {
        result = nullptr;
        CloseBlockOffsetTable(fileNumber);
        return result;
    }
    
    // 4.8 关闭Block Offset Table
    CloseBlockOffsetTable(fileNumber);
    
    fileSize = static_cast<size_t>(unpackedSize);
    return result;
}
```

**关键点**:
- libmpq库自动处理**解密**和**解压缩**
- 使用临时缓冲区进行块解压
- Block Offset Table用于大文件的分块读取

## MPQ归档加载流程

### LoadMPQ函数

**位置**: `Source/engine/assets.cpp:301-332`

```cpp
bool LoadMPQ(std::span<const std::string> paths, std::string_view mpqName, int priority)
{
    int32_t error;
    
    // 1. 在多个路径中搜索MPQ文件
    std::string mpqAbsPath;
    for (const auto &path : paths) {
        mpqAbsPath = StrCat(path, mpqName, ".mpq");
        if (FileExists(mpqAbsPath)) {
            LogVerbose("  Found: {} in {}", mpqName, path);
            
            // 2. 打开MPQ归档
            std::optional<MpqArchive> archive = MpqArchive::Open(mpqAbsPath.c_str(), error);
            if (archive.has_value()) {
                // 3. 添加到归档映射（按优先级）
                MpqArchives[priority] = *std::move(archive);
                return true;
            }
            
            // 打开失败
            LogError("Failed to open {}: {}", 
                     mpqAbsPath, 
                     MpqArchive::ErrorMessage(error));
            return false;
        }
    }
    
    // 未找到MPQ文件
    LogVerbose("Missing: {}", mpqName);
    return false;
}
```

### 典型的加载顺序

```cpp
// 在 Source/engine/assets.cpp 中
void InitAssets()
{
    // 优先级从低到高加载
    LoadMPQ(paths, "diabdat", 1000);      // 基础游戏数据
    LoadMPQ(paths, "spawn", 1001);        // 试玩版（如果存在）
    LoadMPQ(paths, "hellfire", 2001);     // Hellfire扩展
    LoadMPQ(paths, "hfmonk", 2002);       // Monk职业
    LoadMPQ(paths, "hfbard", 2003);       // Bard职业
    LoadMPQ(paths, "hfbarb", 2004);       // Barbarian职业
    LoadMPQ(paths, "devilutionx", 3000);  // DevilutionX扩展
    LoadMPQ(paths, "fonts", 4000);        // 字体
    LoadMPQ(paths, "lang", 5000);         // 语言包
    
    // 优先级高的会覆盖低的
}
```

## 完整的文件读取示例

```cpp
// 游戏代码请求读取调色板文件
std::string palettePath = "levels/towndata/town.pal";

// 1. 查找文件
AssetRef ref = FindAsset(palettePath);
// FindAsset内部:
//   - 检查Override路径
//   - 调用FindMpqFile在所有MPQ中搜索
//     * CalculateMpqFileHash("levels/towndata/town.pal")
//     * 遍历MpqArchives（优先级高的先）
//     * 调用archive.GetFileNumber(hash)
//     * libmpq在Hash Table中查找
//   - 找到后返回 archive指针 和 fileNumber

if (!ref.ok()) {
    // 文件未找到
    return error;
}

// 2. 打开并读取文件
AssetHandle handle = OpenAsset(std::move(ref));
// OpenAsset内部:
//   - 调用SDL_RWops_FromMpqFile创建读取句柄
//   - 最终调用archive.ReadFile()
//     * libmpq__file_read_with_filename_and_temporary_buffer_s
//     * 读取Block Table条目
//     * seek到文件offset
//     * 读取compressed data
//     * 解密（如果需要）
//     * 解压（PKWare/Huffman等）
//     * 返回解压后的数据

// 3. 使用数据
std::vector<uint8_t> data(size);
handle.read(data.data(), size);

// data现在包含town.pal的768字节（256色 × 3 RGB）
```

## 关键优化

### 1. 优先级系统
- 使用 `std::map<int, MpqArchive, std::greater<>>` 
- 自动按优先级降序排列
- 第一个匹配的文件立即返回

### 2. 哈希查找
- 不需要遍历所有文件名
- O(1) 平均时间复杂度（带冲突处理）
- 支持大小写不敏感

### 3. 缓冲区复用
- `GetTemporaryBuffer()` 复用内存
- 避免频繁分配/释放

### 4. Lazy Loading
- MPQ在首次使用时才打开
- 文件在需要时才读取

## 与Rust实现的对照

| 功能 | C++实现 | Rust实现计划 |
|-----|---------|------------|
| 哈希计算 | `CalculateMpqFileHash()` | ✅ `calculate_mpq_file_hash()` |
| 表解密 | libmpq自动处理 | ✅ `decrypt_block()` |
| 文件查找 | `GetFileNumber(hash)` | ✅ `find_file_block()` |
| 线性探测 | libmpq内部实现 | ✅ 已实现 |
| PKWare解压 | libmpq + PKWare库 | ⚠️ 需要实现 |
| 优先级系统 | `std::map<int, _, greater<>>` | ✅ `Vec` + `sort_by_key` |
| 多归档支持 | `MpqArchives` map | ✅ `archives: Vec<>` |

## 总结

原版C++的文件搜索和读取流程：

1. **多层fallback机制**：绝对路径 → Override → MPQ → Assets目录
2. **优先级系统**：高优先级MPQ自动覆盖低优先级
3. **libmpq封装**：自动处理解密、解压、分块读取
4. **哈希查找**：O(1)时间复杂度，支持冲突处理
5. **性能优化**：缓冲区复用、lazy loading

对于Rust实现，我们需要：
- ✅ 实现完整的哈希和解密算法
- ⚠️ 实现PKWare解压缩（最后缺失的部分）
- ✅ 实现优先级系统和多归档支持
- ✅ 处理文件查找的线性探测


