use anyhow::Result;
use chrono;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::io::{Write};

use crate::core::rmm_core::RmakeConfig;

/// Shellcheck 检查结果
#[derive(Debug, Serialize, Deserialize, Clone)]
struct ShellcheckIssue {
    file: String,
    line: u32,
    end_line: u32,
    column: u32,
    end_column: u32,
    level: String,
    code: u32,
    message: String,
    fix: Option<ShellcheckFix>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ShellcheckFix {
    replacements: Vec<ShellcheckReplacement>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ShellcheckReplacement {
    line: u32,
    end_line: u32,
    column: u32,
    end_column: u32,
    replacement: String,
}

/// Shellcheck 输出结果汇总
#[derive(Debug, Serialize, Deserialize)]
struct ShellcheckReport {
    checked_files: Vec<String>,
    total_issues: u32,
    error_count: u32,
    warning_count: u32,
    info_count: u32,
    style_count: u32,
    issues: Vec<ShellcheckIssue>,
}

/// 构建模块项目
pub fn build_project(project_path: &Path) -> Result<()> {
    build_project_with_options(project_path, true) // 默认启用自动修复
}

/// 构建模块项目（带选项）
pub fn build_project_with_options(project_path: &Path, auto_fix: bool) -> Result<()> {
    println!("{}", "🔨 开始构建模块项目".green().bold());
    
    // 检查项目是否有效
    if !is_valid_project(project_path) {
        anyhow::bail!("当前目录不是有效的 RMM 项目");
    }
    
    // 解析 Rmake.toml 配置
    let rmake_config = load_rmake_config(project_path)?;
    println!("{} 解析构建配置", "[+]".green().bold());
    
    // 创建构建目录
    setup_build_directories(project_path)?;
    
    // 执行构建流程
    execute_build_process(project_path, &rmake_config, auto_fix)?;
    
    // 执行源代码打包流程
    execute_source_packaging(project_path, &rmake_config)?;
    
    println!("\n{}", "🎉 模块构建完成！".green().bold());
    
    Ok(())
}

/// 检查是否是有效的项目
fn is_valid_project(project_path: &Path) -> bool {
    project_path.join("module.prop").exists() 
        && project_path.join(".rmmp").exists()
        && project_path.join(".rmmp/Rmake.toml").exists()
}

/// 加载 Rmake.toml 配置
fn load_rmake_config(project_path: &Path) -> Result<RmakeConfig> {
    let rmake_path = project_path.join(".rmmp/Rmake.toml");
    let content = fs::read_to_string(&rmake_path)?;
    let config: RmakeConfig = toml::from_str(&content)?;
    Ok(config)
}

/// 设置构建目录
fn setup_build_directories(project_path: &Path) -> Result<()> {
    let build_dir = project_path.join(".rmmp/build");
    let dist_dir = project_path.join(".rmmp/dist");
    
    // 清理并重新创建构建目录
    if build_dir.exists() {
        fs::remove_dir_all(&build_dir)?;
    }
    fs::create_dir_all(&build_dir)?;
    
    // 创建分发目录
    if !dist_dir.exists() {
        fs::create_dir_all(&dist_dir)?;
    }
    
    println!("{} 准备构建目录", "[+]".green().bold());
    Ok(())
}

/// 执行构建流程
fn execute_build_process(
    project_path: &Path,
    rmake_config: &RmakeConfig,
    auto_fix: bool,
) -> Result<()> {
    // 1. 复制文件到构建目录
    copy_files_to_build(project_path, rmake_config)?;
    
    // 2. 复制 update.json 到 dist 目录
    copy_update_json_to_dist(project_path)?;
    
    // 3. 执行 shell 脚本检查
    check_shell_scripts(project_path, auto_fix)?;
    
    // 4. 执行 prebuild 配置
    execute_prebuild(project_path, rmake_config)?;
    
    // 5. 打包模块
    package_module(project_path, rmake_config)?;
    
    // 6. 执行 postbuild
    execute_postbuild(project_path, rmake_config)?;
    
    Ok(())
}

/// 复制文件到构建目录
fn copy_files_to_build(
    project_path: &Path,
    rmake_config: &RmakeConfig,
) -> Result<()> {
    let build_dir = project_path.join(".rmmp/build");
    
    // 获取需要复制的文件和目录
    let entries = get_build_entries(project_path, rmake_config)?;
    
    for entry in entries {
        let relative_path = entry.strip_prefix(project_path)?;
        let dest_path = build_dir.join(relative_path);
          if entry.is_dir() {
            fs::create_dir_all(&dest_path)?;
            copy_directory(&entry, &dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            copy_file_with_line_ending_normalization(&entry, &dest_path)?;
        }
    }
    
    println!("{} 复制文件到构建目录", "[+]".green().bold());
    Ok(())
}

/// 获取需要构建的文件和目录
fn get_build_entries(
    project_path: &Path,
    rmake_config: &RmakeConfig,
) -> Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    
    // 首先获取项目中的所有文件和目录（基础文件）
    let mut base_entries = Vec::new();
    for entry in fs::read_dir(project_path)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy();
        
        // 排除 .rmmp 目录（构建系统目录）
        if file_name == ".rmmp" {
            continue;
        }
        
        base_entries.push(path);
    }
      // 应用 exclude 规则（排除文件）
    let exclude_patterns = &rmake_config.build.exclude;
    if !exclude_patterns.is_empty() {
        println!("    {} 应用排除规则:", "[!]".bright_yellow());
        for pattern in exclude_patterns {
            println!("      - {}", pattern);
        }
    }
    
    base_entries.retain(|path| {
        let file_name = path.file_name().unwrap().to_string_lossy();
        let path_str = path.to_string_lossy();
        
        for pattern in exclude_patterns {
            // 简单模式匹配
            if pattern.contains('*') {
                // 通配符匹配
                if pattern.ends_with("*") {
                    let prefix = &pattern[..pattern.len() - 1];
                    if file_name.starts_with(prefix) || path_str.contains(prefix) {
                        println!("      {} 排除文件: {} (匹配 {})", "[x]".red(), file_name, pattern);
                        return false;
                    }
                }
                if pattern.starts_with("*") {
                    let suffix = &pattern[1..];
                    if file_name.ends_with(suffix) || path_str.contains(suffix) {
                        println!("      {} 排除文件: {} (匹配 {})", "[x]".red(), file_name, pattern);
                        return false;
                    }
                }
            } else {
                // 精确匹配
                if file_name == pattern.as_str() || path_str.contains(pattern) {
                    println!("      {} 排除文件: {} (匹配 {})", "[x]".red(), file_name, pattern);
                    return false;
                }
            }
        }
        true
    });

    entries.extend(base_entries);    // 应用 include 规则（额外包含文件）
    // include 表示额外包含的文件，这些文件可能在其他位置或者需要特别包含
    let include_patterns: Vec<&String> = rmake_config.build.include
        .iter()
        .filter(|pattern| {
            let trimmed = pattern.trim();
            !trimmed.starts_with('#') && trimmed != "rmm"
        })
        .collect();
    
    if !include_patterns.is_empty() {
        println!("    {} 额外包含规则:", "[+]".green());
        for pattern in &include_patterns {
            println!("      + {}", pattern);
            // 这里可以添加实际的文件搜索逻辑
            // 现在只是提示用户这些是额外包含的文件
        }
    }
    
    Ok(entries)
}

/// 递归复制目录
fn copy_directory(src: &Path, dest: &Path) -> Result<()> {
    // 🔧 修复：添加源目录有效性检查
    if !src.exists() {
        return Err(anyhow::anyhow!("源目录不存在: {}", src.display()));
    }
    if !src.is_dir() {
        return Err(anyhow::anyhow!("源路径不是目录: {}", src.display()));
    }

    // 确保目标目录存在
    fs::create_dir_all(dest)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        
        // 🔧 修复：添加路径有效性检查
        if !src_path.exists() {
            println!("⚠️ 警告: 源路径不存在，跳过: {}", src_path.display());
            continue;
        }
        
        let dest_path = dest.join(entry.file_name());
        
        if src_path.is_dir() {
            if let Err(e) = copy_directory(&src_path, &dest_path) {
                println!("⚠️ 警告: 复制子目录失败 {}: {}", src_path.display(), e);
            }
        } else {
            if let Err(e) = copy_file_with_line_ending_normalization(&src_path, &dest_path) {
                println!("⚠️ 警告: 复制文件失败 {}: {}", src_path.display(), e);
            }
        }
    }
    Ok(())
}

