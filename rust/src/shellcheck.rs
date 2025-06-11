use anyhow::Result;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;
use serde_json;
use chrono;

/// Shellcheck 检查结果
#[derive(Debug)]
pub struct ShellcheckResult {
    pub file_path: String,
    pub is_success: bool,
    pub output: String,
    pub error_count: usize,
    pub warning_count: usize,
}

/// 检查 shellcheck 是否可用
pub fn is_shellcheck_available() -> bool {
    Command::new("shellcheck")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// 获取 shellcheck 版本信息
pub fn get_shellcheck_version() -> Result<String> {
    let output = Command::new("shellcheck")
        .arg("--version")
        .output()?;
    
    if output.status.success() {
        let version_info = String::from_utf8_lossy(&output.stdout);
        // 提取版本号
        for line in version_info.lines() {
            if line.starts_with("version:") {
                return Ok(line.replace("version:", "").trim().to_string());
            }
        }
        Ok("unknown".to_string())
    } else {
        Err(anyhow::anyhow!("Failed to get shellcheck version"))
    }
}

/// 查找项目中的所有 shell 脚本文件
pub fn find_shell_scripts(project_root: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut shell_files = Vec::new();
    
    for entry in WalkDir::new(project_root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        // 跳过隐藏目录和构建目录
        if path.components().any(|c| {
            let name = c.as_os_str().to_string_lossy();
            name.starts_with('.') || 
            name == "target" || 
            name == "node_modules" ||
            name == "__pycache__"
        }) {
            continue;
        }
        
        if is_shell_script(path) {
            shell_files.push(path.to_path_buf());
        }
    }
    
    Ok(shell_files)
}

/// 判断文件是否为 shell 脚本
fn is_shell_script(path: &Path) -> bool {
    // 检查文件扩展名
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        if matches!(ext.as_str(), "sh" | "bash" | "zsh" | "fish") {
            return true;
        }
    }
      // 检查文件名
    if let Some(name) = path.file_name() {
        let name = name.to_string_lossy();
        let name_str = name.as_ref();
        if matches!(name_str, "configure" | "install" | "setup") {
            return true;
        }
    }
    
    // 检查 shebang
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Some(first_line) = content.lines().next() {
            if first_line.starts_with("#!") && 
               (first_line.contains("sh") || 
                first_line.contains("bash") || 
                first_line.contains("zsh")) {
                return true;
            }
        }
    }
    
    false
}

/// 对单个文件进行 shellcheck 检查
pub fn check_file(file_path: &Path) -> Result<ShellcheckResult> {
    let output = Command::new("shellcheck")
        .arg("--format=json")
        .arg("--severity=error")
        .arg("--severity=warning")
        .arg("--severity=info")
        .arg("--severity=style")
        .arg(file_path)
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // 解析 JSON 输出来计算错误和警告数量
    let mut error_count = 0;
    let mut warning_count = 0;
    
    if !stdout.is_empty() {
        // 简单的 JSON 解析来计数问题
        let lines: Vec<&str> = stdout.lines().collect();
        for line in lines {
            if line.contains("\"level\":") {
                if line.contains("\"error\"") {
                    error_count += 1;
                } else if line.contains("\"warning\"") || line.contains("\"info\"") || line.contains("\"style\"") {
                    warning_count += 1;
                }
            }
        }
    }
    
    let result_output = if !stdout.is_empty() {
        stdout.to_string()
    } else if !stderr.is_empty() {
        stderr.to_string()
    } else {
        "No issues found".to_string()
    };
    
    Ok(ShellcheckResult {
        file_path: file_path.to_string_lossy().to_string(),
        is_success: output.status.success() && error_count == 0,
        output: result_output,
        error_count,
        warning_count,
    })
}

/// 对多个文件进行 shellcheck 检查
pub fn check_files(file_paths: &[std::path::PathBuf]) -> Result<Vec<ShellcheckResult>> {
    let mut results = Vec::new();
    
    for file_path in file_paths {
        match check_file(file_path) {
            Ok(result) => results.push(result),
            Err(e) => {
                println!("⚠️  检查文件 {} 时出错: {}", file_path.display(), e);
                results.push(ShellcheckResult {
                    file_path: file_path.to_string_lossy().to_string(),
                    is_success: false,
                    output: format!("检查失败: {}", e),
                    error_count: 1,
                    warning_count: 0,
                });
            }
        }
    }
    
    Ok(results)
}

/// 格式化输出检查结果
pub fn format_results(results: &[ShellcheckResult], verbose: bool) -> String {
    let mut output = String::new();
    let total_files = results.len();
    let successful_files = results.iter().filter(|r| r.is_success).count();
    let total_errors: usize = results.iter().map(|r| r.error_count).sum();
    let total_warnings: usize = results.iter().map(|r| r.warning_count).sum();
    
    output.push_str(&format!("🔍 Shellcheck 检查结果:\n"));
    output.push_str(&format!("📊 检查文件: {} 个\n", total_files));
    output.push_str(&format!("✅ 通过检查: {} 个\n", successful_files));
    output.push_str(&format!("❌ 错误总数: {} 个\n", total_errors));
    output.push_str(&format!("⚠️  警告总数: {} 个\n", total_warnings));
    
    if verbose || total_errors > 0 {
        output.push_str("\n📋 详细结果:\n");
        for result in results {
            if !result.is_success || verbose {
                output.push_str(&format!("\n📄 文件: {}\n", result.file_path));
                if result.is_success {
                    output.push_str("✅ 状态: 通过\n");
                } else {
                    output.push_str("❌ 状态: 失败\n");
                }
                
                if result.error_count > 0 {
                    output.push_str(&format!("❌ 错误: {} 个\n", result.error_count));
                }
                if result.warning_count > 0 {
                    output.push_str(&format!("⚠️  警告: {} 个\n", result.warning_count));
                }
                
                if !result.output.is_empty() && result.output != "No issues found" {
                    output.push_str("📋 详情:\n");
                    // 简化 JSON 输出为更易读的格式
                    if result.output.starts_with('[') || result.output.starts_with('{') {
                        output.push_str("  (JSON格式输出，建议直接运行 shellcheck 查看详细信息)\n");
                    } else {
                        for line in result.output.lines() {
                            output.push_str(&format!("  {}\n", line));
                        }
                    }
                }
            }
        }
    }
    
    output
}

