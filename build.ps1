# RMM 项目构建脚本 (PowerShell)

param(
    [Parameter(Position=0)]
    [ValidateSet("build", "develop", "clean", "test", "install")]
    [string]$Command = "help",
    
    [switch]$RustOnly
)

function Write-Step {
    param([string]$Message)
    Write-Host "🔨 $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "✅ $Message" -ForegroundColor Green
}

function Write-Error {
    param([string]$Message)
    Write-Host "❌ $Message" -ForegroundColor Red
}

function Invoke-Command-Safe {
    param(
        [string[]]$Command,
        [string]$WorkingDirectory = $null
    )
    
    $originalLocation = Get-Location
    
    try {
        if ($WorkingDirectory) {
            Set-Location $WorkingDirectory
        }
        
        Write-Host "运行命令: $($Command -join ' ')" -ForegroundColor Yellow
        if ($WorkingDirectory) {
            Write-Host "工作目录: $WorkingDirectory" -ForegroundColor Yellow
        }
        
        & $Command[0] $Command[1..($Command.Length-1)]
        
        if ($LASTEXITCODE -ne 0) {
            throw "命令失败，退出代码: $LASTEXITCODE"
        }
    }
    finally {
        Set-Location $originalLocation
    }
}

function Build-RustExtension {
    Write-Step "构建 Rust CLI 扩展..."
    
    $cliDir = "src\pyrmm\cli"
    if (-not (Test-Path $cliDir)) {
        Write-Error "CLI 目录不存在: $cliDir"
        exit 1
    }
    
    # 构建 Rust 扩展
    Invoke-Command-Safe @("maturin", "build", "--release") -WorkingDirectory $cliDir
    
    # 查找构建产物
    $targetDir = Join-Path $cliDir "target\release"
    $builtFiles = Get-ChildItem -Path $targetDir -Filter "*.pyd" -ErrorAction SilentlyContinue
    
    if (-not $builtFiles) {
        Write-Error "未找到编译产物 (*.pyd)"
        return $false
    }
    
    # 复制到目标位置
    $targetFile = Join-Path $cliDir "pyrmm_cli.pyd"
    Copy-Item $builtFiles[0].FullName $targetFile -Force
    Write-Success "复制 $($builtFiles[0].FullName) -> $targetFile"
    
    return $true
}

function Build-PythonPackage {
    Write-Step "构建 Python 包..."
    
    # 清理旧的构建文件
    if (Test-Path "dist") {
        Remove-Item -Recurse -Force "dist"
    }
    
    # 构建包
    Invoke-Command-Safe @("python", "-m", "build")
    
    Write-Success "Python 包构建完成"
}

function Build-DevelopMode {
    Write-Step "开发模式构建..."
    
    $cliDir = "src\pyrmm\cli"
    
    # 使用 maturin develop 进行开发构建
    Invoke-Command-Safe @("maturin", "develop") -WorkingDirectory $cliDir
    
    Write-Success "开发模式构建完成"
}

function Clean-BuildFiles {
    Write-Step "清理构建文件..."
    
    # 清理目录列表
    $cleanDirs = @(
        "dist",
        "build",
        "src\pyrmm.egg-info",
        "src\pyrmm\cli\target"
    )
    
    foreach ($dir in $cleanDirs) {
        if (Test-Path $dir) {
            Remove-Item -Recurse -Force $dir
            Write-Host "删除目录: $dir"
        }
    }
    
    # 清理文件
    $cleanFiles = Get-ChildItem -Path "src\pyrmm\cli" -Filter "*.pyd" -ErrorAction SilentlyContinue
    foreach ($file in $cleanFiles) {
        Remove-Item $file.FullName -Force
        Write-Host "删除文件: $($file.FullName)"
    }
    
    Write-Success "清理完成"
}

function Show-Help {
    Write-Host @"
RMM 项目构建脚本

用法: .\build.ps1 [命令] [选项]

命令:
  build      - 构建完整项目（Rust + Python）
  develop    - 开发模式构建
  clean      - 清理构建文件
  test       - 运行测试
  install    - 安装项目

选项:
  -RustOnly  - 只构建 Rust 扩展

示例:
  .\build.ps1 build
  .\build.ps1 develop
  .\build.ps1 build -RustOnly
  .\build.ps1 clean
"@ -ForegroundColor White
}

# 主逻辑
switch ($Command) {
    "build" {
        if ($RustOnly) {
            $success = Build-RustExtension
            if (-not $success) {
                Write-Error "Rust 扩展构建失败"
                exit 1
            }
        } else {
            $success = Build-RustExtension
            if ($success) {
                Build-PythonPackage
            } else {
                Write-Error "Rust 扩展构建失败"
                exit 1
            }
        }
    }
    "develop" {
        Build-DevelopMode
    }
    "clean" {
        Clean-BuildFiles
    }
    "test" {
        Build-DevelopMode
        Invoke-Command-Safe @("python", "-m", "pytest", "tests/", "-v")
    }
    "install" {
        # 先构建再安装
        $success = Build-RustExtension
        if ($success) {
            Build-PythonPackage
            $wheelFiles = Get-ChildItem -Path "dist" -Filter "*.whl"
            if ($wheelFiles) {
                Invoke-Command-Safe @("pip", "install", $wheelFiles[0].FullName)
            } else {
                Write-Error "未找到 wheel 文件"
                exit 1
            }
        } else {
            Write-Error "构建失败"
            exit 1
        }
    }
    default {
        Show-Help
    }
}
