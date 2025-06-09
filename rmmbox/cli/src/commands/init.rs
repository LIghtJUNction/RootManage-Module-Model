use clap::{Arg, ArgMatches, Command};
use anyhow::Result;
use std::path::PathBuf;
use crate::utils::{Context, RmmProject, ensure_dir_exists};

pub fn init_command() -> Command {
    Command::new("init")
        .about("初始化RMM项目")
        .arg(
            Arg::new("project_path")
                .help("项目路径 (默认为当前目录)")
                .value_name("PROJECT_PATH")
                .default_value(".")
        )
        .arg(
            Arg::new("yes")
                .short('y')
                .long("yes")
                .action(clap::ArgAction::SetTrue)
                .help("跳过确认提示")
        )
        .arg(
            Arg::new("basic")
                .short('b')
                .long("basic")
                .action(clap::ArgAction::SetTrue)
                .help("初始化一个基本的RMM项目")
        )
        .arg(
            Arg::new("lib")
                .short('l')
                .long("lib")
                .action(clap::ArgAction::SetTrue)
                .help("初始化一个RMM库项目(模块)")
        )
        .arg(
            Arg::new("ravd")
                .short('r')
                .long("ravd")
                .action(clap::ArgAction::SetTrue)
                .help("初始化一个RMM Android Virtual Device (RAVD)(测试模块用安卓虚拟系统)")
        )
}

pub fn handle_init(ctx: &Context, matches: &ArgMatches) -> Result<()> {
    let project_path = matches.get_one::<String>("project_path").unwrap();
    let yes = matches.get_flag("yes");
    let basic = matches.get_flag("basic");
    let lib = matches.get_flag("lib");
    let ravd = matches.get_flag("ravd");

    let rpath = PathBuf::from(project_path).canonicalize()
        .unwrap_or_else(|_| PathBuf::from(project_path));

    // 确定项目类型
    let rtype = if lib {
        "library"
    } else if ravd {
        "ravd"
    } else {
        "basic"
    };

    ctx.info(&format!("正在初始化 {} 类型的RMM项目到: {}", rtype, rpath.display()));

    // 检查目录是否已存在rmm.toml
    let rmm_toml = rpath.join("rmm.toml");
    if rmm_toml.exists() && !yes {
        ctx.warn("目录中已存在 rmm.toml 文件");
        if !confirm_overwrite()? {
            ctx.info("操作已取消");
            return Ok(());
        }
    }

    // 确保目录存在
    ensure_dir_exists(&rpath)?;

    match rtype {
        "basic" => {
            init_basic_project(&rpath, ctx)?;
            ctx.info("✅ 基本项目初始化完成");
        }
        "library" => {
            init_library_project(&rpath, ctx)?;
            ctx.info("✅ 库项目初始化完成");
        }
        "ravd" => {
            init_ravd_project(&rpath, ctx)?;
            ctx.info("✅ RAVD项目初始化完成");
        }
        _ => {
            anyhow::bail!("❌ 无效的模块类型");
        }
    }

    ctx.info(&format!("✅ 项目 '{}' 初始化完成！", rpath.file_name().unwrap().to_string_lossy()));

    Ok(())
}

fn confirm_overwrite() -> Result<bool> {
    use std::io::{self, Write};

    print!("是否要覆盖现有的配置? (y/N): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes")
}

fn init_basic_project(project_path: &std::path::Path, ctx: &Context) -> Result<()> {
    let project_name = project_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("rmm-project")
        .to_string();

    // 创建rmm.toml配置文件
    let rmm_project = RmmProject {
        name: project_name.clone(),
        version: "1.0.0".to_string(),
        author: Some("Your Name".to_string()),
        description: Some(format!("A basic RMM module: {}", project_name)),
        id: Some(project_name.clone()),
        versionCode: Some(1),
        updateJson: None,
        dependencies: None,
        build: None,
    };

    let rmm_toml_path = project_path.join("rmm.toml");
    rmm_project.save_to_file(&rmm_toml_path)?;

    // 创建基本的模块文件
    create_basic_files(project_path, &project_name, ctx)?;

    ctx.debug(&format!("创建了配置文件: {}", rmm_toml_path.display()));

    Ok(())
}

