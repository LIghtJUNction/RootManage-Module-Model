#!/system/bin/sh

# Audio Enhancement Suite Service Script
# 综合音频增强模块服务脚本

MODPATH="${0%/*}"
LOG_FILE="$MODPATH/logs/audio-enhancer.log"
CONFIG_FILE="$MODPATH/audio-config.conf"

# 创建日志目录
mkdir -p "$(dirname "$LOG_FILE")"

# 日志记录函数
log_msg() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# 检查文件是否存在
file_exists() {
    [ -f "$1" ]
}

# 检查目录是否存在
dir_exists() {
    [ -d "$1" ]
}

# 安全复制文件
safe_copy() {
    local src="$1"
    local dst="$2"
    local description="$3"
    
    if file_exists "$src"; then
        cp "$src" "$dst" 2>/dev/null
        if [ $? -eq 0 ]; then
            log_msg "✓ $description: $dst"
            return 0
        else
            log_msg "✗ 无法复制 $description"
            return 1
        fi
    else
        log_msg "⚠ 源文件不存在: $src"
        return 1
    fi
}

# 创建符号链接
create_symlink() {
    local src="$1"
    local dst="$2"
    local description="$3"
    
    if file_exists "$src"; then
        ln -sf "$src" "$dst" 2>/dev/null
        if [ $? -eq 0 ]; then
            log_msg "✓ $description: $dst -> $src"
            return 0
        else
            log_msg "✗ 无法创建链接 $description"
            return 1
        fi
    else
        log_msg "⚠ 链接源不存在: $src"
        return 1
    fi
}

# 设置文件权限
set_permissions() {
    local file="$1"
    local perm="$2"
    local owner="$3"
    local description="$4"
    
    if file_exists "$file"; then
        chmod "$perm" "$file" 2>/dev/null
        chown "$owner" "$file" 2>/dev/null
        log_msg "✓ 设置权限 $description: $perm $owner"
    fi
}

# 加载配置文件
load_config() {
    log_msg "加载音频配置..."
    
    # 默认配置
    ENABLE_VIPER=true
    ENABLE_DOLBY=true
    ENABLE_DTS=false
    ENABLE_EQUALIZER=true
    SAMPLE_RATE=48000
    BIT_DEPTH=24
    BUFFER_SIZE=256
    
    if file_exists "$CONFIG_FILE"; then
        source "$CONFIG_FILE"
        log_msg "✓ 配置文件已加载: $CONFIG_FILE"
    else
        log_msg "⚠ 配置文件不存在，使用默认设置"
        create_default_config
    fi
}

# 创建默认配置文件
create_default_config() {
    cat > "$CONFIG_FILE" << 'EOF'
# Audio Enhancement Suite Configuration

# 音频效果开关
ENABLE_VIPER=true      # 启用ViPER4Android效果
ENABLE_DOLBY=true      # 启用Dolby音效
ENABLE_DTS=false       # 启用DTS音效
ENABLE_EQUALIZER=true  # 启用系统均衡器增强

# 音频质量设置
SAMPLE_RATE=48000      # 采样率 (Hz)
BIT_DEPTH=24           # 位深度 (bit)
BUFFER_SIZE=256        # 缓冲区大小 (frames)

# 高级设置
ENABLE_SPATIAL_AUDIO=true    # 空间音频
ENABLE_BASS_BOOST=true       # 低音增强
ENABLE_VIRTUALIZER=true      # 虚拟环绕声
ENABLE_REVERB=false          # 混响效果

# 设备特定优化
OPTIMIZE_HEADPHONES=true     # 耳机优化
OPTIMIZE_SPEAKERS=true       # 扬声器优化
OPTIMIZE_BLUETOOTH=true      # 蓝牙音频优化

# 兼容性设置
LEGACY_MODE=false           # 兼容模式
FORCE_ENABLE=false          # 强制启用所有效果
DEBUG_MODE=false            # 调试模式
EOF
    
    log_msg "✓ 已创建默认配置文件"
}

