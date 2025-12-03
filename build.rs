// build.rs - 编译时构建脚本
// 用于链接预编译的 libmpq 静态库
//
// 依赖: zlib, bzip2
// 安装: vcpkg install --triplet x64-windows
//
// 使用说明:
// 1. 先编译 libmpq: cd ../3rdParty/libmpq && mkdir build && cd build && cmake .. && cmake --build . --config Release
// 2. 然后运行 cargo build

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=../3rdParty/libmpq/build");

    // 保存 libmpq 库的目录路径，稍后统一配置链接
    let mut libmpq_lib_dir: Option<PathBuf> = None;

    // libmpq 静态库路径（根据构建系统不同，路径可能不同）
    let libmpq_build_dir = PathBuf::from("../3rdParty/libmpq/build");

    // Windows MSVC: Release/mpq.lib 或 Debug/mpq.lib
    // Windows MinGW/Linux: libmpq.a 或 mpq.a
    let possible_lib_paths = vec![
        libmpq_build_dir.join("Release").join("mpq.lib"),
        libmpq_build_dir.join("Debug").join("mpq.lib"),
        libmpq_build_dir.join("mpq.lib"),
        libmpq_build_dir.join("libmpq.a"),
        libmpq_build_dir.join("mpq.a"),
    ];

    // 查找静态库文件
    let mut found_lib = None;
    for lib_path in &possible_lib_paths {
        if lib_path.exists() {
            found_lib = Some(lib_path.clone());
            break;
        }
    }

    if let Some(lib_path) = found_lib {
        libmpq_lib_dir = Some(lib_path.parent().unwrap().to_path_buf());
        println!("cargo:warning=Found libmpq library at: {:?}", lib_path);
        // 注意：链接配置将在后面统一处理，确保正确的链接顺序
    } else {
        println!("cargo:warning=libmpq static library not found!");
        println!("cargo:warning=Please compile libmpq first:");
        println!("cargo:warning=  cd ../3rdParty/libmpq");
        println!("cargo:warning=  mkdir build && cd build");
        println!("cargo:warning=  cmake ..");
        println!("cargo:warning=  cmake --build . --config Release");
        println!("cargo:warning=Skipping libmpq FFI support");
        return;
    }

    // 查找并链接 zlib 和 bzip2
    let mut found_zlib = false;
    let mut found_bzip2 = false;

    // 方法1: 使用 VCPKG_ROOT 环境变量
    if let Ok(vcpkg_root) = env::var("VCPKG_ROOT") {
        // 检查 packages 目录（manifest 模式）
        let vcpkg_packages = PathBuf::from(&vcpkg_root).join("packages");
        if vcpkg_packages.exists() {
            // 查找 zlib
            let zlib_dir = vcpkg_packages.join("zlib_x64-windows");
            if zlib_dir.exists() {
                let lib_dir = zlib_dir.join("lib");
                if lib_dir.exists() {
                    println!("cargo:rustc-link-search=native={}", lib_dir.display());
                    found_zlib = true;
                }
            }

            // 查找 bzip2
            let bzip2_dir = vcpkg_packages.join("bzip2_x64-windows");
            if bzip2_dir.exists() {
                let lib_dir = bzip2_dir.join("lib");
                if lib_dir.exists() {
                    println!("cargo:rustc-link-search=native={}", lib_dir.display());
                    found_bzip2 = true;
                }
            }
        }

        // 方法2: 检查 installed 目录
        let vcpkg_installed = PathBuf::from(&vcpkg_root)
            .join("installed")
            .join("x64-windows");
        if vcpkg_installed.exists() {
            let lib_dir = vcpkg_installed.join("lib");
            if lib_dir.exists() {
                println!("cargo:rustc-link-search=native={}", lib_dir.display());
                if !found_zlib {
                    found_zlib =
                        lib_dir.join("zlib.lib").exists() || lib_dir.join("zlibd.lib").exists();
                }
                if !found_bzip2 {
                    found_bzip2 =
                        lib_dir.join("bz2.lib").exists() || lib_dir.join("bz2d.lib").exists();
                }
            }
        }
    }

    // 方法3: 检查当前项目的 vcpkg_installed（manifest 模式）
    let project_vcpkg = env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("vcpkg_installed")
        .join("x64-windows");
    if project_vcpkg.exists() {
        let lib_dir = project_vcpkg.join("lib");
        if lib_dir.exists() {
            println!("cargo:rustc-link-search=native={}", lib_dir.display());
            if !found_zlib {
                found_zlib =
                    lib_dir.join("zlib.lib").exists() || lib_dir.join("zlibd.lib").exists();
            }
            if !found_bzip2 {
                found_bzip2 = lib_dir.join("bz2.lib").exists() || lib_dir.join("bz2d.lib").exists();
            }
        }
    }

    // 统一配置链接搜索路径和链接库
    // 重要：必须先配置搜索路径，再配置链接库，并且按正确顺序

    if cfg!(target_os = "windows") {
        // Windows 链接配置
        println!("cargo:warning=Configuring Windows linking...");

        // 1. 配置搜索路径（vcpkg 优先，然后是 libmpq）
        let project_vcpkg_lib = env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("vcpkg_installed")
            .join("x64-windows")
            .join("lib");
        if project_vcpkg_lib.exists() {
            println!(
                "cargo:rustc-link-search=native={}",
                project_vcpkg_lib.display()
            );
            println!(
                "cargo:warning=Added vcpkg lib search path: {}",
                project_vcpkg_lib.display()
            );
        }

        if let Some(ref mpq_dir) = libmpq_lib_dir {
            println!("cargo:rustc-link-search=native={}", mpq_dir.display());
            println!(
                "cargo:warning=Added libmpq lib search path: {}",
                mpq_dir.display()
            );
        }

        // 2. 直接传递库文件的完整路径给链接器（按依赖顺序：zlib, bz2, mpq）
        let zlib_lib = project_vcpkg_lib.join("zlib.lib");
        if zlib_lib.exists() {
            println!("cargo:rustc-link-arg={}", zlib_lib.display());
            println!("cargo:warning=Adding zlib.lib: {}", zlib_lib.display());
        } else {
            println!(
                "cargo:warning=zlib.lib not found at: {}",
                zlib_lib.display()
            );
        }

        let bz2_lib = project_vcpkg_lib.join("bz2.lib");
        if bz2_lib.exists() {
            println!("cargo:rustc-link-arg={}", bz2_lib.display());
            println!("cargo:warning=Adding bz2.lib: {}", bz2_lib.display());
        } else {
            println!("cargo:warning=bz2.lib not found at: {}", bz2_lib.display());
        }

        if let Some(ref mpq_dir) = libmpq_lib_dir {
            let mpq_lib = mpq_dir.join("mpq.lib");
            if mpq_lib.exists() {
                println!("cargo:rustc-link-arg={}", mpq_lib.display());
                println!("cargo:warning=Adding mpq.lib: {}", mpq_lib.display());
            } else {
                println!("cargo:warning=mpq.lib not found at: {}", mpq_lib.display());
            }
        }
    } else {
        // Linux/Unix 链接配置
        if let Some(ref mpq_dir) = libmpq_lib_dir {
            println!("cargo:rustc-link-search=native={}", mpq_dir.display());
        }
        // Linux 上使用标准的链接库名称
        println!("cargo:rustc-link-lib=z");
        println!("cargo:rustc-link-lib=bz2");
        if libmpq_lib_dir.is_some() {
            println!("cargo:rustc-link-lib=static=mpq");
        }
    }

    if !found_zlib || !found_bzip2 {
        println!(
            "cargo:warning=zlib found: {}, bzip2 found: {}",
            found_zlib, found_bzip2
        );
        println!("cargo:warning=If linking fails, ensure zlib and bzip2 are installed via vcpkg");
    }

    println!("cargo:warning=libmpq linking configured successfully");

    // 复制运行时需要的 DLL 文件
    copy_runtime_dlls();
}

