# Windows 运行脚本
# 复制 SDL2 DLL 并运行项目
# 注意：SDL2 库路径已配置在 .cargo/config.toml 中

# 确保 target\debug 目录存在
New-Item -ItemType Directory -Force -Path "target\debug" | Out-Null

# 复制 SDL2.dll 到 target\debug
Copy-Item "F:\SDL2-2.28.3\lib\x64\SDL2.dll" -Destination "target\debug\SDL2.dll" -Force

# 复制 SDL2_image.dll 到 target\debug（如果存在）
if (Test-Path "F:\SDL2-2.28.3\lib\x64\SDL2_image.dll") {
    Copy-Item "F:\SDL2-2.28.3\lib\x64\SDL2_image.dll" -Destination "target\debug\SDL2_image.dll" -Force
    Write-Host "SDL2.dll 和 SDL2_image.dll 已复制到 target\debug\"
} else {
    Write-Host "SDL2.dll 已复制到 target\debug\"
    Write-Host "警告: 未找到 SDL2_image.dll，如果运行时出错，请确保已安装 SDL2_image"
}

# 运行项目
cargo run

