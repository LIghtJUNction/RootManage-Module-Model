
/// 在构建目录中寻找最新的模块文件
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn find_latest_build_files(dist_dir: &Path, project_id: &str) -> Result<(PathBuf, PathBuf)> {
    if !dist_dir.exists() {
        anyhow::bail!("❌ 构建目录不存在: {}\n请先运行 'rmm build' 构建项目", dist_dir.display());
    }
    
    // 查找所有匹配的ZIP文件
    let mut zip_files = Vec::new();
    let mut tar_files = Vec::new();
    
    for entry in std::fs::read_dir(dist_dir)? {
        let entry = entry?;
        let path = entry.path();
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
          // 查找匹配项目ID的ZIP文件
        if filename.ends_with(".zip") && filename.starts_with(project_id) {
            let metadata = entry.metadata()?;
            zip_files.push((path.clone(), metadata.modified()?));
        }
        
        // 查找匹配项目ID的源码包
        if filename.ends_with("-source.tar.gz") && filename.starts_with(project_id) {
            let metadata = entry.metadata()?;
            tar_files.push((path.clone(), metadata.modified()?));
        }
    }
    
    if zip_files.is_empty() {
        anyhow::bail!("❌ 未找到模块包文件 ({}*.zip)\n请先运行 'rmm build' 构建项目", project_id);
    }
    
    if tar_files.is_empty() {
        anyhow::bail!("❌ 未找到源码包文件 ({}*-source.tar.gz)\n请先运行 'rmm build' 构建项目", project_id);
    }
    
    // 按修改时间排序，获取最新的文件
    zip_files.sort_by(|a, b| b.1.cmp(&a.1));
    tar_files.sort_by(|a, b| b.1.cmp(&a.1));
    
    let latest_zip = zip_files.into_iter().next().unwrap().0;
    let latest_tar = tar_files.into_iter().next().unwrap().0;
    
    println!("📦 找到最新模块包: {}", latest_zip.file_name().unwrap().to_string_lossy());
    println!("📋 找到最新源码包: {}", latest_tar.file_name().unwrap().to_string_lossy());
    
    Ok((latest_zip, latest_tar))
}