fn copy_runtime_dlls() {
    // 获取输出目录
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let out_path = PathBuf::from(&out_dir);

    // 从 OUT_DIR 推导出 target/debug 或 target/release 目录
    // OUT_DIR 格式: target/debug/build/rust-diablo-xxx/out
    let mut target_dir = out_path.clone();
    loop {
        if let Some(parent) = target_dir.parent() {
            let dir_name = target_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if dir_name == "debug" || dir_name == "release" {
                break;
            }
            target_dir = parent.to_path_buf();
        } else {
            println!("cargo:warning=Failed to find target directory from OUT_DIR");
            return;
        }
    }

    println!("cargo:warning=Target directory: {}", target_dir.display());

    // 定义需要复制的 DLL 列表
    let dll_sources = vec![
        ("SDL2.dll", "F:\\SDL2-2.28.3\\lib\\x64\\SDL2.dll"),
        (
            "SDL2_image.dll",
            "F:\\SDL2-2.28.3\\lib\\x64\\SDL2_image.dll",
        ),
        ("bz2.dll", "vcpkg_installed\\x64-windows\\bin\\bz2.dll"),
        ("zlib1.dll", "vcpkg_installed\\x64-windows\\bin\\zlib1.dll"),
    ];

    // 复制每个 DLL
    for (dll_name, source_path) in dll_sources {
        let source = if source_path.starts_with("vcpkg_installed") {
            // 相对路径，需要基于 Cargo.toml 所在目录
            env::current_dir()
                .expect("Failed to get current dir")
                .join(source_path)
        } else {
            // 绝对路径
            PathBuf::from(source_path)
        };

        if !source.exists() {
            println!("cargo:warning=DLL not found: {}", source.display());
            continue;
        }

        let destination = target_dir.join(dll_name);

        match fs::copy(&source, &destination) {
            Ok(_) => {
                println!(
                    "cargo:warning=Copied {} to {}",
                    dll_name,
                    target_dir.display()
                );
            }
            Err(e) => {
                println!("cargo:warning=Failed to copy {}: {}", dll_name, e);
            }
        }
    }
}