/// 复制 update.json 到 dist 目录
fn copy_update_json_to_dist(project_path: &Path) -> Result<()> {
    let update_json_path = project_path.join("update.json");
    let dist_dir = project_path.join(".rmmp/dist");
    let dest_path = dist_dir.join("update.json");
      if update_json_path.exists() {
        copy_file_with_line_ending_normalization(&update_json_path, &dest_path)?;
        println!("{} 复制 update.json 到分发目录", "[+]".green().bold());
    }
    
    Ok(())
}

/// 检查 shell 脚本
fn check_shell_scripts(project_path: &Path, auto_fix: bool) -> Result<()> {
    let build_dir = project_path.join(".rmmp/build");
    let rmmp_dir = project_path.join(".rmmp");
    
    // 查找所有 .sh 文件
    let sh_files = find_shell_scripts(&build_dir)?;
    
    if sh_files.is_empty() {
        return Ok(());
    }
    
    println!("{} 检查 shell 脚本", "[+]".green().bold());
    
    // 检查是否安装了 shellcheck
    let shellcheck_available = Command::new("shellcheck")
        .arg("--version")
        .output()
        .is_ok();
    
    if !shellcheck_available {
        println!("{} shellcheck 未安装，跳过脚本检查", "[!]".yellow().bold());
        return Ok(());
    }    
    // 创建检查报告
    let mut report = ShellcheckReport {
        checked_files: Vec::new(),
        total_issues: 0,
        error_count: 0,
        warning_count: 0,
        info_count: 0,
        style_count: 0,
        issues: Vec::new(),
    };
    
    let mut has_errors = false;
    let mut all_fixes = String::new(); // 收集所有修复建
    
    // 对每个 shell 脚本运行 shellcheck
    for sh_file in &sh_files {
        println!("    检查: {}", sh_file.display());
        report.checked_files.push(sh_file.to_string_lossy().to_string());
        
        // 使用 JSON 格式输出获取详细信息
        let json_output = Command::new("shellcheck")
            .arg("--format=json")
            .arg(&sh_file)
            .output()?;
        
        // 获取带 wiki 链接的详细输出
        let wiki_output = Command::new("shellcheck")
            .arg("-W")
            .arg("10") // 显示最多10个wiki链接
            .arg(&sh_file)
            .output()?;
        
        // 获取 diff 格式的修复建议
        let diff_output = Command::new("shellcheck")
            .arg("--format=diff")
            .arg(&sh_file)
            .output()?;
        
        // 解析 JSON 输出
        if !json_output.stdout.is_empty() {
            let json_str = String::from_utf8_lossy(&json_output.stdout);
            if let Ok(issues) = serde_json::from_str::<Vec<ShellcheckIssue>>(&json_str) {
                for issue in issues {
                    // 统计各类问题数量
                    match issue.level.as_str() {
                        "error" => {
                            report.error_count += 1;
                            has_errors = true;
                        }
                        "warning" => report.warning_count += 1,
                        "info" => report.info_count += 1,
                        "style" => report.style_count += 1,
                        _ => {}
                    }
                    report.issues.push(issue);
                }
            }
        }
        
        // 处理修复建议
        if !diff_output.stdout.is_empty() {
            let diff_content = String::from_utf8_lossy(&diff_output.stdout);
            if !diff_content.trim().is_empty() {
                all_fixes.push_str(&format!("\n=== {} ===\n", sh_file.display()));
                all_fixes.push_str(&diff_content);
                all_fixes.push_str("\n");
            }
        }
        
        // 如果有问题，显示详细信息
        if !wiki_output.status.success() || !wiki_output.stdout.is_empty() {
            let output_str = String::from_utf8_lossy(&wiki_output.stdout);
            if !output_str.trim().is_empty() {
                println!("{} shellcheck 发现问题: {}", "[!]".yellow().bold(), sh_file.display());
                println!("{}", output_str);
            }
        } else {
            println!("{} shellcheck 检查通过: {}", "✅".green(), sh_file.display());
        }
    }    
    report.total_issues = report.error_count + report.warning_count + report.info_count + report.style_count;
    
    // 写入 JSON 格式报告（机器友好）
    let json_report_path = rmmp_dir.join("shellcheck.json");
    let json_content = serde_json::to_string_pretty(&report)?;
    fs::write(&json_report_path, json_content)?;
    println!("{} 检查报告已保存到: {}", "[+]".green().bold(), json_report_path.display());
    
    // 写入 AI 友好格式报告
    let ai_report_path = rmmp_dir.join("shellcheck.llms.txt");
    let ai_content = generate_ai_friendly_report(&report);
    fs::write(&ai_report_path, ai_content)?;
    println!("{} AI 友好报告已保存到: {}", "[+]".green().bold(), ai_report_path.display());
      // 保存修复建议
    if !all_fixes.is_empty() {
        let fixes_path = rmmp_dir.join("shellcheck-fixes.diff");
        fs::write(&fixes_path, &all_fixes)?;
        println!("{} 修复建议已保存到: {}", "[+]".green().bold(), fixes_path.display());
          // 自动修复功能
        if auto_fix {
            println!("{} 尝试自动应用修复...", "[exec]".blue().bold());
            
            match apply_fixes_directly(&sh_files) {
                Ok(fixed_count) => {
                    if fixed_count > 0 {
                        println!("{} 自动修复已应用！修复了 {} 个文件", "✅".green().bold(), fixed_count);
                        
                        // 重新检查以确认修复
                        println!("{} 重新检查修复后的脚本...", "[exec]".blue().bold());
                        let recheck_result = recheck_fixed_scripts(&sh_files)?;
                        if recheck_result.total_issues == 0 {
                            println!("{} 所有问题已修复！", "🎉".green().bold());
                        } else {                        println!("{} 部分问题已修复，剩余 {} 个问题需要手动处理", 
                               "[!]".yellow().bold(), recheck_result.total_issues);
                        }                    } else {
                        println!("{} 没有发现可自动修复的问题", "[~]".truecolor(255, 165, 0).bold()); // 橙色
                    }
                }
                Err(e) => {
                    println!("{} 自动修复失败: {}", "[x]".red().bold(), e);
                    
                    // 尝试使用 git apply 作为备选方案（使用规范化路径）
                    println!("{} 尝试使用备选修复方法...", "[exec]".blue().bold());
                    if try_git_apply(project_path, &fixes_path).is_ok() {
                        println!("{} 备选修复方法成功！", "✅".green().bold());
                    } else {
                        println!("{} 手动应用修复: cd {} && git apply .rmmp/shellcheck-fixes.diff", 
                               "💡".blue().bold(), project_path.display());
                    }
                }
            }
        } else {
            println!("{} 手动应用修复: cd {} && git apply .rmmp/shellcheck-fixes.diff", 
                   "💡".blue().bold(), project_path.display());
        }
    }
    
    // 如果有错误，终止构建
    if has_errors {
        anyhow::bail!("Shell 脚本检查发现错误，终止构建。详情请查看: {}", json_report_path.display());
    }
    
    if report.total_issues > 0 {        println!("{} 发现 {} 个问题（错误: {}, 警告: {}, 信息: {}, 样式: {}）", 
                 "[!]".yellow().bold(), 
                 report.total_issues, 
                 report.error_count, 
                 report.warning_count, 
                 report.info_count, 
                 report.style_count);
    }
    
    Ok(())
}

