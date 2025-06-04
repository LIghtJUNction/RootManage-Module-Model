#!/bin/bash
# 测试NDK编译环境设置脚本

echo "🔍 测试NDK编译环境设置..."

# 源码脚本中的函数
source "./compile_git_android.sh" 2>/dev/null || {
    echo "❌ 无法加载编译脚本"
    exit 1
}

# 测试NDK环境检测
echo ""
echo "📋 测试NDK环境检测..."
if detect_ndk_environment; then
    echo "✅ NDK环境检测成功: $ANDROID_NDK_HOME"
else
    echo "⚠️  NDK环境未检测到，请设置ANDROID_NDK_HOME"
fi

# 测试NDK依赖编译环境设置
echo ""
echo "⚙️  测试NDK依赖编译环境设置..."
if setup_ndk_for_dependencies 2>/dev/null; then
    echo "✅ NDK依赖编译环境设置成功"
    echo "   工具链: $NDK_TOOLCHAIN"
    echo "   目标: $NDK_TARGET$NDK_API"
    echo "   编译器: $NDK_CC"
    
    # 验证编译器文件是否存在
    if [ -f "$NDK_CC" ]; then
        echo "✅ NDK编译器文件存在"
        # 尝试获取版本信息
        version_info=$("$NDK_CC" --version 2>/dev/null | head -n1 || echo "无法获取版本")
        echo "   版本: $version_info"
    else
        echo "❌ NDK编译器文件不存在: $NDK_CC"
    fi
else
    echo "❌ NDK依赖编译环境设置失败"
fi

echo ""
echo "🎯 NDK编译环境测试完成"