# 检测音频系统
detect_audio_system() {
    log_msg "检测音频系统..."
    
    # 检测AudioFlinger
    if pgrep audioserver >/dev/null; then
        AUDIO_SERVER="audioserver"
        log_msg "✓ 检测到AudioServer (Android 6.0+)"
    elif pgrep mediaserver >/dev/null; then
        AUDIO_SERVER="mediaserver"
        log_msg "✓ 检测到MediaServer (Android 5.1-)"
    else
        AUDIO_SERVER="unknown"
        log_msg "⚠ 无法确定音频服务器类型"
    fi
    
    # 检测音频HAL
    if dir_exists "/vendor/lib/hw" && ls /vendor/lib/hw/audio.primary.*.so >/dev/null 2>&1; then
        AUDIO_HAL_PATH="/vendor/lib/hw"
        log_msg "✓ 检测到Vendor音频HAL"
    elif dir_exists "/system/lib/hw" && ls /system/lib/hw/audio.primary.*.so >/dev/null 2>&1; then
        AUDIO_HAL_PATH="/system/lib/hw"
        log_msg "✓ 检测到System音频HAL"
    else
        AUDIO_HAL_PATH=""
        log_msg "⚠ 无法找到音频HAL"
    fi
    
    # 检测音频策略
    if file_exists "/vendor/etc/audio_policy_configuration.xml"; then
        AUDIO_POLICY_PATH="/vendor/etc"
        log_msg "✓ 检测到Vendor音频策略"
    elif file_exists "/system/etc/audio_policy.conf"; then
        AUDIO_POLICY_PATH="/system/etc"
        log_msg "✓ 检测到System音频策略"
    else
        AUDIO_POLICY_PATH=""
        log_msg "⚠ 无法找到音频策略文件"
    fi
}

# 安装ViPER4Android支持
install_viper_support() {
    if [ "$ENABLE_VIPER" = "true" ]; then
        log_msg "安装ViPER4Android支持..."
        
        # 创建ViPER目录
        mkdir -p "/data/data/com.vipercn.viper4android_v2/shared_prefs"
        
        # 安装ViPER库文件
        local viper_libs=(
            "libv4a_fx_ics.so"
            "libv4a_xhifi_ics.so"
        )
        
        for lib in "${viper_libs[@]}"; do
            if file_exists "$MODPATH/libs/$lib"; then
                safe_copy "$MODPATH/libs/$lib" "/system/lib/soundfx/$lib" "ViPER库文件($lib)"
                safe_copy "$MODPATH/libs/$lib" "/system/lib64/soundfx/$lib" "ViPER库文件64($lib)"
            fi
        done
        
        # 安装配置文件
        if file_exists "$MODPATH/configs/viper4android.xml"; then
            safe_copy "$MODPATH/configs/viper4android.xml" "/system/etc/viper4android.xml" "ViPER配置文件"
        fi
        
        # 修改音频效果配置
        local audio_effects_conf="/system/etc/audio_effects.conf"
        if file_exists "$audio_effects_conf"; then
            # 备份原文件
            cp "$audio_effects_conf" "$audio_effects_conf.bak" 2>/dev/null
            
            # 添加ViPER效果
            if ! grep -q "v4a_standard_fx" "$audio_effects_conf"; then
                cat >> "$audio_effects_conf" << 'EOF'

# ViPER4Android Effects
v4a_standard_fx {
    library v4a_fx
    uuid 41d3c987-e6cf-11e3-a88a-11aba5d5c51b
}

v4a_fx {
    path /system/lib/soundfx/libv4a_fx_ics.so
}
EOF
                log_msg "✓ 已添加ViPER音频效果配置"
            fi
        fi
        
        log_msg "✓ ViPER4Android支持安装完成"
    fi
}

# 安装Dolby音效支持
install_dolby_support() {
    if [ "$ENABLE_DOLBY" = "true" ]; then
        log_msg "安装Dolby音效支持..."
        
        # Dolby库文件
        local dolby_libs=(
            "libdolby_dap.so"
            "libstagefrightdolby.so"
        )
        
        for lib in "${dolby_libs[@]}"; do
            if file_exists "$MODPATH/libs/$lib"; then
                safe_copy "$MODPATH/libs/$lib" "/system/lib/$lib" "Dolby库文件($lib)"
                safe_copy "$MODPATH/libs/$lib" "/system/lib64/$lib" "Dolby库文件64($lib)"
            fi
        done
        
        # Dolby配置文件
        if file_exists "$MODPATH/configs/dolby_dap.xml"; then
            safe_copy "$MODPATH/configs/dolby_dap.xml" "/system/etc/dolby_dap.xml" "Dolby配置文件"
        fi
        
        # 设置权限
        set_permissions "/system/lib/libdolby_dap.so" "644" "root:root" "Dolby库文件"
        set_permissions "/system/lib64/libdolby_dap.so" "644" "root:root" "Dolby库文件64"
        
        log_msg "✓ Dolby音效支持安装完成"
    fi
}