/// 快速检查项目中的所有 shell 脚本
pub fn check_project(project_root: &Path, verbose: bool) -> Result<(Vec<ShellcheckResult>, bool)> {
    println!("🔍 正在查找 shell 脚本文件...");
    
    let shell_files = find_shell_scripts(project_root)?;
    
    if shell_files.is_empty() {
        println!("📋 未找到 shell 脚本文件");
        return Ok((Vec::new(), true));
    }
    
    println!("📄 发现 {} 个 shell 脚本文件:", shell_files.len());
    for file in &shell_files {
        println!("  - {}", file.display());
    }
    
    println!("\n🔍 正在进行 shellcheck 检查...");
    let results = check_files(&shell_files)?;
    
    let all_passed = results.iter().all(|r| r.is_success);
    let output = format_results(&results, verbose);
    println!("{}", output);
    
    // 保存详细结果到 .rmmp 目录
    save_results_to_file(project_root, &results)?;
    
    Ok((results, all_passed))
}

/// 保存 shellcheck 检查结果到文件
fn save_results_to_file(project_root: &Path, results: &[ShellcheckResult]) -> Result<()> {
    use std::fs;
    
    // 创建 .rmmp 目录
    let rmmp_dir = project_root.join(".rmmp");
    fs::create_dir_all(&rmmp_dir)?;
    
    // 创建检查报告文件
    let report_file = rmmp_dir.join("shellcheck_report.txt");
    let detailed_file = rmmp_dir.join("shellcheck_details.json");
    
    // 生成时间戳
    let now = chrono::Utc::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S UTC").to_string();
    
    // 写入文本报告
    let mut report_content = String::new();
    report_content.push_str(&format!("# Shellcheck 检查报告\n"));
    report_content.push_str(&format!("检查时间: {}\n", timestamp));
    report_content.push_str(&format!("检查工具: shellcheck\n\n"));
    
    let total_files = results.len();
    let successful_files = results.iter().filter(|r| r.is_success).count();
    let total_errors: usize = results.iter().map(|r| r.error_count).sum();
    let total_warnings: usize = results.iter().map(|r| r.warning_count).sum();
    
    report_content.push_str(&format!("## 汇总信息\n"));
    report_content.push_str(&format!("- 检查文件: {} 个\n", total_files));
    report_content.push_str(&format!("- 通过检查: {} 个\n", successful_files));
    report_content.push_str(&format!("- 失败检查: {} 个\n", total_files - successful_files));
    report_content.push_str(&format!("- 错误总数: {} 个\n", total_errors));
    report_content.push_str(&format!("- 警告总数: {} 个\n\n", total_warnings));
    
    report_content.push_str("## 详细结果\n\n");
    
    for result in results {
        report_content.push_str(&format!("### 文件: {}\n", result.file_path));
        
        if result.is_success {
            report_content.push_str("✅ 状态: 通过\n");
        } else {
            report_content.push_str("❌ 状态: 失败\n");
        }
        
        if result.error_count > 0 {
            report_content.push_str(&format!("❌ 错误: {} 个\n", result.error_count));
        }
        if result.warning_count > 0 {
            report_content.push_str(&format!("⚠️  警告: {} 个\n", result.warning_count));
        }
        
        if !result.output.is_empty() && result.output != "No issues found" {
            report_content.push_str("\n#### 详细信息:\n");
            report_content.push_str("```\n");
            report_content.push_str(&result.output);
            report_content.push_str("\n```\n");
        }
        
        report_content.push_str("\n---\n\n");
    }
    
    // 写入建议
    if total_errors > 0 || total_warnings > 0 {
        report_content.push_str("## 修复建议\n\n");
        report_content.push_str("要查看详细的错误信息和修复建议，请运行:\n");
        report_content.push_str("```bash\n");
        for result in results {
            if !result.is_success {
                report_content.push_str(&format!("shellcheck \"{}\"\n", result.file_path));
            }
        }
        report_content.push_str("```\n\n");
        report_content.push_str("或者查看详细的 JSON 格式报告: `.rmmp/shellcheck_details.json`\n");
    }
    
    // 写入文本报告
    fs::write(&report_file, report_content)?;
    
    // 写入 JSON 详细报告
    let json_data = serde_json::json!({
        "timestamp": timestamp,
        "summary": {
            "total_files": total_files,
            "successful_files": successful_files,
            "failed_files": total_files - successful_files,
            "total_errors": total_errors,
            "total_warnings": total_warnings
        },
        "results": results.iter().map(|r| {
            serde_json::json!({
                "file_path": r.file_path,
                "is_success": r.is_success,
                "error_count": r.error_count,
                "warning_count": r.warning_count,
                "raw_output": r.output
            })
        }).collect::<Vec<_>>()
    });
    
    fs::write(&detailed_file, serde_json::to_string_pretty(&json_data)?)?;
    
    println!("📄 检查报告已保存:");
    println!("  - 文本报告: {}", report_file.display());
    println!("  - JSON 详情: {}", detailed_file.display());
    
    Ok(())
}