/// 查找所有 shell 脚本文件
fn find_shell_scripts(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut sh_files = Vec::new();
    
    if !dir.exists() {
        return Ok(sh_files);
    }
    
    find_shell_scripts_recursive(dir, &mut sh_files)?;
    Ok(sh_files)
}

/// 递归查找 shell 脚本
fn find_shell_scripts_recursive(dir: &Path, sh_files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            find_shell_scripts_recursive(&path, sh_files)?;
        } else if let Some(extension) = path.extension() {
            if extension == "sh" {
                sh_files.push(path);
            }
        }
    }
    Ok(())
}

/// 执行 prebuild 脚本
fn execute_prebuild(
    project_path: &Path,
    rmake_config: &RmakeConfig,
) -> Result<()> {
    // 执行 Rmake.toml 中定义的 prebuild 命令
    if !rmake_config.build.prebuild.is_empty() {
        println!("{} 执行 prebuild 命令", "[exec]".blue().bold());
          for command in &rmake_config.build.prebuild {
            println!("    运行: {}", command.cyan());
              // 修复 Windows 路径问题：确保使用正确的路径格式
            let working_dir = normalize_path_for_command(project_path)?;
            
            let output = if cfg!(target_os = "windows") {
                let powershell_command = convert_bash_to_powershell(command);
                Command::new("powershell")
                    .args(&["-Command", &powershell_command])
                    .current_dir(working_dir)
                    .output()?
            } else {
                Command::new("sh")
                    .args(&["-c", command])
                    .current_dir(working_dir)
                    .output()?
            };
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("prebuild 命令执行失败: {}\n错误: {}", command, stderr);
            }
            
            // 打印输出
            if !output.stdout.is_empty() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("    输出: {}", stdout.trim());
            }
        }
    }
      // 检查是否有传统的 prebuild 脚本
    let prebuild_script = project_path.join("scripts/prebuild.sh");
    if prebuild_script.exists() {
        println!("{} 执行传统 prebuild 脚本", "[+]".green().bold());
        
        let working_dir = normalize_path_for_command(project_path)?;
        let output = Command::new("sh")
            .arg(&prebuild_script)
            .current_dir(working_dir)
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("prebuild 脚本执行失败: {}", stderr);
        }
        
        // 打印输出
        if !output.stdout.is_empty() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }
    
    Ok(())
}

/// 打包模块
fn package_module(
    project_path: &Path,
    _rmake_config: &RmakeConfig,
) -> Result<()> {
    let build_dir = project_path.join(".rmmp/build");
    let dist_dir = project_path.join(".rmmp/dist");
    
    // 读取项目信息
    let project_info = read_project_info(project_path)?;
    let module_name = format!("{}-{}.zip", project_info.id, project_info.version_code);
    let output_path = dist_dir.join(&module_name);
    
    println!("{} 打包模块: {}", "[zip]".magenta().bold(), module_name.cyan());
    
    // 创建 ZIP 文件
    create_zip_archive(&build_dir, &output_path)?;
    
    println!("{} 模块打包完成: {}", "✅".green().bold(), output_path.display());
    
    Ok(())
}

/// 读取项目信息
fn read_project_info(project_path: &Path) -> Result<ProjectInfo> {
    let module_prop_path = project_path.join("module.prop");
    let content = fs::read_to_string(&module_prop_path)?;
    
    let mut id = String::new();
    let mut version_code = String::new();
    
    for line in content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "id" => id = value.trim().to_string(),
                "versionCode" => version_code = value.trim().to_string(),
                _ => {}
            }
        }
    }
    
    Ok(ProjectInfo { id, version_code })
}

/// 项目信息结构
struct ProjectInfo {
    id: String,
    version_code: String,
}

/// 创建 ZIP 压缩包
fn create_zip_archive(source_dir: &Path, output_path: &Path) -> Result<()> {
    let file = fs::File::create(output_path)?;
    let mut zip = zip::ZipWriter::new(file);
    
    add_directory_to_zip(&mut zip, source_dir, source_dir)?;
    
    zip.finish()?;
    Ok(())
}

