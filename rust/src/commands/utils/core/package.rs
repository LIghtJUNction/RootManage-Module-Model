use anyhow::Result;
use std::path::{Path, PathBuf};
use crate::commands::utils::core::project::ProjectConfig;
use crate::commands::utils::core::RmakeConfig;

/// 包信息结构
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub zip_path: PathBuf,
    pub zip_filename: String,
    pub source_path: PathBuf,
    pub source_filename: String,
}

/// 包管理器，处理模块打包和压缩相关操作
pub struct PackageManager;

impl PackageManager {
    /// 创建模块 ZIP 包
    pub fn create_module_zip(
        build_dir: &Path,
        dist_dir: &Path,
        zip_filename: &str,
    ) -> Result<PathBuf> {
        use std::fs::File;
        use zip::ZipWriter;
        use zip::write::FileOptions;
        use std::io::{Read, Write};
        use walkdir::WalkDir;

        let zip_path = dist_dir.join(zip_filename);
        println!("📦 创建模块包: {}", zip_path.display());

        let file = File::create(&zip_path)?;
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for entry in WalkDir::new(build_dir) {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                let relative_path = path.strip_prefix(build_dir)?;
                let relative_path_str = relative_path.to_string_lossy().replace('\\', "/");
                
                zip.start_file(relative_path_str, options)?;
                
                let mut file = File::open(path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
            }
        }

        zip.finish()?;
        println!("✅ 模块包创建完成: {}", zip_filename);
        Ok(zip_path)
    }

    /// 创建源代码归档
    pub fn create_source_archive(
        project_root: &Path,
        dist_dir: &Path,
        source_filename: &str,
        exclude_items: &[&str],
    ) -> Result<PathBuf> {
        use std::fs::File;
        use zip::ZipWriter;
        use zip::write::FileOptions;
        use std::io::{Read, Write};
        use walkdir::WalkDir;

        let source_path = dist_dir.join(source_filename);
        println!("📦 创建源代码归档: {}", source_path.display());

        let file = File::create(&source_path)?;
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for entry in WalkDir::new(project_root) {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                let relative_path = path.strip_prefix(project_root)?;
                
                // 检查是否应该排除
                if Self::should_exclude_from_source(relative_path, exclude_items) {
                    continue;
                }
                
                let relative_path_str = relative_path.to_string_lossy().replace('\\', "/");
                
                zip.start_file(relative_path_str, options)?;
                
                let mut file = File::open(path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
            }
        }

        zip.finish()?;
        println!("✅ 源代码归档创建完成: {}", source_filename);
        Ok(source_path)
    }

    /// 检查文件是否应该从源代码归档中排除
    fn should_exclude_from_source(path: &Path, exclude_items: &[&str]) -> bool {
        let path_str = path.to_string_lossy();
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        
        // 默认排除构建目录和常见的开发文件
        let default_excludes = [
            ".git", ".gitignore", ".rmmp", "dist", "build",
            "*.log", "*.tmp", ".DS_Store", "Thumbs.db"
        ];
        
        for exclude in default_excludes.iter().chain(exclude_items.iter()) {
            if exclude.contains('*') {
                if exclude.starts_with("*.") && file_name.ends_with(&exclude[1..]) {
                    return true;
                }
            } else if path_str.contains(exclude) || file_name == *exclude {
                return true;
            }
        }
        
        false
    }

    /// 生成 ZIP 文件名，支持变量替换
    pub fn generate_zip_filename(config: &ProjectConfig, rmake_config: Option<&RmakeConfig>) -> Result<String> {
        let template = if let Some(rmake) = rmake_config {
            if let Some(ref package) = rmake.package {
                if let Some(ref zip_name) = package.zip_name {
                    if zip_name == "default" {
                        "{name}-{version}-{version_code}.zip".to_string()
                    } else {
                        zip_name.clone()
                    }
                } else {
                    "{name}-{version}-{version_code}.zip".to_string()
                }
            } else {
                "{name}-{version}-{version_code}.zip".to_string()
            }
        } else {
            "{name}-{version}-{version_code}.zip".to_string()
        };
        
        Self::replace_template_variables(&template, config)
    }

    /// 生成源代码文件名
    pub fn generate_source_filename(config: &ProjectConfig, rmake_config: Option<&RmakeConfig>) -> Result<String> {
        let template = if let Some(rmake) = rmake_config {
            if let Some(ref package) = rmake.package {
                if let Some(ref source_name) = package.source_name {
                    if source_name == "default" {
                        "{name}-{version}-source.zip".to_string()
                    } else {
                        source_name.clone()
                    }
                } else {
                    "{name}-{version}-source.zip".to_string()
                }
            } else {
                "{name}-{version}-source.zip".to_string()
            }
        } else {
            "{name}-{version}-source.zip".to_string()
        };
        
        Self::replace_template_variables(&template, config)
    }

    /// 替换模板中的变量
    fn replace_template_variables(template: &str, config: &ProjectConfig) -> Result<String> {
        let mut result = template.to_string();
        
        // 获取 Git 提交 hash
        let git_hash = "unknown".to_string(); // 简化版本，实际应该获取 git commit hash
        let short_hash = if git_hash.len() >= 8 { &git_hash[..8] } else { &git_hash };
        
        // 获取当前时间
        let now = chrono::Utc::now();
        let date = now.format("%Y%m%d").to_string();
        let datetime = now.format("%Y%m%d_%H%M%S").to_string();
        let timestamp = now.timestamp().to_string();
        
        // 获取作者信息
        let author_name = config.authors.first()
            .map(|a| a.name.as_str())
            .unwrap_or("unknown");
        let author_email = config.authors.first()
            .map(|a| a.email.as_str())
            .unwrap_or("unknown");
        
        // 定义变量映射
        let variables = [        
            ("{id}", config.id.as_str()),
            ("{name}", config.name.as_str()),
            ("{version}", config.version.as_deref().unwrap_or("unknown")),
            ("{version_code}", config.version_code.as_str()),
            ("{author}", author_name),
            ("{email}", author_email),
            ("{hash}", &git_hash),
            ("{short_hash}", short_hash),
            ("{date}", &date),
            ("{datetime}", &datetime),
            ("{timestamp}", &timestamp),
        ];
        
        // 执行替换
        for (var, value) in &variables {
            result = result.replace(var, value);
        }
        
        println!("📝 文件名模板: '{}' -> '{}'", template, result);
        
        Ok(result)
    }
}
