use anyhow::Result;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

/// 检查 shellcheck 是否可用
pub fn is_shellcheck_available() -> bool {
    Command::new("shellcheck")
        .arg("--version")
        .output()
        .map_or(false, |output| output.status.success())
}

/// 获取 shellcheck 版本
pub fn get_shellcheck_version() -> Result<String> {
    let output = Command::new("shellcheck")
        .arg("--version")
        .output()?;

    if output.status.success() {
        let version_line = String::from_utf8_lossy(&output.stdout)
            .lines()
            .find(|line| line.to_lowercase().contains("version"))
            .unwrap_or("")
            .to_string();

        // Assuming version format like "ShellCheck - version 0.8.0" or "version: 0.8.0"
        if let Some(version_part) = version_line.split_whitespace().last() {
            // Further clean up if version is like "0.8.0
            // " or similar
            let version = version_part
                .trim_matches(|c: char| !c.is_digit(10) && c != '.')
                .to_string();
            if !version.is_empty() {
                Ok(version)
            } else {
                Err(anyhow::anyhow!(
                    "无法从输出中解析 shellcheck 版本: {}",
                    version_line
                ))
            }
        } else {
            Err(anyhow::anyhow!(
                "无法从输出中找到 shellcheck 版本号: {}",
                version_line
            ))
        }
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!(
            "执行 shellcheck --version 失败: {}",
            error_message
        ))
    }
}

/// Shellcheck 结果结构
#[derive(Debug, Clone, serde::Deserialize)] // Added serde::Deserialize
pub struct ShellCheckResult {
    pub file: String,
    pub line: u32,
    #[serde(alias = "endLine")]
    pub end_line: u32,
    pub column: u32,
    #[serde(alias = "endColumn")]
    pub end_column: u32,
    pub level: String, // "error", "warning", "info", "style"
    pub code: u32, // e.g., 2034
    pub message: String,
    // Optional: for future use with auto-fixing or more detailed suggestions
    // pub fix: Option<Value>, 
}

/// 检查项目中的 shell 脚本（带详细信息）
pub fn check_project(project_dir: &Path, _verbose: bool) -> Result<(Vec<ShellCheckResult>, bool)> {
    let mut results: Vec<ShellCheckResult> = Vec::new();
    let mut all_passed = true;

    println!("🔍 正在查找 shell 脚本文件于: {}", project_dir.display());

    for entry in WalkDir::new(project_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
        let extension = path.extension().unwrap_or_default().to_string_lossy().to_lowercase();

        // Basic check for .sh files or files named 'customize' (common in Magisk modules)
        // More sophisticated checks (e.g., shebang) can be added if needed.
        if extension == "sh" || file_name == "customize.sh" || file_name == "uninstall.sh" || file_name.starts_with("service.sh") {
            println!("  -> 发现脚本: {}", path.display());
            match Command::new("shellcheck")
                .arg("--format=json")
                .arg(path)
                .output()
            {
                Ok(output) => {
                    if output.status.success() {
                        // Even if shellcheck command succeeds, there might be no issues (empty stdout)
                        // or JSON output of issues.
                        if output.stdout.is_empty() {
                            // No issues found for this file
                            continue;
                        }
                        match serde_json::from_slice::<Vec<ShellCheckResult>>(&output.stdout) {
                            Ok(file_results) => {
                                for res in &file_results {
                                    if res.level == "error" {
                                        all_passed = false;
                                    }
                                }
                                results.extend(file_results);
                            }
                            Err(e) => {
                                eprintln!("⚠️  无法解析 shellcheck JSON 输出สำหรับ {}: {}", path.display(), e);
                                // Optionally, treat parsing errors as a failure
                                // all_passed = false;
                            }
                        }
                    } else {
                        // Shellcheck command itself failed (e.g., file not found, though WalkDir should prevent this)
                        // Or, if shellcheck returns non-zero for errors found (older versions might do this, modern ones use 0 for success with issues)
                        // We primarily rely on parsing the JSON. If JSON is empty, no issues.
                        // If JSON has items, those are the issues.
                        // If stderr has content, it's likely a shellcheck execution error.
                        if !output.stderr.is_empty() {
                             eprintln!(
                                "⚠️  Shellcheck 执行错误สำหรับ {}: {}",
                                path.display(),
                                String::from_utf8_lossy(&output.stderr)
                            );
                            all_passed = false; // Treat shellcheck execution error as a failure
                        }
                        // If stdout is not empty, try to parse it anyway, as some versions might output JSON and return error code.
                        if !output.stdout.is_empty() {
                             match serde_json::from_slice::<Vec<ShellCheckResult>>(&output.stdout) {
                                Ok(file_results) => {
                                    for res in &file_results {
                                        if res.level == "error" {
                                            all_passed = false;
                                        }
                                    }
                                    results.extend(file_results);
                                }
                                Err(e) => {
                                    eprintln!("⚠️  无法解析 shellcheck JSON 输出 (即使执行失败) สำหรับ {}: {}", path.display(), e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  执行 shellcheck 失败สำหรับ {}: {}", path.display(), e);
                    all_passed = false; // Failed to run shellcheck
                }
            }
        }
    }

    if results.is_empty() && all_passed {
        println!("✅ 未在任何脚本中发现问题。");
    } else if !results.is_empty() {
        println!("📊 发现 {} 个脚本问题。", results.len());
    }


    Ok((results, all_passed))
}

/// 格式化 shellcheck 结果
pub fn format_results(results: &[ShellCheckResult]) -> String {
    if results.is_empty() {
        return "✅ 所有 shell 脚本检查通过".to_string();
    }

    let mut formatted_output = String::new();
    let mut total_errors = 0;
    let mut total_warnings = 0;

    // Group results by file
    let mut results_by_file: std::collections::HashMap<String, Vec<&ShellCheckResult>> = std::collections::HashMap::new();
    for res in results {
        results_by_file.entry(res.file.clone()).or_default().push(res);
    }

    for (file_path, file_results) in &results_by_file {
        formatted_output.push_str(&format!("\n📄 文件: {}\n", file_path));
        for res in file_results {
            let level_icon = match res.level.as_str() {
                "error" => {
                    total_errors += 1;
                    "❌"
                }
                "warning" => {
                    total_warnings += 1;
                    "⚠️"
                }
                "info" => "ℹ️",
                "style" => "🎨",
                _ => "➡️",
            };
            formatted_output.push_str(&format!(
                "  {}:{}:{} [{}] SC{}: {}\n",
                level_icon, res.line, res.column, res.level, res.code, res.message
            ));
            formatted_output.push_str(&format!(
                "    (详细信息: https://www.shellcheck.net/wiki/SC{}\n",
                res.code
            ));
        }
    }

    formatted_output.push_str(&format!(
        "\n📊 总结: 共发现 {} 个错误, {} 个警告 (在 {} 个文件中)。\n",
        total_errors,
        total_warnings,
        results_by_file.len()
    ));

    formatted_output
}