/// 添加目录到 ZIP
fn add_directory_to_zip<W: Write + std::io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    dir: &Path,
    base_dir: &Path,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path.strip_prefix(base_dir)?;
          if path.is_dir() {
            // 添加目录 - 确保使用正斜杠分隔符
            let dir_name = format!("{}/", relative_path.display().to_string().replace('\\', "/"));
            zip.add_directory(dir_name, zip::write::SimpleFileOptions::default())?;
            
            // 递归添加子目录
            add_directory_to_zip(zip, &path, base_dir)?;
        } else {
            // 添加文件 - 确保使用正斜杠分隔符
            let file_name = relative_path.display().to_string().replace('\\', "/");
            zip.start_file(file_name, zip::write::SimpleFileOptions::default())?;
            
            let file_content = fs::read(&path)?;
            zip.write_all(&file_content)?;
        }
    }
    
    Ok(())
}

/// 创建 tar.gz 压缩包
fn create_tar_gz_archive(source_dir: &Path, output_path: &Path) -> Result<()> {
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use tar::Builder;
    
    let tar_gz_file = fs::File::create(output_path)?;
    let enc = GzEncoder::new(tar_gz_file, Compression::default());
    let mut tar = Builder::new(enc);
    
    // 递归添加目录中的所有文件
    add_directory_to_tar(&mut tar, source_dir, source_dir)?;
    
    tar.finish()?;
    Ok(())
}

/// 添加目录到 tar
fn add_directory_to_tar<W: Write>(
    tar: &mut tar::Builder<W>,
    dir: &Path,
    base_dir: &Path,
) -> Result<()> {
    // 🔧 修复：添加路径有效性检查
    if !dir.exists() {
        println!("⚠️ 警告: 目录不存在，跳过: {}", dir.display());
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        // 🔧 修复：添加路径存在性检查
        if !path.exists() {
            println!("⚠️ 警告: 路径不存在，跳过: {}", path.display());
            continue;
        }
        
        let relative_path = match path.strip_prefix(base_dir) {
            Ok(rel_path) => rel_path,
            Err(e) => {
                println!("⚠️ 警告: 无法计算相对路径 {}: {}", path.display(), e);
                continue;
            }
        };
        
        // 确保路径使用正确的分隔符，并且不为空
        let normalized_path = if relative_path.as_os_str().is_empty() {
            continue; // 跳过根目录自身
        } else {
            relative_path.to_string_lossy().replace('\\', "/")
        };
        
        if path.is_dir() {
            // 添加目录条目（以 / 结尾）
            let mut header = tar::Header::new_gnu();
            header.set_mode(0o755);
            header.set_entry_type(tar::EntryType::Directory);
            header.set_size(0);
            header.set_cksum();
            
            let dir_path = if normalized_path.is_empty() {
                continue; // 跳过空路径
            } else {
                format!("{}/", normalized_path)
            };
            
            // 🔧 修复：添加错误处理
            if let Err(e) = tar.append_data(&mut header, &dir_path, std::io::empty()) {
                println!("⚠️ 警告: 添加目录到tar失败 {}: {}", dir_path, e);
                continue;
            }
            
            // 递归添加子目录
            add_directory_to_tar(tar, &path, base_dir)?;
        } else {
            // 添加文件
            if normalized_path.is_empty() {
                continue; // 跳过空路径
            }
            
            // 🔧 修复：更安全的文件打开方式
            let mut file = match fs::File::open(&path) {
                Ok(f) => f,
                Err(e) => {
                    println!("⚠️ 警告: 无法打开文件 {}: {}", path.display(), e);
                    continue;
                }
            };
            
            let metadata = match file.metadata() {
                Ok(m) => m,
                Err(e) => {
                    println!("⚠️ 警告: 无法获取文件元数据 {}: {}", path.display(), e);
                    continue;
                }
            };
            
            let mut header = tar::Header::new_gnu();
            header.set_mode(0o644);
            header.set_size(metadata.len());
            header.set_cksum();
            
            // 🔧 修复：添加错误处理
            if let Err(e) = tar.append_data(&mut header, &normalized_path, &mut file) {
                println!("⚠️ 警告: 添加文件到tar失败 {}: {}", normalized_path, e);
                continue;
            }
        }
    }
    
    Ok(())
}

/// 执行 postbuild 脚本
fn execute_postbuild(
    project_path: &Path,
    rmake_config: &RmakeConfig,
) -> Result<()> {
    // 执行 Rmake.toml 中定义的 postbuild 命令
    if !rmake_config.build.postbuild.is_empty() {
        println!("{} 执行 postbuild 命令", "[exec]".blue().bold());
          for command in &rmake_config.build.postbuild {
            println!("    运行: {}", command.cyan());
              let working_dir = normalize_path_for_command(project_path)?;
            let output = if cfg!(target_os = "windows") {
                let powershell_command = convert_bash_to_powershell(command);
                Command::new("powershell")
                    .args(&["-Command", &powershell_command])
                    .current_dir(working_dir)
                    .output()?
            } else {
                Command::new("sh")
                    .args(&["-c", command])
                    .current_dir(working_dir)
                    .output()?
            };
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);                println!("{} postbuild 命令执行失败: {}\n错误: {}", 
                       "[x]".red().bold(), command, stderr);
            } else {
                // 打印输出
                if !output.stdout.is_empty() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    println!("    输出: {}", stdout.trim());
                }
            }
        }
    }
    
    // 检查是否有传统的 postbuild 脚本
    let postbuild_script = project_path.join("scripts/postbuild.sh");
    if postbuild_script.exists() {
        println!("{} 执行传统 postbuild 脚本", "[+]".green().bold());
        
        let working_dir = normalize_path_for_command(project_path)?;
        let output = Command::new("sh")
            .arg(&postbuild_script)
            .current_dir(working_dir)
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("{} postbuild 脚本执行失败: {}", "[x]".red().bold(), stderr);
        } else {
            // 打印输出
            if !output.stdout.is_empty() {
                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
        }
    }
    
    Ok(())
}

