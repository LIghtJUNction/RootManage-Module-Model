# Android Git 编译脚本 - ARM64 NDK 依赖编译

## 🎯 更新内容

本次更新确保了OpenSSL和zlib等依赖库使用Android NDK编译ARM64架构版本，并将编译好的依赖库复制到最终的输出文件夹中。

## ✨ 主要改进

### 1. 强制NDK编译依赖库
- **OpenSSL**: 使用NDK编译ARM64版本 (`compile_openssl_ndk`)
- **zlib**: 使用NDK编译ARM64版本 (`compile_zlib_ndk`)
- **架构验证**: 自动验证编译出的库文件是否为ARM64架构

### 2. 自动NDK环境设置
- 新增 `setup_ndk_for_dependencies()` 函数
- 自动检测宿主系统 (Linux/Windows/macOS)
- 设置正确的NDK工具链路径
- 配置ARM64目标编译器

### 3. 依赖库复制到输出包
- 新增 `copy_dependencies_to_package()` 函数
- 自动复制编译好的ARM64库文件到最终输出目录
- 包含头文件和依赖信息文件
- 架构验证和信息记录

## 🛠️ 技术细节

### NDK编译器设置
```bash
NDK_TARGET="aarch64-linux-android"
NDK_API="21"  # Android 5.0+
NDK_CC="$NDK_TOOLCHAIN/bin/${NDK_TARGET}${NDK_API}-clang"
NDK_CXX="$NDK_TOOLCHAIN/bin/${NDK_TARGET}${NDK_API}-clang++"
```

### OpenSSL配置
- 目标平台: `android-arm64`
- 编译选项: `no-shared no-tests no-ui-console`
- API级别: `-D__ANDROID_API__=21`

### zlib配置
- 静态编译: `--static`
- 编译标志: `-fPIC -O2 -DANDROID -D__ANDROID_API__=21`

## 📁 输出文件结构

编译完成后，最终输出包将包含：

```
git-android-<version>-<mode>-<date>/
├── bin/                    # Git可执行文件
├── lib/                    # ARM64依赖库
│   ├── libz.a             # zlib (ARM64)
│   ├── libssl.a           # OpenSSL SSL (ARM64)
│   └── libcrypto.a        # OpenSSL Crypto (ARM64)
├── include/               # 头文件
│   ├── openssl/           # OpenSSL头文件
│   ├── zlib.h             # zlib头文件
│   └── zconf.h            # zlib配置头文件
└── DEPENDENCIES_INFO.txt  # 依赖库详细信息
```

## 🔧 使用方法

### 前置要求
1. **Android NDK**: 下载并设置 `ANDROID_NDK_HOME` 环境变量
2. **支持的NDK版本**: r21+ (推荐r25+)
3. **目标API**: Android API 21+ (Android 5.0+)

### 编译命令
```bash
# 自动模式（推荐）
./compile_git_android.sh auto

# 手动选择模式
./compile_git_android.sh

# 测试NDK环境
./test_ndk_compilation.sh
```

### 环境变量设置
```bash
# Linux/macOS
export ANDROID_NDK_HOME="/path/to/android-ndk-r25c"

# Windows (PowerShell)
$env:ANDROID_NDK_HOME = "C:\Android\SDK\ndk\25.2.9519653"

# Windows (CMD)
set ANDROID_NDK_HOME=C:\Android\SDK\ndk\25.2.9519653
```

## ✅ 架构验证

脚本会自动验证编译的库文件架构：

```bash
# 验证示例输出
✓ libz.a (156K, aarch64)
✓ libssl.a (2.1M, aarch64) 
✓ libcrypto.a (4.8M, aarch64)
```

## 🚀 编译模式

所有编译模式都会强制使用NDK编译依赖库：

- **termux_native**: Termux本机编译 + NDK依赖
- **android_native**: Android原生编译 + NDK依赖  
- **static**: 静态交叉编译 + NDK依赖
- **ndk**: 完全NDK编译
- **minimal**: 最小化编译 + NDK核心依赖

## 📋 依赖信息文件

`DEPENDENCIES_INFO.txt` 包含详细的编译信息：

```
Git Android Dependencies Information
==================================

编译时间: 2025-06-04 10:30:00
编译模式: static
目标架构: ARM64 (aarch64)

NDK编译工具链:
- NDK路径: /opt/android-ndk-r25c
- 工具链: /opt/android-ndk-r25c/toolchains/llvm/prebuilt/linux-x86_64
- 目标: aarch64-linux-android21
- 编译器: .../aarch64-linux-android21-clang

包含的ARM64依赖库:
- libz.a (156K, aarch64)
- libssl.a (2.1M, aarch64)
- libcrypto.a (4.8M, aarch64)
```

## 🛡️ 质量保证

- ✅ 强制ARM64架构编译
- ✅ NDK工具链验证
- ✅ 库文件架构检查
- ✅ 编译器版本验证
- ✅ 依赖完整性检查

## 🐛 故障排除

### NDK未找到
```bash
❌ 编译Android依赖需要NDK环境
解决方案: 设置ANDROID_NDK_HOME环境变量
```

### 编译器不存在
```bash
❌ NDK编译器不存在: /path/to/clang
解决方案: 检查NDK版本和安装完整性
```

### 架构验证失败
```bash
⚠ libz.a: 架构检测失败，请手动验证
解决方案: 使用 file 命令手动检查库文件架构
```

## 📞 支持

如果遇到问题，请检查：
1. NDK版本是否兼容 (推荐r25+)
2. 环境变量设置是否正确
3. 宿主系统是否支持 (Linux/Windows/macOS)
4. 磁盘空间是否充足 (至少2GB)

---

🎉 现在您的Git编译将包含正确编译的ARM64依赖库，确保在Android设备上的最佳兼容性！
