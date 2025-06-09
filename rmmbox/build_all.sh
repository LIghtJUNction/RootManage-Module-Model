#!/bin/bash
# Bash 构建脚本 - 构建所有 Rust 模块为 .pyd 文件

RMMBOX_PATH="${1:-$(dirname "$0")}"

echo "=== Building all modules in: $RMMBOX_PATH ==="

success_count=0
total_count=0

# 获取目标库目录
target_lib_dir="$(dirname "$(dirname "$RMMBOX_PATH")")/src/pyrmm/usr/lib"
mkdir -p "$target_lib_dir"

echo "Target lib directory: $target_lib_dir"

# 遍历所有子目录
for module_path in "$RMMBOX_PATH"/*; do
    if [[ ! -d "$module_path" ]]; then
        continue
    fi
    
    module_name=$(basename "$module_path")
    
    # 跳过特殊目录
    if [[ "$module_name" == .* ]] || [[ "$module_name" == "__pycache__" ]]; then
        continue
    fi
    
    ((total_count++))
    
    echo ""
    echo "=== Building module: $module_name ==="
    
    # 检查是否有 pyproject.toml
    if [[ ! -f "$module_path/pyproject.toml" ]]; then
        echo "Skipping $module_name: no pyproject.toml found"
        ((success_count++))
        continue
    fi
    
    # 切换到模块目录
    pushd "$module_path" > /dev/null
    
    if ! {
        echo "Creating virtual environment..."
        uv venv &&
        
        echo "Syncing dependencies..."
        uv sync &&
        
        echo "Building wheel..."
        uv build
    }; then
        echo "Failed to build module $module_name"
        popd > /dev/null
        continue
    fi
    
    # 查找并处理轮子文件
    if [[ -d "dist" ]]; then
        for wheel_file in dist/*.whl; do
            if [[ -f "$wheel_file" ]]; then
                echo "Extracting .pyd files from: $(basename "$wheel_file")"
                
                # 创建临时目录
                temp_dir="/tmp/wheel_extract_$$"
                mkdir -p "$temp_dir"
                
                # 解压轮子文件
                unzip -q "$wheel_file" -d "$temp_dir"
                
                # 查找并复制 .pyd 文件
                find "$temp_dir" -name "*.pyd" -exec cp {} "$target_lib_dir/" \;
                find "$temp_dir" -name "*.so" -exec cp {} "$target_lib_dir/" \;
                
                # 显示复制的文件
                for pyd_file in "$temp_dir"/**/*.pyd "$temp_dir"/**/*.so; do
                    if [[ -f "$pyd_file" ]]; then
                        echo "Copied: $(basename "$pyd_file")"
                    fi
                done
                
                # 清理临时目录
                rm -rf "$temp_dir"
            fi
        done
    fi
    
    ((success_count++))
    echo "Successfully built: $module_name"
    
    popd > /dev/null
done

echo ""
echo "=== Build Summary ==="
echo "Successfully built: $success_count/$total_count modules"

if [[ $success_count -eq $total_count ]]; then
    echo ""
    echo "=== Committing changes ==="
    repo_root=$(dirname "$RMMBOX_PATH")
    
    pushd "$repo_root" > /dev/null
    if git add . && git commit -m "Auto-build: Update .pyd files"; then
        echo "Changes committed successfully!"
    else
        echo "Failed to commit changes"
    fi
    popd > /dev/null
else
    echo "Some modules failed to build. Please check the errors above."
    exit 1
fi