/// 执行源代码打包流程
fn execute_source_packaging(
    project_path: &Path,
    rmake_config: &RmakeConfig,
) -> Result<()> {
    println!("{} 开始源代码打包", "[tar]".cyan().bold());
    
    // 创建源代码构建目录
    let source_build_dir = project_path.join(".rmmp/source-build");
    if source_build_dir.exists() {
        fs::remove_dir_all(&source_build_dir)?;
    }
    fs::create_dir_all(&source_build_dir)?;
    
    // 复制源代码文件（依据 src 配置）
    copy_source_files(project_path, &source_build_dir, rmake_config)?;
    
    // 执行源代码 prebuild
    execute_source_prebuild(project_path)?;
    
    // 打包源代码
    package_source_code(project_path, &source_build_dir)?;
    
    // 执行源代码 postbuild
    execute_source_postbuild(project_path)?;
    
    println!("{} 源代码打包完成", "✅".green().bold());
    
    Ok(())
}

/// 复制源代码文件
fn copy_source_files(project_path: &Path, source_build_dir: &Path, rmake_config: &RmakeConfig) -> Result<()> {
    // 根据 Rmake.toml 中的 build.src 配置复制源代码文件
    if let Some(src_config) = &rmake_config.build.src {
        // 首先获取所有文件
        let mut source_entries = Vec::new();
        for entry in fs::read_dir(project_path)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_string_lossy();
            
            // 不排除 .rmmp 目录，因为源代码需要包含配置
            source_entries.push(path);
        }
          // 应用 src exclude 规则
        if !src_config.exclude.is_empty() {
            println!("    {} 源代码排除规则:", "[!]".bright_yellow());
            for pattern in &src_config.exclude {
                println!("      - {}", pattern);
            }
        }
        
        source_entries.retain(|path| {
            let file_name = path.file_name().unwrap().to_string_lossy();
            let path_str = path.to_string_lossy();
            
            for pattern in &src_config.exclude {
                if pattern.contains('*') {
                    if pattern.ends_with("*") {
                        let prefix = &pattern[..pattern.len() - 1];
                        if file_name.starts_with(prefix) || path_str.contains(prefix) {
                            println!("      {} 排除源文件: {} (匹配 {})", "[x]".red(), file_name, pattern);
                            return false;
                        }
                    }
                    if pattern.starts_with("*") {
                        let suffix = &pattern[1..];
                        if file_name.ends_with(suffix) || path_str.contains(suffix) {
                            println!("      {} 排除源文件: {} (匹配 {})", "[x]".red(), file_name, pattern);
                            return false;
                        }
                    }
                } else {
                    if file_name == pattern.as_str() || path_str.contains(pattern) {
                        println!("      {} 排除源文件: {} (匹配 {})", "[x]".red(), file_name, pattern);
                        return false;
                    }
                }
            }
            true
        });
          // 复制文件
        for path in source_entries {
            // 🔧 修复：添加路径有效性检查
            if !path.exists() {
                println!("⚠️ 警告: 源文件不存在，跳过: {}", path.display());
                continue;
            }
            
            let file_name = match path.file_name() {
                Some(name) => name,
                None => {
                    println!("⚠️ 警告: 无法获取文件名，跳过: {}", path.display());
                    continue;
                }
            };
            let dest_path = source_build_dir.join(file_name);
            
            if path.is_dir() {
                if file_name == ".rmmp" {
                    // 特殊处理 .rmmp 目录，只复制 Rmake.toml
                    if let Err(e) = fs::create_dir_all(&dest_path) {
                        println!("⚠️ 警告: 创建目录失败 {}: {}", dest_path.display(), e);
                        continue;
                    }
                    let rmake_source = path.join("Rmake.toml");
                    let rmake_dest = dest_path.join("Rmake.toml");
                    if rmake_source.exists() {
                        if let Err(e) = fs::copy(&rmake_source, &rmake_dest) {
                            println!("⚠️ 警告: 复制配置文件失败: {}", e);
                        } else {
                            println!("    ✅ 包含配置文件: .rmmp/Rmake.toml");
                        }
                    }                } else {
                    if let Err(e) = copy_directory(&path, &dest_path) {
                        println!("⚠️ 警告: 复制目录失败 {}: {}", path.display(), e);
                    }
                }
            } else {
                if let Err(e) = copy_file_with_line_ending_normalization(&path, &dest_path) {
                    println!("⚠️ 警告: 复制文件失败 {}: {}", path.display(), e);
                }
            }
        }// 处理 src include（额外包含文件）
        let src_include_patterns: Vec<&String> = src_config.include
            .iter()
            .filter(|pattern| {
                let trimmed = pattern.trim();
                !trimmed.starts_with('#') && trimmed != "rmm"
            })
            .collect();
            
        if !src_include_patterns.is_empty() {
            println!("    {} 源代码额外包含:", "[+]".green());
            for include_pattern in &src_include_patterns {
                println!("      + {}", include_pattern);
            }
        }
    } else {
        // 如果没有 src 配置，复制所有文件（包括 .rmmp/Rmake.toml）
        for entry in fs::read_dir(project_path)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_string_lossy();
            
            let dest_path = source_build_dir.join(file_name.as_ref());
            
            if path.is_dir() {
                if file_name == ".rmmp" {
                    // 特殊处理 .rmmp 目录，只复制 Rmake.toml
                    fs::create_dir_all(&dest_path)?;
                    let rmake_source = path.join("Rmake.toml");
                    let rmake_dest = dest_path.join("Rmake.toml");                    if rmake_source.exists() {
                        copy_file_with_line_ending_normalization(&rmake_source, &rmake_dest)?;
                        println!("    ✅ 包含配置文件: .rmmp/Rmake.toml");
                    }
                } else {
                    copy_directory(&path, &dest_path)?;
                }            } else {
                copy_file_with_line_ending_normalization(&path, &dest_path)?;
            }
        }
    }
    
    println!("{} 复制源代码文件", "[+]".green().bold());
    Ok(())
}

/// 执行源代码 prebuild
fn execute_source_prebuild(project_path: &Path) -> Result<()> {
    let prebuild_script = project_path.join("scripts/source-prebuild.sh");
    
    if prebuild_script.exists() {
        println!("{} 执行源代码 prebuild 脚本", "[+]".green().bold());
        
        let working_dir = normalize_path_for_command(project_path)?;
        let output = Command::new("sh")
            .arg(&prebuild_script)
            .current_dir(working_dir)
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("{} 源代码 prebuild 脚本执行失败: {}", "[x]".red().bold(), stderr);
        }
    }
    
    Ok(())
}