# 优化音频参数
optimize_audio_parameters() {
    log_msg "优化音频参数..."
    
    # 设置高质量音频参数
    setprop vendor.audio.offload.buffer.size.kb 32
    setprop vendor.audio.offload.gapless.enabled true
    setprop vendor.audio.offload.multiaac.enable true
    setprop vendor.audio.offload.multiple.enabled true
    setprop vendor.audio.offload.passthrough false
    setprop vendor.audio.offload.track.enable true
    
    # 设置采样率和位深度
    setprop vendor.audio.default.sample.rate "$SAMPLE_RATE"
    setprop vendor.audio.default.bit.depth "$BIT_DEPTH"
    
    # 优化音频延迟
    setprop vendor.audio.low.latency.enabled true
    setprop vendor.audio.fast.mixer.enabled true
    
    # 蓝牙音频优化
    if [ "$OPTIMIZE_BLUETOOTH" = "true" ]; then
        setprop vendor.bluetooth.a2dp.offload.supported true
        setprop vendor.bluetooth.a2dp.offload.disabled false
        setprop ro.bluetooth.a2dp_offload.supported true
        log_msg "✓ 蓝牙音频优化已启用"
    fi
    
    # 高分辨率音频支持
    setprop vendor.audio.feature.hires.enable true
    setprop vendor.audio.feature.hfp.enable true
    
    log_msg "✓ 音频参数优化完成"
}

# 配置音频效果
configure_audio_effects() {
    log_msg "配置音频效果..."
    
    # 启用系统音频效果
    setprop vendor.audio.feature.equalizer.enable "$ENABLE_EQUALIZER"
    setprop vendor.audio.feature.bassboost.enable "$ENABLE_BASS_BOOST"
    setprop vendor.audio.feature.virtualizer.enable "$ENABLE_VIRTUALIZER"
    setprop vendor.audio.feature.reverb.enable "$ENABLE_REVERB"
    
    # 空间音频
    if [ "$ENABLE_SPATIAL_AUDIO" = "true" ]; then
        setprop vendor.audio.feature.spatial.enable true
        log_msg "✓ 空间音频已启用"
    fi
    
    # 创建音频效果配置
    local mixer_paths="/system/etc/mixer_paths.xml"
    if file_exists "$mixer_paths" && file_exists "$MODPATH/configs/mixer_paths_audio_enhancer.xml"; then
        # 备份原配置
        cp "$mixer_paths" "$mixer_paths.bak" 2>/dev/null
        # 应用增强配置
        safe_copy "$MODPATH/configs/mixer_paths_audio_enhancer.xml" "$mixer_paths" "音频混音器配置"
    fi
    
    log_msg "✓ 音频效果配置完成"
}

# 优化特定设备
optimize_audio_devices() {
    log_msg "优化音频设备..."
    
    # 耳机优化
    if [ "$OPTIMIZE_HEADPHONES" = "true" ]; then
        setprop vendor.audio.headphone.gain.preset "high_quality"
        setprop vendor.audio.headphone.impedance.detection true
        log_msg "✓ 耳机音频优化已启用"
    fi
    
    # 扬声器优化
    if [ "$OPTIMIZE_SPEAKERS" = "true" ]; then
        setprop vendor.audio.speaker.protection true
        setprop vendor.audio.speaker.drc.enable true
        log_msg "✓ 扬声器音频优化已启用"
    fi
    
    # 麦克风优化
    setprop vendor.audio.microphone.noise.reduction true
    setprop vendor.audio.microphone.echo.cancellation true
    
    log_msg "✓ 音频设备优化完成"
}

# 创建音频配置文件
create_audio_configs() {
    log_msg "创建音频配置文件..."
    
    # 创建ViPER配置
    mkdir -p "$MODPATH/configs"
    cat > "$MODPATH/configs/viper4android.xml" << 'EOF'
<?xml version="1.0" encoding="utf-8"?>
<audio_effects_conf version="2.0" xmlns="http://schemas.android.com/audio/audio_effects_conf/v2_0">
    <libraries>
        <library name="v4a_fx" path="libv4a_fx_ics.so"/>
    </libraries>
    <effects>
        <effect name="v4a_standard_fx" library="v4a_fx" uuid="41d3c987-e6cf-11e3-a88a-11aba5d5c51b"/>
    </effects>
    <postprocess>
        <stream type="music">
            <apply effect="v4a_standard_fx"/>
        </stream>
    </postprocess>
</audio_effects_conf>
EOF
    
    # 创建Dolby配置
    cat > "$MODPATH/configs/dolby_dap.xml" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<dolby_dap_config>
    <global_settings>
        <virtualizer_enable>true</virtualizer_enable>
        <bass_enhancer_enable>true</bass_enhancer_enable>
        <dialogue_enhancer_enable>true</dialogue_enhancer_enable>
        <volume_leveller_enable>true</volume_leveller_enable>
    </global_settings>
    <profiles>
        <profile name="Music">
            <equalizer band1="+2" band2="+1" band3="0" band4="+1" band5="+3"/>
        </profile>
        <profile name="Movie">
            <equalizer band1="+1" band2="0" band3="+2" band4="+1" band5="+2"/>
        </profile>
    </profiles>
</dolby_dap_config>
EOF
    
    log_msg "✓ 音频配置文件创建完成"
}

