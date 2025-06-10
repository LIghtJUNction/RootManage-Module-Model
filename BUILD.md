# RMM 项目构建说明

## 项目结构

```
RootManageModuleModel/
├── pyproject.toml          # 主项目配置
├── build.py               # Python 构建脚本
├── build.ps1              # PowerShell 构建脚本
├── Makefile               # Make 构建脚本
├── src/
│   └── pyrmm/
│       ├── __init__.py    # 主包初始化
│       └── cli/           # Rust CLI 扩展
│           ├── Cargo.toml
│           ├── pyproject.toml
│           ├── __init__.py
│           └── src/
│               ├── lib.rs
│               ├── config.rs
│               ├── utils.rs
│               └── commands/
├── tests/                 # 测试文件
└── dist/                  # 构建输出（生成）
```

## 构建方法

### 1. PowerShell 构建（推荐，Windows）

```powershell
# 显示帮助
.\build.ps1

# 完整构建
.\build.ps1 build

# 开发模式构建
.\build.ps1 develop

# 只构建 Rust 扩展
.\build.ps1 build -RustOnly

# 清理构建文件
.\build.ps1 clean

# 运行测试
.\build.ps1 test

# 安装项目
.\build.ps1 install
```

### 2. Python 构建脚本

```bash
# 完整构建
python build.py build

# 开发模式构建
python build.py develop

# 只构建 Rust 扩展
python build.py build --rust-only

# 清理构建文件
python build.py clean
```

### 3. 手动构建

```bash
# 1. 构建 Rust 扩展
cd src/pyrmm/cli
maturin build --release

# 2. 复制扩展到正确位置
# Windows: 复制 target/release/*.pyd 到 cli 目录
# Linux: 复制 target/release/*.so 到 cli 目录
# macOS: 复制 target/release/*.dylib 到 cli 目录

# 3. 构建 Python 包
cd ../../..
python -m build
```

## 开发环境设置

```bash
# 1. 安装开发依赖
pip install -e ".[dev]"

# 2. 安装 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. 安装 maturin
pip install maturin

# 4. 开发模式构建
.\build.ps1 develop
```

## 使用方法

```bash
# 安装后可以直接使用
rmm --help
rmm init --help
rmm build --help
```

## 环境变量

- `RMM_ROOT`: RMM 元数据存储位置（默认: `~/data/adb/.rmm/`）
- `GITHUB_ACCESS_TOKEN`: GitHub 访问令牌

## 注意事项

1. 确保安装了 Rust 工具链
2. 确保安装了 maturin
3. 构建前请先运行 `.\build.ps1 clean` 清理旧文件
4. 开发时使用 `.\build.ps1 develop` 进行快速构建

## 故障排除

### 1. 找不到 Rust 扩展

```
ImportError: 无法找到 Rust CLI 扩展
```

解决方法：运行 `.\build.ps1 build` 重新构建

### 2. maturin 命令未找到

```
maturin: command not found
```

解决方法：`pip install maturin`

### 3. Rust 工具链未安装

解决方法：安装 Rust 工具链 https://rustup.rs/