fn init_library_project(project_path: &std::path::Path, ctx: &Context) -> Result<()> {
    let project_name = project_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("rmm-library")
        .to_string();

    // 创建rmm.toml配置文件
    let rmm_project = RmmProject {
        name: project_name.clone(),
        version: "1.0.0".to_string(),
        author: Some("Your Name".to_string()),
        description: Some(format!("A RMM library module: {}", project_name)),
        id: Some(format!("{}.lib", project_name)),
        versionCode: Some(1),
        updateJson: None,
        dependencies: None,
        build: None,
    };

    let rmm_toml_path = project_path.join("rmm.toml");
    rmm_project.save_to_file(&rmm_toml_path)?;

    // 创建库模块文件
    create_library_files(project_path, &project_name, ctx)?;

    ctx.debug(&format!("创建了库项目配置文件: {}", rmm_toml_path.display()));

    Ok(())
}

fn init_ravd_project(project_path: &std::path::Path, ctx: &Context) -> Result<()> {
    let project_name = project_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("rmm-ravd")
        .to_string();

    // 创建rmm.toml配置文件
    let rmm_project = RmmProject {
        name: project_name.clone(),
        version: "1.0.0".to_string(),
        author: Some("Your Name".to_string()),
        description: Some(format!("A RMM Android Virtual Device: {}", project_name)),
        id: Some(format!("{}.ravd", project_name)),
        versionCode: Some(1),
        updateJson: None,
        dependencies: None,
        build: None,
    };

    let rmm_toml_path = project_path.join("rmm.toml");
    rmm_project.save_to_file(&rmm_toml_path)?;

    // 创建RAVD相关文件
    create_ravd_files(project_path, &project_name, ctx)?;

    ctx.debug(&format!("创建了RAVD项目配置文件: {}", rmm_toml_path.display()));

    Ok(())
}