/// 打包源代码
fn package_source_code(project_path: &Path, source_build_dir: &Path) -> Result<()> {
    // 🔧 修复：验证源目录
    if !source_build_dir.exists() {
        return Err(anyhow::anyhow!("源代码构建目录不存在: {}", source_build_dir.display()));
    }
    
    // 检查目录是否为空
    let is_empty = fs::read_dir(source_build_dir)?.next().is_none();
    if is_empty {
        println!("⚠️ 警告: 源代码构建目录为空: {}", source_build_dir.display());
        // 仍然继续创建空的 tar.gz 文件
    }
    
    let dist_dir = project_path.join(".rmmp/dist");
    
    // 🔧 修复：确保 dist 目录存在
    if !dist_dir.exists() {
        fs::create_dir_all(&dist_dir)?;
    }
    
    let project_info = read_project_info(project_path)?;
    let source_name = format!("{}-{}-source.tar.gz", project_info.id, project_info.version_code);
    let output_path = dist_dir.join(&source_name);
    
    println!("{} 打包源代码: {}", "[tar]".cyan().bold(), source_name.cyan());
    
    // 🔧 修复：添加详细的错误处理
    match create_tar_gz_archive(source_build_dir, &output_path) {
        Ok(()) => {
            println!("{} 源代码打包完成: {}", "✅".green().bold(), output_path.display());
            Ok(())
        }
        Err(e) => {
            Err(anyhow::anyhow!("打包源代码失败: {} -> {}: {}", 
                source_build_dir.display(), output_path.display(), e))
        }
    }
}

/// 执行源代码 postbuild
fn execute_source_postbuild(project_path: &Path) -> Result<()> {
    let postbuild_script = project_path.join("scripts/source-postbuild.sh");
    
    if postbuild_script.exists() {
        println!("{} 执行源代码 postbuild 脚本", "[+]".green().bold());
        
        let working_dir = normalize_path_for_command(project_path)?;
        let output = Command::new("sh")
            .arg(&postbuild_script)
            .current_dir(working_dir)
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("{} 源代码 postbuild 脚本执行失败: {}", "[x]".red().bold(), stderr);
        }
    }
    
    Ok(())
}

