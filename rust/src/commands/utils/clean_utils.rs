
use anyhow::Result;
use std::fs;
use std::path::Path;

/// 清理目录下的文件
pub fn clean_directory(dir_path: &str, patterns: &[&str], dry_run: bool, file_count: &mut usize) -> Result<u64> {
    let path = Path::new(dir_path);
    
    if !path.exists() {
        return Ok(0);
    }

    let mut total_size = 0u64;

    if path.is_dir() {
        let entries = fs::read_dir(path)?;
        
        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();
            
            if should_clean_file(&entry_path, patterns) {
                let metadata = entry.metadata()?;
                total_size += metadata.len();
                *file_count += 1;
                
                if dry_run {
                    println!("  🗑️  {}", entry_path.display());
                } else {
                    if entry_path.is_dir() {
                        fs::remove_dir_all(&entry_path)?;
                        println!("  🗂️  已删除目录: {}", entry_path.display());
                    } else {
                        fs::remove_file(&entry_path)?;
                        println!("  📄 已删除文件: {}", entry_path.display());
                    }
                }
            }
        }
        
        // 如果目录为空且不是根目录，则删除目录本身
        if !dry_run && dir_path != "." && dir_path != ".rmmp" {
            if let Ok(entries) = fs::read_dir(path) {
                if entries.count() == 0 {
                    fs::remove_dir(path)?;
                    println!("  🗂️  已删除空目录: {}", path.display());
                }
            }
        }
    }

    Ok(total_size)
}

/// 清理单个文件
pub fn clean_file(file_path: &str, dry_run: bool, file_count: &mut usize) -> Result<u64> {
    let path = Path::new(file_path);
    
    if !path.exists() {
        return Ok(0);
    }

    let metadata = path.metadata()?;
    let size = metadata.len();
    *file_count += 1;

    if dry_run {
        println!("  🗑️  {}", path.display());
    } else {
        fs::remove_file(path)?;
        println!("  📄 已删除文件: {}", path.display());
    }

    Ok(size)
}

/// 检查文件是否应该被清理
pub fn should_clean_file(path: &Path, patterns: &[&str]) -> bool {
    if patterns.contains(&"*") {
        return true;
    }

    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    for pattern in patterns {
        if pattern.ends_with("*") {
            let prefix = &pattern[..pattern.len()-1];
            if file_name.starts_with(prefix) {
                return true;
            }
        } else if pattern.starts_with("*") {
            let suffix = &pattern[1..];
            if file_name.ends_with(suffix) {
                return true;
            }
        } else if file_name == *pattern {
            return true;
        }
    }

    false
}