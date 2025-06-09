# PowerShell 构建脚本
param(
    [string]$RmmboxPath = $PSScriptRoot
)

Write-Host "=== Building all modules in: $RmmboxPath ===" -ForegroundColor Green

$successCount = 0
$totalCount = 0

# 获取目标库目录
$targetLibDir = Join-Path (Split-Path (Split-Path $RmmboxPath -Parent) -Parent) "src\pyrmm\usr\lib"
if (!(Test-Path $targetLibDir)) {
    New-Item -ItemType Directory -Path $targetLibDir -Force | Out-Null
}

Write-Host "Target lib directory: $targetLibDir" -ForegroundColor Cyan

# 遍历所有子目录
Get-ChildItem -Path $RmmboxPath -Directory | ForEach-Object {
    $modulePath = $_.FullName
    $moduleName = $_.Name
    
    # 跳过特殊目录
    if ($moduleName.StartsWith('.') -or $moduleName -eq '__pycache__') {
        return
    }
    
    $totalCount++
    
    Write-Host "`n=== Building module: $moduleName ===" -ForegroundColor Yellow
    
    # 检查是否有 pyproject.toml
    $pyprojectPath = Join-Path $modulePath "pyproject.toml"
    if (!(Test-Path $pyprojectPath)) {
        Write-Host "Skipping $moduleName`: no pyproject.toml found" -ForegroundColor Gray
        $successCount++
        return
    }
    
    try {
        # 切换到模块目录
        Push-Location $modulePath
        
        # 创建虚拟环境
        Write-Host "Creating virtual environment..." -ForegroundColor Cyan
        & uv venv
        if ($LASTEXITCODE -ne 0) { throw "Failed to create venv" }
        
        # 同步依赖
        Write-Host "Syncing dependencies..." -ForegroundColor Cyan
        & uv sync
        if ($LASTEXITCODE -ne 0) { throw "Failed to sync dependencies" }
        
        # 构建轮子
        Write-Host "Building wheel..." -ForegroundColor Cyan
        & uv build
        if ($LASTEXITCODE -ne 0) { throw "Failed to build wheel" }
        
        # 查找并处理轮子文件
        $distDir = Join-Path $modulePath "dist"
        if (Test-Path $distDir) {
            $wheelFiles = Get-ChildItem -Path $distDir -Filter "*.whl"
            
            foreach ($wheelFile in $wheelFiles) {
                Write-Host "Extracting .pyd files from: $($wheelFile.Name)" -ForegroundColor Cyan
                
                # 使用 PowerShell 展开压缩文件
                $tempDir = Join-Path $env:TEMP "wheel_extract_$(Get-Random)"
                Expand-Archive -Path $wheelFile.FullName -DestinationPath $tempDir -Force
                
                # 查找 .pyd 文件
                $pydFiles = Get-ChildItem -Path $tempDir -Filter "*.pyd" -Recurse
                
                foreach ($pydFile in $pydFiles) {
                    $destPath = Join-Path $targetLibDir $pydFile.Name
                    Copy-Item -Path $pydFile.FullName -Destination $destPath -Force
                    Write-Host "Copied: $($pydFile.Name)" -ForegroundColor Green
                }
                
                # 清理临时目录
                Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
            }
        }
        
        $successCount++
        Write-Host "Successfully built: $moduleName" -ForegroundColor Green
        
    } catch {
        Write-Host "Failed to build module $moduleName`: $($_.Exception.Message)" -ForegroundColor Red
    } finally {
        Pop-Location
    }
}

Write-Host "`n=== Build Summary ===" -ForegroundColor Green
Write-Host "Successfully built: $successCount/$totalCount modules" -ForegroundColor Green

if ($successCount -eq $totalCount) {
    Write-Host "`n=== Committing changes ===" -ForegroundColor Green
    $repoRoot = Split-Path $RmmboxPath -Parent
    
    try {
        Push-Location $repoRoot
        & git add .
        & git commit -m "Auto-build: Update .pyd files"
        Write-Host "Changes committed successfully!" -ForegroundColor Green
    } catch {
        Write-Host "Failed to commit changes: $($_.Exception.Message)" -ForegroundColor Red
    } finally {
        Pop-Location
    }
} else {
    Write-Host "Some modules failed to build. Please check the errors above." -ForegroundColor Red
    exit 1
}