/// 生成 AI 友好的 shellcheck 报告
fn generate_ai_friendly_report(report: &ShellcheckReport) -> String {
    let mut content = String::new();
    
    content.push_str("# Shellcheck Analysis Report\n\n");
    content.push_str(&format!("**Generated**: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    
    // 摘要
    content.push_str("## Summary\n\n");
    content.push_str(&format!("- **Files Checked**: {}\n", report.checked_files.len()));
    content.push_str(&format!("- **Total Issues**: {}\n", report.total_issues));
    content.push_str(&format!("- **Errors**: {} (build-blocking)\n", report.error_count));
    content.push_str(&format!("- **Warnings**: {}\n", report.warning_count));
    content.push_str(&format!("- **Info**: {}\n", report.info_count));
    content.push_str(&format!("- **Style**: {}\n\n", report.style_count));
    
    // 检查的文件列表
    content.push_str("## Checked Files\n\n");
    for file in &report.checked_files {
        content.push_str(&format!("- `{}`\n", file));
    }
    content.push_str("\n");
    
    if report.issues.is_empty() {
        content.push_str("## Result\n\n");
        content.push_str("🎉 **All shell scripts passed shellcheck analysis!**\n\n");
        content.push_str("No issues found in any of the checked shell scripts.\n");
    } else {
        // 按严重程度分组显示问题
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut infos = Vec::new();
        let mut styles = Vec::new();
        
        for issue in &report.issues {
            match issue.level.as_str() {
                "error" => errors.push(issue),
                "warning" => warnings.push(issue),
                "info" => infos.push(issue),
                "style" => styles.push(issue),
                _ => {}
            }
        }
        
        // 错误（构建阻断）
        if !errors.is_empty() {
            content.push_str("## 🚨 Errors (Build Blocking)\n\n");
            for issue in errors {
                content.push_str(&format_issue_for_ai(issue));
            }
        }
        
        // 警告
        if !warnings.is_empty() {
            content.push_str("## ⚠️ Warnings\n\n");
            for issue in warnings {
                content.push_str(&format_issue_for_ai(issue));
            }
        }
        
        // 信息
        if !infos.is_empty() {
            content.push_str("## ℹ️ Info\n\n");
            for issue in infos {
                content.push_str(&format_issue_for_ai(issue));
            }
        }
        
        // 样式
        if !styles.is_empty() {
            content.push_str("## 🎨 Style\n\n");
            for issue in styles {
                content.push_str(&format_issue_for_ai(issue));
            }
        }
          // 建议
        content.push_str("## 💡 Recommendations\n\n");
        if report.error_count > 0 {
            content.push_str("- **Fix all errors**: Errors must be resolved before the build can proceed.\n");
        }
        if report.warning_count > 0 {
            content.push_str("- **Review warnings**: While not build-blocking, warnings indicate potential issues.\n");
        }
        if report.style_count > 0 {
            content.push_str("- **Consider style improvements**: These suggestions can improve code quality and maintainability.\n");
        }
        content.push_str("- **Use shellcheck locally**: Run `shellcheck <script.sh>` to catch issues early.\n");
        content.push_str("- **Enable shellcheck in your editor**: Many editors have shellcheck integration.\n");
        content.push_str("- **Apply automatic fixes**: Use `git apply .rmmp/shellcheck-fixes.diff` to apply suggested fixes.\n");
        content.push_str("- **View detailed fixes**: Check `.rmmp/shellcheck-fixes.diff` for patch-ready fixes.\n\n");
        
        // 快速修复指南
        content.push_str("## 🔧 Quick Fix Guide\n\n");
        content.push_str("### Automatic Application\n");
        content.push_str("```bash\n");
        content.push_str("# Navigate to project root\n");
        content.push_str("cd /path/to/your/project\n\n");
        content.push_str("# Apply all suggested fixes\n");
        content.push_str("git apply .rmmp/shellcheck-fixes.diff\n\n");
        content.push_str("# Review changes\n");
        content.push_str("git diff\n\n");
        content.push_str("# Commit if satisfied\n");
        content.push_str("git add .\n");
        content.push_str("git commit -m \"Apply shellcheck fixes\"\n");
        content.push_str("```\n\n");
        
        content.push_str("### Manual Review\n");
        content.push_str("```bash\n");
        content.push_str("# View the suggested changes\n");
        content.push_str("cat .rmmp/shellcheck-fixes.diff\n\n");
        content.push_str("# Apply selectively using your editor or patch tool\n");
        content.push_str("# Each fix can be applied individually\n");
        content.push_str("```\n\n");
    }
    
    content.push_str("---\n");
    content.push_str("*This report was generated by RMM (Root Manage Module) build system.*\n");
    
    content
}

/// 格式化单个问题为 AI 友好格式
fn format_issue_for_ai(issue: &ShellcheckIssue) -> String {
    let mut content = String::new();
    
    content.push_str(&format!("### SC{} in `{}`\n\n", issue.code, issue.file));
    content.push_str(&format!("**Location**: Line {}, Column {}", issue.line, issue.column));
    if issue.line != issue.end_line || issue.column != issue.end_column {
        content.push_str(&format!(" to Line {}, Column {}", issue.end_line, issue.end_column));
    }
    content.push_str("\n\n");
    
    content.push_str(&format!("**Message**: {}\n\n", issue.message));
    
    // 如果有修复建议，显示它
    if let Some(fix) = &issue.fix {
        content.push_str("**Suggested Fix**:\n");
        for replacement in &fix.replacements {
            content.push_str(&format!("- Replace text at line {}, column {} with: `{}`\n", 
                                     replacement.line, replacement.column, replacement.replacement));
        }
        content.push_str("\n");
    }
    
    // 添加 shellcheck 规则链接
    content.push_str(&format!("**Reference**: [ShellCheck SC{}](https://www.shellcheck.net/wiki/SC{})\n\n", 
                             issue.code, issue.code));
    
    content.push_str("---\n\n");
    content
}

/// 重新检查修复后的脚本
fn recheck_fixed_scripts(sh_files: &[PathBuf]) -> Result<ShellcheckReport> {
    let mut report = ShellcheckReport {
        checked_files: Vec::new(),
        total_issues: 0,
        error_count: 0,
        warning_count: 0,
        info_count: 0,
        style_count: 0,
        issues: Vec::new(),
    };
    
    for sh_file in sh_files {
        report.checked_files.push(sh_file.to_string_lossy().to_string());
        
        // 使用 JSON 格式输出获取详细信息
        let json_output = Command::new("shellcheck")
            .arg("--format=json")
            .arg(&sh_file)
            .output()?;
        
        // 解析 JSON 输出
        if !json_output.stdout.is_empty() {
            let json_str = String::from_utf8_lossy(&json_output.stdout);
            if let Ok(issues) = serde_json::from_str::<Vec<ShellcheckIssue>>(&json_str) {
                for issue in issues {
                    // 统计各类问题数量
                    match issue.level.as_str() {
                        "error" => report.error_count += 1,
                        "warning" => report.warning_count += 1,
                        "info" => report.info_count += 1,
                        "style" => report.style_count += 1,
                        _ => {}
                    }
                    report.issues.push(issue);
                }
            }
        }
    }
    
    report.total_issues = report.error_count + report.warning_count + report.info_count + report.style_count;
    Ok(report)
}

/// 直接应用 shellcheck 修复
fn apply_fixes_directly(sh_files: &[PathBuf]) -> Result<usize> {
    let mut fixed_count = 0;
    
    for sh_file in sh_files {
        println!("    修复: {}", sh_file.display());
        
        // 获取该文件的修复建议
        let fix_output = Command::new("shellcheck")
            .arg("--format=diff")
            .arg(&sh_file)
            .output()?;
        
        if fix_output.stdout.is_empty() {
            continue; // 没有修复建议
        }
        
        let diff_content = String::from_utf8_lossy(&fix_output.stdout);
        
        // 应用修复到构建目录的文件
        if apply_simple_fixes(&sh_file, &diff_content)? {
            // 尝试找到对应的源文件并也修复它
            if let Some(source_file) = find_source_file(&sh_file) {
                if source_file.exists() {
                    println!("      📝 同时修复源文件: {}", source_file.display());
                    let source_fix_output = Command::new("shellcheck")
                        .arg("--format=diff")
                        .arg(&source_file)
                        .output()?;
                    
                    if !source_fix_output.stdout.is_empty() {
                        let source_diff = String::from_utf8_lossy(&source_fix_output.stdout);
                        apply_simple_fixes(&source_file, &source_diff)?;
                    }
                }
            }
            
            fixed_count += 1;
            println!("      ✅ 修复成功");
        } else {
            println!("      ⚠️ 修复跳过（复杂修改）");
        }
    }
    
    Ok(fixed_count)
}

/// 找到构建文件对应的源文件
fn find_source_file(build_file: &Path) -> Option<PathBuf> {
    // 构建文件路径格式: project/.rmmp/build/file.sh
    // 对应源文件路径: project/file.sh
    
    let build_path_str = build_file.to_string_lossy();
    
    // 查找 .rmmp/build/ 部分并替换
    if let Some(rmmp_build_pos) = build_path_str.find(".rmmp/build/") {
        let project_root = &build_path_str[..rmmp_build_pos];
        let file_name = &build_path_str[rmmp_build_pos + ".rmmp/build/".len()..];
        
        let source_path = PathBuf::from(format!("{}{}", project_root, file_name));
        return Some(source_path);
    }
    
    // Windows 路径格式
    if let Some(rmmp_build_pos) = build_path_str.find(".rmmp\\build\\") {
        let project_root = &build_path_str[..rmmp_build_pos];
        let file_name = &build_path_str[rmmp_build_pos + ".rmmp\\build\\".len()..];
        
        let source_path = PathBuf::from(format!("{}{}", project_root, file_name));
        return Some(source_path);
    }
    
    None
}

/// 应用简单的修复（主要针对引号、空格等简单问题）
fn apply_simple_fixes(file_path: &Path, diff_content: &str) -> Result<bool> {
    let content = fs::read_to_string(file_path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut modified = false;
    
    // 解析 diff 格式
    let mut in_hunk = false;
    let mut hunk_old_start = 0usize;
    let mut current_line = 0usize;
    
    for line in diff_content.lines() {
        if line.starts_with("@@") {
            // 解析 hunk header: @@ -old_start,old_count +new_start,new_count @@
            if let Some(captures) = regex::Regex::new(r"@@ -(\d+),?\d* \+(\d+),?\d* @@")
                .unwrap()
                .captures(line) 
            {
                hunk_old_start = captures.get(1).unwrap().as_str().parse::<usize>().unwrap_or(1);
                current_line = hunk_old_start;
                in_hunk = true;
            }
        } else if in_hunk {
            if line.starts_with("-") && !line.starts_with("---") {
                // 这是要删除的行，跳过（在下一个+行中处理）
                continue;
            } else if line.starts_with("+") && !line.starts_with("+++") {
                // 这是要添加的行
                let new_content = &line[1..]; // 移除 '+' 前缀
                if current_line > 0 && current_line <= lines.len() {
                    lines[current_line - 1] = new_content.to_string();
                    modified = true;
                }
                current_line += 1;
            } else if line.starts_with(" ") {
                // 上下文行，移动到下一行
                current_line += 1;
            } else if line.is_empty() || line.starts_with("\\") {
                // 忽略空行和其他元数据
                continue;
            } else {
                // 结束当前 hunk
                in_hunk = false;
            }
        }
    }
    
    if modified {
        let new_content = lines.join("\n") + "\n";
        fs::write(file_path, new_content)?;
    }
    
    Ok(modified)
}

/// 尝试使用 git apply（使用规范化路径）
fn try_git_apply(project_path: &Path, _fixes_path: &Path) -> Result<()> {
    // 将路径转换为相对路径，避免长路径问题
    let relative_fixes_path = Path::new(".rmmp").join("shellcheck-fixes.diff");
    
    let working_dir = normalize_path_for_command(project_path)?;
    let output = Command::new("git")
        .arg("apply")
        .arg("--verbose")
        .arg(relative_fixes_path)
        .current_dir(working_dir)
        .output()?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            println!("Git apply 输出:\n{}", stdout);
        }
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git apply 失败: {}", stderr);
    }
}

/// 标准化路径以避免 Windows UNC 路径问题
fn normalize_path_for_command(path: &Path) -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        // 如果路径以 \\?\ 开头（UNC 长路径），尝试转换为普通路径
        let path_str = path.to_string_lossy();
        if path_str.starts_with("\\\\?\\") {
            // 移除 \\?\ 前缀
            let normal_path = &path_str[4..];
            return Ok(PathBuf::from(normal_path));
        }
        
        // 尝试获取绝对路径但避免长路径格式
        if let Ok(canonical) = path.canonicalize() {
            let canonical_str = canonical.to_string_lossy();
            if canonical_str.starts_with("\\\\?\\") {
                // 如果 canonicalize 返回了 UNC 路径，移除前缀
                let normal_path = &canonical_str[4..];
                Ok(PathBuf::from(normal_path))
            } else {
                Ok(canonical)
            }
        } else {
            // 如果无法规范化，使用原路径
            Ok(path.to_path_buf())
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // 非 Windows 系统直接返回路径
        Ok(path.to_path_buf())
    }
}