fn create_basic_files(project_path: &std::path::Path, project_name: &str, ctx: &Context) -> Result<()> {
    // 创建META-INF目录和com/google/android/update-binary
    let meta_inf_dir = project_path.join("META-INF").join("com").join("google").join("android");
    ensure_dir_exists(&meta_inf_dir)?;

    // 创建update-binary脚本
    let update_binary_content = include_str!("../templates/update-binary.sh");
    std::fs::write(meta_inf_dir.join("update-binary"), update_binary_content)?;

    // 创建updater-script（可选）
    let updater_script_content = "#MAGISK\n";
    std::fs::write(meta_inf_dir.join("updater-script"), updater_script_content)?;

    // 创建customize.sh
    let customize_sh_content = format!(r#"#!/system/bin/sh
# {project_name} 自定义安装脚本

# 基本安装逻辑
ui_print "正在安装 {project_name}..."
ui_print "版本: 1.0.0"

# 设置权限
set_perm_recursive $MODPATH 0 0 0755 0644

ui_print "{project_name} 安装完成!"
"#, project_name = project_name);
    std::fs::write(project_path.join("customize.sh"), customize_sh_content)?;

    // 创建service.sh
    let service_sh_content = format!(r#"#!/system/bin/sh
# {project_name} 服务脚本

# 在这里添加需要在每次启动时运行的代码
"#, project_name = project_name);
    std::fs::write(project_path.join("service.sh"), service_sh_content)?;

    // 创建post-fs-data.sh
    let post_fs_data_content = format!(r#"#!/system/bin/sh
# {project_name} post-fs-data 脚本

# 在这里添加需要在post-fs-data阶段运行的代码
"#, project_name = project_name);
    std::fs::write(project_path.join("post-fs-data.sh"), post_fs_data_content)?;

    // 创建uninstall.sh
    let uninstall_sh_content = format!(r#"#!/system/bin/sh
# {project_name} 卸载脚本

# 在这里添加卸载时需要执行的清理代码
"#, project_name = project_name);
    std::fs::write(project_path.join("uninstall.sh"), uninstall_sh_content)?;

    // 创建system目录
    ensure_dir_exists(&project_path.join("system"))?;

    // 创建README.md
    let readme_content = format!(r#"# {project_name}

这是一个基本的RMM（Root Module Manager）项目。

## 项目结构

- `rmm.toml` - 项目配置文件
- `customize.sh` - 自定义安装脚本
- `service.sh` - 服务脚本（每次启动时运行）
- `post-fs-data.sh` - post-fs-data阶段脚本
- `uninstall.sh` - 卸载脚本
- `system/` - 系统文件目录
- `META-INF/` - 安装包元数据

## 构建

```bash
rmm build
```

## 安装

构建完成后，将生成的zip文件通过Magisk Manager安装。
"#, project_name = project_name);
    std::fs::write(project_path.join("README.md"), readme_content)?;

    ctx.debug("创建了基本项目文件");

    Ok(())
}

fn create_library_files(project_path: &std::path::Path, project_name: &str, ctx: &Context) -> Result<()> {
    // 库项目主要是提供给其他模块使用的
    
    // 创建lib目录
    let lib_dir = project_path.join("lib");
    ensure_dir_exists(&lib_dir)?;

    // 创建库主文件
    let lib_main_content = format!(r#"#!/system/bin/sh
# {project_name} 库文件

# 在这里定义库函数
library_function() {{
    echo "这是来自 {project_name} 的库函数"
}}

# 导出函数
export -f library_function
"#, project_name = project_name);
    std::fs::write(lib_dir.join("main.sh"), lib_main_content)?;

    // 创建包含目录
    let include_dir = project_path.join("include");
    ensure_dir_exists(&include_dir)?;

    // 创建头文件
    let header_content = format!(r#"# {project_name} 头文件

# 常量定义
LIBRARY_VERSION="1.0.0"
LIBRARY_NAME="{project_name}"

# 函数声明
# library_function - 示例库函数
"#, project_name = project_name);
    std::fs::write(include_dir.join("main.h"), header_content)?;

    // 创建README.md
    let readme_content = format!(r#"# {project_name}

这是一个RMM库项目，提供可重用的功能给其他RMM模块。

## 使用方法

在其他RMM项目的rmm.toml中添加依赖：

```toml
[dependencies]
{project_name} = "1.0.0"
```

然后在脚本中引用：

```bash
source /path/to/{project_name}/lib/main.sh
library_function
```

## 构建

```bash
rmm build
```
"#, project_name = project_name);
    std::fs::write(project_path.join("README.md"), readme_content)?;

    ctx.debug("创建了库项目文件");

    Ok(())
}

fn create_ravd_files(project_path: &std::path::Path, project_name: &str, ctx: &Context) -> Result<()> {
    // RAVD是Android虚拟设备相关的文件
    
    // 创建avd目录
    let avd_dir = project_path.join("avd");
    ensure_dir_exists(&avd_dir)?;

    // 创建AVD配置
    let avd_config_content = format!(r#"# {project_name} AVD 配置

# AVD名称
AVD_NAME="{project_name}"

# Android版本
API_LEVEL=30

# 系统镜像
SYSTEM_IMAGE="system-images;android-30;google_apis;x86_64"

# 设备定义
DEVICE="pixel_4"
"#, project_name = project_name);
    std::fs::write(avd_dir.join("config.ini"), avd_config_content)?;

    // 创建启动脚本
    let start_script_content = format!(r#"#!/bin/bash
# {project_name} AVD 启动脚本

echo "启动 {project_name} Android虚拟设备..."

# 启动模拟器
emulator -avd {project_name} -no-snapshot-save -no-snapshot-load
"#, project_name = project_name);
    std::fs::write(avd_dir.join("start.sh"), start_script_content)?;

    // 创建README.md
    let readme_content = format!(r#"# {project_name}

这是一个RMM Android虚拟设备(RAVD)项目，用于测试RMM模块。

## 使用方法

1. 确保已安装Android SDK和模拟器
2. 运行启动脚本：

```bash
cd avd
./start.sh
```

## 配置

编辑 `avd/config.ini` 来修改AVD配置。

## 构建

```bash
rmm build
```
"#, project_name = project_name);
    std::fs::write(project_path.join("README.md"), readme_content)?;

    ctx.debug("创建了RAVD项目文件");

    Ok(())
}
