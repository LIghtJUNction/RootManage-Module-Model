# KernelSU模块开发用户指南

## 目录
1. [快速开始](#快速开始)
2. [目录结构](#目录结构)
3. [开发工具](#开发工具)
4. [配置文件](#配置文件)
5. [最佳实践](#最佳实践)
6. [故障排除](#故障排除)

## 快速开始

### 环境准备
确保你的开发环境满足以下要求：
- Android设备已获取root权限
- 已安装KernelSU
- 具备Shell脚本基础知识

### 创建第一个模块
1. 使用模块构建工具创建基础结构：
   ```bash
   /usr/bin/module-builder --create my-first-module
   ```

2. 编辑模块属性文件：
   ```bash
   vi /data/local/tmp/my-first-module/module.prop
   ```

3. 添加功能脚本：
   ```bash
   vi /data/local/tmp/my-first-module/service.sh
   ```

4. 构建并打包模块：
   ```bash
   /usr/bin/module-builder --build /data/local/tmp/my-first-module
   /usr/bin/module-packager --package /data/local/tmp/my-first-module
   ```

## 目录结构

### 系统目录结构
```
/usr/
├── bin/                    # 系统工具
│   ├── module-builder      # 模块构建工具
│   ├── module-validator    # 模块验证工具
│   └── module-packager     # 模块打包工具
├── include/                # 头文件
│   ├── kernelsu-module.h   # KernelSU模块定义
│   └── shell-utils.h       # Shell工具定义
├── lib/                    # 系统库
│   ├── module-manager.sh   # 模块管理库
│   └── webui-helpers.sh    # WebUI辅助库
├── local/                  # 用户本地内容
│   ├── bin/                # 用户工具
│   ├── etc/                # 用户配置
│   ├── include/            # 用户头文件
│   ├── lib/                # 用户库
│   └── share/              # 用户共享资源
└── share/                  # 系统共享资源
```

### 模块目录结构
```
module/
├── META-INF/               # 签名信息
│   └── com/
│       └── google/
│           └── android/
│               ├── update-binary
│               └── updater-script
├── module.prop             # 模块属性
├── service.sh              # 启动服务脚本
├── post-fs-data.sh         # 文件系统数据脚本
├── uninstall.sh           # 卸载脚本
├── system/                # 系统文件替换
├── vendor/                # 供应商文件替换
└── webroot/               # WebUI文件
```

## 开发工具

### 模块构建工具 (module-builder)
用于创建、构建和编译KernelSU模块。

主要功能：
- 创建模块基础结构
- 编译模块代码
- 验证模块完整性
- 生成安装包

使用示例：
```bash
# 创建新模块
module-builder --create my-module

# 构建现有模块
module-builder --build /path/to/module

# 清理构建文件
module-builder --clean /path/to/module
```

### 模块验证工具 (module-validator)
用于验证模块的合法性和完整性。

验证项目：
- 文件结构检查
- 权限验证
- 签名校验
- 依赖检查

### 模块打包工具 (module-packager)
用于将模块打包成可安装的zip文件。

功能特性：
- 自动压缩模块文件
- 生成安装脚本
- 创建签名信息
- 优化包大小

### 部署工具 (module-deploy)
用于将模块部署到本地或远程设备。

部署模式：
- 本地部署：直接安装到当前设备
- 远程部署：通过SSH部署到远程设备
- 批量部署：同时部署到多个设备

### 监控工具 (module-monitor)
用于实时监控模块状态。

监控功能：
- 模块启用状态
- 资源使用情况
- 错误日志记录
- 性能指标统计

## 配置文件

### 系统配置 (kernelsu.conf)
位置：`/usr/local/etc/kernelsu.conf`

主要配置项：
- 模块路径设置
- 日志配置
- 网络设置
- 安全选项
- 构建参数

### 部署配置 (deploy.conf)
位置：`/usr/local/etc/deploy.conf`

配置内容：
- 本地部署设置
- 远程服务器配置
- 通知设置
- 回滚选项

### 用户配置
用户可以创建自己的配置文件来覆盖默认设置：
- `/data/local/tmp/user.conf` - 用户全局配置
- `/data/local/tmp/deploy.conf` - 用户部署配置

## 最佳实践

### 模块开发
1. **遵循命名规范**
   - 模块ID使用小写字母、数字和下划线
   - 文件名使用有意义的描述

2. **代码质量**
   - 添加充分的注释
   - 进行错误处理
   - 使用模块验证工具检查

3. **测试策略**
   - 在多个设备上测试
   - 验证兼容性
   - 进行压力测试

### 部署策略
1. **分阶段部署**
   - 先在测试环境验证
   - 逐步扩展到生产环境

2. **备份策略**
   - 部署前自动备份
   - 保留多个版本备份

3. **监控告警**
   - 设置关键指标监控
   - 配置异常告警

### 安全考虑
1. **权限管理**
   - 最小权限原则
   - 定期权限审查

2. **代码审查**
   - 使用代码扫描工具
   - 人工代码审查

3. **签名验证**
   - 启用签名检查
   - 使用可信密钥

## 故障排除

### 常见问题

#### 模块无法安装
1. 检查模块格式是否正确
2. 验证设备兼容性
3. 查看安装日志

#### 模块不生效
1. 确认模块已启用
2. 检查脚本权限
3. 重启设备

#### 构建失败
1. 检查依赖是否满足
2. 验证源码语法
3. 查看构建日志

### 调试技巧
1. **启用调试模式**
   ```bash
   export DEBUG_ENABLED=true
   ```

2. **查看详细日志**
   ```bash
   tail -f /data/local/tmp/logs/debug.log
   ```

3. **使用监控工具**
   ```bash
   module-monitor -d  # 后台监控
   module-monitor -t  # 查看实时日志
   ```

### 日志位置
- 系统日志：`/data/local/tmp/logs/`
- 模块日志：`/data/adb/modules/[module-id]/logs/`
- 构建日志：`/data/local/tmp/build/[module-id]/logs/`

### 获取帮助
1. 查看工具帮助文档
2. 检查示例代码
3. 访问社区论坛
4. 提交问题报告

## 高级主题

### WebUI开发
模块可以包含Web界面用于配置和管理：

1. **创建WebUI模块**
   ```bash
   module-builder --create --webui my-webui-module
   ```

2. **WebUI文件结构**
   ```
   webroot/
   ├── index.html
   ├── css/
   ├── js/
   └── api/
   ```

3. **API开发**
   使用提供的WebUI辅助库快速开发API接口。

### 模块管理API
使用模块管理库可以实现：
- 程序化模块安装
- 自动化模块管理
- 批量操作处理

### 自定义构建流程
通过修改构建配置文件，可以：
- 添加自定义构建步骤
- 集成第三方工具
- 实现持续集成

---

更多详细信息请参考：
- [KernelSU官方文档](https://kernelsu.org/)
- [模块开发示例](/usr/local/share/examples/)
- [API参考文档](/usr/include/)