/// 规范化文件的行尾序列为 LF
fn normalize_line_endings(content: &str) -> String {
    content.replace("\r\n", "\n").replace("\r", "\n")
}

/// 检查文件是否需要规范化行尾序列
fn needs_line_ending_normalization(file_path: &Path) -> bool {
    let extension = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
    matches!(extension, "sh" | "prop" | "txt" | "md" | "conf" | "json" | "toml" | "xml" | "yml" | "yaml")
        || file_path.file_name().and_then(|s| s.to_str()).map_or(false, |name| {
            matches!(name, "module.prop" | "service.sh" | "post-fs-data.sh" | "uninstall.sh" | "customize.sh")
        })
}

/// 复制文件并规范化行尾序列
fn copy_file_with_line_ending_normalization(src: &Path, dst: &Path) -> Result<()> {
    if needs_line_ending_normalization(src) {
        // 需要规范化行尾序列的文件
        let content = std::fs::read_to_string(src)?;
        let has_crlf = content.contains("\r\n") || content.contains("\r");
        let normalized_content = normalize_line_endings(&content);
        
        if has_crlf {
            // 修复源文件的行尾序列
            std::fs::write(src, &normalized_content)?;
            println!("    {} 修复源文件行尾序列: {}", "[~]".bright_yellow(), src.display());
        }
        
        // 写入构建目录
        std::fs::write(dst, normalized_content)?;
    } else {
        // 二进制文件或不需要规范化的文件
        std::fs::copy(src, dst)?;
    }
    Ok(())
}

/// 将 bash 风格的命令转换为 PowerShell 兼容的命令
fn convert_bash_to_powershell(command: &str) -> String {
    // 替换 && 为 ; (PowerShell 的命令分隔符)
    let mut converted = command.replace(" && ", "; ");
    
    // 处理其他常见的 bash 语法差异
    converted = converted.replace(" || ", "; if ($LASTEXITCODE -ne 0) { ");
    
    // 如果包含 || 的替换，需要在末尾添加结束括号
    if command.contains(" || ") {
        converted.push_str(" }");
    }
    
    converted
}

/// 应用排除规则并收集路径
fn apply_exclusions_and_collect_paths(
    project_path: &Path,
    entries: Vec<PathBuf>,
    is_source_packaging: bool,
    rmake_config: &RmakeConfig,
) -> Result<Vec<PathBuf>> {
    let mut paths_to_copy = Vec::new();
    let mut excluded_messages = Vec::new();
    
    // 编译排除模式
    let compiled_exclusions: Vec<regex::Regex> = rmake_config.build.exclude
        .iter()
        .filter_map(|pattern| {
            let trimmed = pattern.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                None // 忽略空行和注释
            } else {
                // 编译正则表达式
                match regex::Regex::new(&format!("^{}$", regex::escape(trimmed).replace(r"\*", ".*"))) {
                    Ok(re) => Some(re),
                    Err(e) => {
                        println!("⚠️ 警告: 排除模式编译失败 {}: {}", trimmed, e);
                        None
                    }
                }
            }
        })
        .collect();
      for entry in entries {
        let relative_path = entry.strip_prefix(project_path)?;
        
        // 检查是否被排除
        let mut is_excluded = false;
        let mut matched_pattern = None;
        
        for pattern_regex in &compiled_exclusions {
            if pattern_regex.is_match(&relative_path.display().to_string()) {
                is_excluded = true;
                matched_pattern = Some(pattern_regex.as_str());
                break;
            }
        }

        if is_excluded {
            // 确保正确区分文件和目录
            let item_type_str = if entry.is_dir() {
                "目录" // Directory
            } else {
                "文件" // File
            };

            let exclusion_reason = matched_pattern
                .map_or_else(String::new, |p| format!(" (匹配 {})", p.cyan()));

            excluded_messages.push(format!(
                "      [x] {} {}: {}{}",
                item_type_str, // 使用更准确的类型字符串
                if is_source_packaging { "排除源" } else { "排除" }.yellow(),
                relative_path.display().to_string().yellow(),
                exclusion_reason
            ));
            continue; // Skip this entry from being added to paths_to_copy
        }

        // If the entry is a file, add it to the list of paths to copy
        if entry.is_file() {
            paths_to_copy.push(entry);
        } else if entry.is_dir() {
            // If it's a directory, we may want to copy the whole directory
            // 这里可以根据需要决定是否复制整个目录
            paths_to_copy.push(entry);
        }
    }
    
    // 输出排除的文件和目录
    if !excluded_messages.is_empty() {
        println!("{} 排除的文件和目录:", "[!]".bright_yellow());
        for message in excluded_messages {
            println!("{}", message);
        }
    }
    
    Ok(paths_to_copy)
}
