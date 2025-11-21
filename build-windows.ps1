# Windows 构建脚本
# 编译项目
# 注意：SDL2 库路径已配置在 .cargo/config.toml 中

# 编译项目
cargo build

# 复制 SDL2.dll 和 SDL2_image.dll
New-Item -ItemType Directory -Force -Path "target\debug" | Out-Null
Copy-Item "F:\SDL2-2.28.3\lib\x64\SDL2.dll" -Destination "target\debug\SDL2.dll" -Force

if (Test-Path "F:\SDL2-2.28.3\lib\x64\SDL2_image.dll") {
    Copy-Item "F:\SDL2-2.28.3\lib\x64\SDL2_image.dll" -Destination "target\debug\SDL2_image.dll" -Force
    Write-Host "`n编译完成！SDL2.dll 和 SDL2_image.dll 已复制到 target\debug\"
} else {
    Write-Host "`n编译完成！SDL2.dll 已复制到 target\debug\"
    Write-Host "警告: 未找到 SDL2_image.dll，如果运行时出错，请确保已安装 SDL2_image"
}

Write-Host "运行: .\target\debug\rust-diablo.exe"