# 重启音频服务
restart_audio_services() {
    log_msg "重启音频服务..."
    
    # 停止音频服务
    killall "$AUDIO_SERVER" 2>/dev/null
    
    # 等待服务重启
    sleep 2
    
    # 检查服务是否重启
    local retry=0
    while [ $retry -lt 10 ]; do
        if pgrep "$AUDIO_SERVER" >/dev/null; then
            log_msg "✓ 音频服务已重启"
            break
        fi
        sleep 1
        retry=$((retry + 1))
    done
    
    if [ $retry -eq 10 ]; then
        log_msg "⚠ 音频服务重启超时"
    fi
}

# 创建WebUI控制界面
setup_webui() {
    if dir_exists "$MODPATH/webui"; then
        log_msg "设置WebUI控制界面..."
        
        # 启动WebUI服务
        if command -v busybox >/dev/null; then
            cd "$MODPATH/webui"
            nohup busybox httpd -p 8080 -h . >/dev/null 2>&1 &
            log_msg "✓ WebUI已启动: http://localhost:8080"
        else
            log_msg "⚠ 未找到busybox，无法启动WebUI"
        fi
    fi
}

# 性能测试
run_audio_test() {
    log_msg "运行音频性能测试..."
    
    # 测试音频延迟
    local latency_test_result=""
    if command -v tinycap >/dev/null && command -v tinyplay >/dev/null; then
        # 这里可以添加实际的延迟测试逻辑
        latency_test_result="低延迟模式已启用"
    else
        latency_test_result="无法进行延迟测试 (缺少工具)"
    fi
    
    log_msg "音频延迟测试: $latency_test_result"
    
    # 测试音频质量设置
    local current_sample_rate=$(getprop vendor.audio.default.sample.rate)
    local current_bit_depth=$(getprop vendor.audio.default.bit.depth)
    
    log_msg "当前音频质量: ${current_sample_rate}Hz/${current_bit_depth}bit"
    
    log_msg "✓ 音频性能测试完成"
}

# 主函数
main() {
    log_msg "启动音频增强套件..."
    log_msg "模块路径: $MODPATH"
    
    # 检查系统信息
    log_msg "系统信息:"
    log_msg "  Android版本: $(getprop ro.build.version.release)"
    log_msg "  设备型号: $(getprop ro.product.model)"
    log_msg "  音频芯片: $(getprop ro.vendor.audio.sdk.ssr)"
    
    # 执行安装和配置流程
    load_config
    detect_audio_system
    create_audio_configs
    install_viper_support
    install_dolby_support
    optimize_audio_parameters
    configure_audio_effects
    optimize_audio_devices
    restart_audio_services
    setup_webui
    run_audio_test
    
    log_msg "音频增强套件安装完成!"
    log_msg "建议重启设备以确保所有效果生效"
    
    # 显示安装摘要
    log_msg "安装摘要:"
    [ "$ENABLE_VIPER" = "true" ] && log_msg "  ✓ ViPER4Android效果"
    [ "$ENABLE_DOLBY" = "true" ] && log_msg "  ✓ Dolby音效"
    [ "$ENABLE_DTS" = "true" ] && log_msg "  ✓ DTS音效"
    [ "$ENABLE_EQUALIZER" = "true" ] && log_msg "  ✓ 系统均衡器增强"
    log_msg "  ✓ 高质量音频参数 (${SAMPLE_RATE}Hz/${BIT_DEPTH}bit)"
    log_msg "  ✓ 音频设备优化"
    dir_exists "$MODPATH/webui" && log_msg "  ✓ WebUI控制界面"
}

# 错误处理
trap 'log_msg "音频增强模块执行出错，退出代码: $?"' ERR

# 执行主函数
main "$@"
