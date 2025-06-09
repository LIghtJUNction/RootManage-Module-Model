use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use std::process::Command;
use anyhow::{Result, anyhow};
use regex::Regex;

// === 类型定义 ===
type ProjectInfo = HashMap<String, PyObject>;
type Projects = HashMap<String, String>;

// === 项目元类功能 ===

/// 项目信息缓存
static mut PROJECT_CACHE: Option<HashMap<String, ProjectInfo>> = None;
static mut PROJECT_MTIME: Option<HashMap<String, f64>> = None;

fn get_project_cache() -> &'static mut HashMap<String, ProjectInfo> {
    unsafe {
        PROJECT_CACHE.get_or_insert_with(HashMap::new)
    }
}

fn get_project_mtime() -> &'static mut HashMap<String, f64> {
    unsafe {
        PROJECT_MTIME.get_or_insert_with(HashMap::new)
    }
}

/// 获取项目元数据配置
#[pyfunction]
fn get_projects_meta(py: Python) -> PyResult<PyObject> {
    // 这里需要调用 Config.META.get("projects", {})
    let config_module = py.import("pyrmm.usr.lib.config")?;
    let config_class = config_module.getattr("Config")?;
    let meta = config_class.getattr("META")?;
    let projects = meta.call_method1("get", ("projects", PyDict::new(py)))?;
      // 检查是否为字符串（错误情况）
    if projects.extract::<String>().is_ok() {
        return Err(pyo3::exceptions::PyAttributeError::new_err(
            format!("项目配置错误!： '{}' 请检查配置文件", projects.extract::<String>()?)
        ));
    }
    
    Ok(projects.into_pyobject(py)?.into_any().unbind())
}

/// 获取项目路径
#[pyfunction]
fn project_path(py: Python, project_name: &str) -> PyResult<String> {
    let meta = get_projects_meta(py)?;
    let projects_dict = meta.downcast_bound::<PyDict>(py)?;
    
    if let Some(project_path) = projects_dict.get_item(project_name)? {
        let path_str = project_path.extract::<String>()?;
        let path = Path::new(&path_str);
        
        if path.exists() {
            Ok(path_str)
        } else {
            Err(pyo3::exceptions::PyFileNotFoundError::new_err(
                format!("项目路径不存在: {}", path_str)
            ))
        }
    } else {
        Err(pyo3::exceptions::PyKeyError::new_err(
            format!("项目 '{}' 不存在于配置中。", project_name)
        ))
    }
}

/// 获取项目信息（带缓存）
#[pyfunction]
fn project_info(py: Python, project_path: &str) -> PyResult<PyObject> {
    let path = Path::new(project_path);
    
    if !path.exists() {
        return Err(pyo3::exceptions::PyFileNotFoundError::new_err(
            format!("项目路径不存在: {}", project_path)
        ));
    }
    
    let meta_file = path.join("rmmproject.toml");
    if !meta_file.exists() {
        return Err(pyo3::exceptions::PyFileNotFoundError::new_err(
            format!("项目元数据文件不存在: {}", meta_file.display())
        ));
    }
    
    let cache_key = meta_file.canonicalize()
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?
        .to_string_lossy()
        .to_string();
    
    // 检查文件修改时间
    let metadata = fs::metadata(&meta_file)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
    let current_mtime = metadata.modified()
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    
    let cache = get_project_cache();
    let mtime_cache = get_project_mtime();
    
    // 检查缓存
    if let (Some(cached_info), Some(&cached_mtime)) = (cache.get(&cache_key), mtime_cache.get(&cache_key)) {
        if (cached_mtime - current_mtime).abs() < 0.001 {
            return Ok(cached_info.clone().into_pyobject(py)?.into_any().unbind());
        }
    }
    
    // 读取和解析 TOML 文件
    let content = fs::read_to_string(&meta_file)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
    
    let toml_value: toml::Value = toml::from_str(&content)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    
    // 转换为 Python 对象
    let py_dict = toml_value_to_py_object(py, &toml_value)?;
      // 更新缓存
    cache.insert(cache_key.clone(), py_dict.bind(py).clone().unbind());
    mtime_cache.insert(cache_key, current_mtime);
    
    Ok(py_dict)
}

/// 设置项目配置
#[pyfunction]
fn set_project_config(py: Python, name: &str, value: PyObject) -> PyResult<()> {
    let project_path_str = project_path(py, name)?;
    let project_path = Path::new(&project_path_str);
    
    // 获取当前项目信息
    let mut current_info = project_info(py, &project_path_str)?;
    let current_dict = current_info.downcast_bound::<PyDict>(py)?;
    
    // 更新信息
    if let Ok(value_dict) = value.downcast_bound::<PyDict>(py) {
        current_dict.update(value_dict.as_mapping())?;
    }
    
    // 写入文件
    let meta_file = project_path.join("rmmproject.toml");
    let toml_value = py_object_to_toml_value(py, &current_info)?;
    let toml_string = toml::to_string(&toml_value)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    
    fs::write(&meta_file, toml_string)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
    
    // 清理缓存
    let cache_key = meta_file.canonicalize()
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?
        .to_string_lossy()
        .to_string();
    
    get_project_cache().remove(&cache_key);
    get_project_mtime().remove(&cache_key);
    
    Ok(())
}

/// 删除项目配置
#[pyfunction]
fn delete_project_config(py: Python, name: &str) -> PyResult<()> {
    // 尝试获取项目路径并删除目录
    if let Ok(project_path_str) = project_path(py, name) {
        let project_path = Path::new(&project_path_str);
        if project_path.exists() {
            fs::remove_dir_all(project_path)
                .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
            println!("项目目录 '{}' 已删除", project_path.display());
        }
    }
    
    // 从配置中移除项目记录
    let config_module = py.import("pyrmm.usr.lib.config")?;
    let config_class = config_module.getattr("Config")?;
    let mut projects = get_projects_meta(py)?;
    let projects_dict = projects.downcast_bound::<PyDict>(py)?;
    
    if projects_dict.contains(name)? {
        projects_dict.del_item(name)?;
        config_class.setattr("projects", projects)?;
        println!("项目 '{}' 已从配置中移除", name);
    }
    
    Ok(())
}

// === 项目类功能 ===

/// 添加现有项目到配置
#[pyfunction]
fn add_project(py: Python, project_name: &str, project_path: &str) -> PyResult<()> {
    let path = Path::new(project_path);
    
    if !path.exists() {
        return Err(pyo3::exceptions::PyFileNotFoundError::new_err(
            format!("项目路径不存在: {}", project_path)
        ));
    }
    
    if !is_rmmproject(project_path) {
        return Err(pyo3::exceptions::PyValueError::new_err(
            format!("路径 {} 不是一个有效的 RMM 项目", project_path)
        ));
    }
    
    // 获取当前项目配置
    let config_module = py.import("pyrmm.usr.lib.config")?;
    let config_class = config_module.getattr("Config")?;
    let meta = config_class.getattr("META")?;    let projects = meta.call_method1("get", ("projects", PyDict::new(py)))?;
    
    if let Ok(projects_dict) = projects.downcast_bound::<PyDict>(py) {
        let canonical_path = path.canonicalize()
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        projects_dict.set_item(project_name, canonical_path.to_string_lossy().to_string())?;
        config_class.setattr("projects", projects)?;
    } else {
        return Err(pyo3::exceptions::PyAttributeError::new_err("项目配置格式错误"));
    }
    
    Ok(())
}

/// 检查是否是有效的 RMM 项目
#[pyfunction]
fn is_valid_item(py: Python, item_name: &str) -> PyResult<bool> {
    match project_path(py, item_name) {
        Ok(path_str) => Ok(is_rmmproject(&path_str)),
        Err(_) => Ok(false),
    }
}

/// 获取同步提示信息
#[pyfunction]
fn get_sync_prompt(item_name: &str) -> String {
    format!("项目 '{}' 不是一个有效的 RMM 项目。移除？", item_name)
}

/// 检查路径是否是 RMM 项目
#[pyfunction]
fn is_rmmproject(project_path: &str) -> bool {
    let meta_file = Path::new(project_path).join("rmmproject.toml");
    meta_file.exists() && meta_file.is_file()
}

/// 初始化新的 RMM 项目
#[pyfunction]
fn init_project(py: Python, project_path: &str) -> PyResult<PyObject> {
    let path = Path::new(project_path);
    let project_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unnamed_project");
    
    // 确保项目目录存在
    fs::create_dir_all(path)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
    
    // Git 仓库检测
    let git_info = detect_git_info(path);
    let config_module = py.import("pyrmm.usr.lib.config")?;
    let config_class = config_module.getattr("Config")?;
    let config_username = config_class.getattr("username")?.extract::<String>()?;
    let config_email = config_class.getattr("email")?.extract::<String>()?;
    let config_version = config_class.getattr("version")?.extract::<String>()?;
    
    let mut username = config_username.clone();
    let mut repo_name = project_name.to_string();
    let mut is_in_repo_root = false;
    
    let mut github_url = format!("https://github.com/{}/{}", username, project_name);
    let mut update_json_url = format!("https://raw.githubusercontent.com/{}/{}/main/update.json", username, project_name);
    
    if let Some(ref git) = git_info {
        if let Some(ref remote) = git.get("remote_info") {
            if let (Some(git_username), Some(git_repo)) = (
                remote.get("username").and_then(|v| v.as_str()),
                remote.get("repo_name").and_then(|v| v.as_str())
            ) {
                username = git_username.to_string();
                repo_name = git_repo.to_string();
                
                // 如果配置用户名是默认值，自动更新
                if config_username == "username" {
                    config_class.setattr("username", &username)?;
                    println!("📝 自动更新配置用户名: {}", username);
                }
                
                is_in_repo_root = git.get("is_in_repo_root")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                if is_in_repo_root {
                    github_url = format!("https://github.com/{}/{}", username, repo_name);
                    update_json_url = format!("https://raw.githubusercontent.com/{}/{}/main/update.json", username, repo_name);
                }
                
                if let Some(url) = remote.get("url").and_then(|v| v.as_str()) {
                    println!("检测到 Git 仓库: {}", url);
                }
                println!("用户名: {}, 仓库名: {}", username, repo_name);
                println!("项目位置: {}", if is_in_repo_root { "仓库根目录" } else { "子目录" });
            }
        }
    }
    
    // 创建项目信息
    let mut project_info = PyDict::new(py);
    project_info.set_item("id", project_name)?;
    project_info.set_item("name", project_name)?;
    project_info.set_item("requires_rmm", format!(">={}", config_version))?;
    project_info.set_item("versionCode", path.canonicalize()
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?
        .to_string_lossy().to_string())?;
    project_info.set_item("updateJson", update_json_url)?;
    project_info.set_item("readme", "README.MD")?;
    project_info.set_item("changelog", "CHANGELOG.MD")?;
    project_info.set_item("lecense", "LICENSE")?;
    
    // URLs
    let urls = PyDict::new(py);
    urls.set_item("github", github_url)?;
    project_info.set_item("urls", urls)?;
      // Dependencies
    let deps = PyList::new(py, [PyDict::new(py)])?;
    if let Some(first_dep) = deps.get_item(0)? {
        let dep_dict = first_dep.downcast::<PyDict>()?;
        dep_dict.set_item("dep?", "?version")?;
    }
    project_info.set_item("dependencies", deps)?;
    
    // Authors
    let author = PyDict::new(py);
    author.set_item("name", &username)?;
    author.set_item("email", config_email)?;    let authors = PyList::new(py, [author])?;
    project_info.set_item("authors", authors)?;
    
    // Scripts
    let script = PyDict::new(py);
    script.set_item("build", "rmm build")?;
    let scripts = PyList::new(py, [script])?;
    project_info.set_item("scripts", scripts)?;
    
    // 添加 Git 信息
    if let Some(git) = git_info {
        let git_dict = serde_json_to_py_object(py, &git)?;
        project_info.set_item("git", git_dict)?;
    }
    
    // 写入项目元数据文件
    let meta_file = path.join("rmmproject.toml");
    let toml_value = py_object_to_toml_value(py, &project_info.into_pyobject(py)?.into_any().unbind())?;
    let toml_string = toml::to_string(&toml_value)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    
    fs::write(&meta_file, toml_string)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
    
    // 生成版本信息并创建 module.prop
    let version_module = py.import("pyrmm.usr.lib.version")?;
    let version_generator = version_module.getattr("VersionGenerator")?;
    let version_info = version_generator.call_method("generate", ("", path.to_string_lossy().as_ref()), None)?;
      let version = version_info.get_item("version")?.ok_or_else(|| 
        pyo3::exceptions::PyKeyError::new_err("Missing version in version_info"))?.extract::<String>()?;
    let version_code = version_info.get_item("versionCode")?.ok_or_else(|| 
        pyo3::exceptions::PyKeyError::new_err("Missing versionCode in version_info"))?.extract::<String>()?;
    
    let author_name = project_info.get_item("authors")?
        .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err("Missing authors"))?
        .downcast::<PyList>()?
        .get_item(0)?
        .ok_or_else(|| pyo3::exceptions::PyIndexError::new_err("Empty authors list"))?
        .downcast::<PyDict>()?
        .get_item("name")?
        .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err("Missing author name"))?
        .extract::<String>()?;
    
    // 创建 module.prop
    let module_prop = path.join("module.prop");    let module_prop_content = format!(
        "id={}\nname={}\nversion={}\nversionCode={}\nauthor={}\ndescription=RMM项目 {}\nupdateJson={}\n",
        project_name, project_name, version, version_code, author_name, project_name,
        project_info.get_item("updateJson")?
            .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err("Missing updateJson"))?
            .extract::<String>()?
    );
    
    fs::write(&module_prop, module_prop_content)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
      // 创建必要的文件
    create_project_files(path, project_name, &author_name)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
    
    // 将项目路径添加到配置中
    let config_meta = config_class.getattr("META")?;    let projects = config_meta.call_method1("get", ("projects", PyDict::new(py)))?;
    if let Ok(projects_dict) = projects.downcast_bound::<PyDict>(py) {
        projects_dict.set_item(project_name, path.canonicalize()
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?
            .to_string_lossy().to_string())?;
        config_class.setattr("projects", projects)?;
    }
    
    Ok(project_info.into_pyobject(py)?.into_any().unbind())
}

/// 同步项目
#[pyfunction]
fn sync_project(py: Python, project_name: &str) -> PyResult<()> {
    // 检查项目有效性
    if !is_valid_item(py, project_name)? {
        // 这里应该调用 sync_item，但简化处理
        println!("项目 '{}' 无效，需要移除", project_name);
        return Ok(());
    }
    
    // 对于有效项目，更新版本信息
    let project_path_str = project_path(py, project_name)?;
    let path = Path::new(&project_path_str);
    
    // 使用 VersionGenerator 来生成并更新版本信息
    let version_module = py.import("pyrmm.usr.lib.version")?;
    let version_generator = version_module.getattr("VersionGenerator")?;
    
    // 读取当前版本
    let current_version = match project_info(py, &project_path_str) {        Ok(info) => {
            info.downcast_bound::<PyDict>(py)?
                .get_item("version")?
                .map(|v| v.extract::<String>().unwrap_or_else(|_| "v1.0.0".to_string()))
                .unwrap_or_else(|| "v1.0.0".to_string())
        },
        Err(_) => "v1.0.0".to_string(),
    };
    
    // 自动判断升级类型并更新版本
    let version_info = version_generator.call_method("auto_bump", (current_version, project_path_str.clone()), None)?;
    version_generator.call_method("update_project_files", (project_path_str, version_info), None)?;
    
    Ok(())
}

/// 初始化基础项目
#[pyfunction]
fn init_basic(py: Python, project_path: &str) -> PyResult<PyObject> {
    let result = init_project(py, project_path)?;
    let path = Path::new(project_path);
    let system_dir = path.join("system");
    fs::create_dir_all(system_dir)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
    
    let result_dict = PyDict::new(py);    result_dict.set_item("message", "RMM basic project initialized.")?;
    Ok(result_dict.into_pyobject(py)?.into_any().unbind())
}

/// 初始化库项目
#[pyfunction]
fn init_library(py: Python, project_path: &str) -> PyResult<PyObject> {
    let result = init_project(py, project_path)?;
    let path = Path::new(project_path);
    let lib_dir = path.join("lib");
    fs::create_dir_all(lib_dir)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
      let result_dict = PyDict::new(py);
    result_dict.set_item("message", "RMM library project initialized.")?;
    Ok(result_dict.into_pyobject(py)?.into_any().unbind())
}

/// 清理构建目录
#[pyfunction]
fn clean_dist(project_path: &str) -> PyResult<()> {
    let path = Path::new(project_path);
    let dist_dir = path.join(".rmmp").join("dist");
    
    if dist_dir.exists() {
        fs::remove_dir_all(&dist_dir)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        println!("🧹 已清理构建输出目录: {}", dist_dir.display());
    } else {
        println!("ℹ️  构建输出目录不存在: {}", dist_dir.display());
    }
    
    Ok(())
}

// === 辅助函数 ===

/// 检测 Git 信息
fn detect_git_info(project_path: &Path) -> Option<serde_json::Value> {
    // 寻找 Git 根目录
    let mut current = project_path;
    let mut git_root = None;
    
    loop {
        if current.join(".git").exists() {
            git_root = Some(current);
            break;
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }
    
    let git_root = git_root?;
    let is_in_repo_root = git_root == project_path;
    
    // 解析 Git 配置
    let git_config_path = git_root.join(".git").join("config");
    if !git_config_path.exists() {
        return None;
    }
    
    let config_content = fs::read_to_string(&git_config_path).ok()?;
    let remote_info = parse_git_remote(&config_content, "origin")?;
    
    let mut git_info = serde_json::Map::new();
    git_info.insert("git_root".to_string(), serde_json::Value::String(git_root.to_string_lossy().to_string()));
    git_info.insert("is_in_repo_root".to_string(), serde_json::Value::Bool(is_in_repo_root));
    
    let mut remote_map = serde_json::Map::new();
    remote_map.insert("url".to_string(), serde_json::Value::String(remote_info.url.clone()));
    remote_map.insert("username".to_string(), serde_json::Value::String(remote_info.username.clone()));
    remote_map.insert("repo_name".to_string(), serde_json::Value::String(remote_info.repo_name.clone()));
    git_info.insert("remote_info".to_string(), serde_json::Value::Object(remote_map));
    
    Some(serde_json::Value::Object(git_info))
}

/// Git 远程仓库信息
#[derive(Debug)]
struct RemoteInfo {
    url: String,
    username: String,
    repo_name: String,
}

/// 解析 Git 配置中的远程仓库信息
fn parse_git_remote(config_content: &str, remote_name: &str) -> Option<RemoteInfo> {
    let section_pattern = format!(r#"\[remote "{}"\]"#, remote_name);
    let section_regex = Regex::new(&section_pattern).ok()?;
    
    let mut in_remote_section = false;
    let mut url = None;
    
    for line in config_content.lines() {
        let line = line.trim();
        
        if section_regex.is_match(line) {
            in_remote_section = true;
        } else if line.starts_with('[') && line.ends_with(']') {
            in_remote_section = false;
        } else if in_remote_section && line.starts_with("url = ") {
            url = Some(line.strip_prefix("url = ")?.to_string());
            break;
        }
    }
    
    let url = url?;
    let (username, repo_name) = extract_repo_info(&url)?;
    
    Some(RemoteInfo {
        url,
        username,
        repo_name,
    })
}

/// 从 Git URL 中提取用户名和仓库名
fn extract_repo_info(url: &str) -> Option<(String, String)> {
    // 支持多种 Git URL 格式
    let patterns = [
        r"https://github\.com/([^/]+)/([^/]+?)(?:\.git)?/?$",
        r"git@github\.com:([^/]+)/([^/]+?)(?:\.git)?/?$",
        r"ssh://git@github\.com/([^/]+)/([^/]+?)(?:\.git)?/?$",
    ];
    
    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(captures) = re.captures(url) {
                let username = captures.get(1)?.as_str().to_string();
                let repo_name = captures.get(2)?.as_str().to_string();
                return Some((username, repo_name));
            }
        }
    }
    
    None
}

/// 创建项目文件
fn create_project_files(project_path: &Path, project_name: &str, author_name: &str) -> Result<()> {
    // 使用 basic 模块的模板内容
    let readme_content = format!(r#"
# {project_name}

一个基于 RMM (Root Module Manager) 的模块项目。

## 功能特性

- 支持 Magisk、APatch、KernelSU
- 自动版本管理
- 构建输出优化
- GitHub 集成

## 安装方法

1. 下载最新的 release 文件
2. 通过 Magisk/APatch/KernelSU 安装模块
3. 重启设备

## 构建

```bash
# 构建模块
rmm build

# 发布到 GitHub
rmm publish
```

## 开发

```bash
# 安装开发依赖
uv tool install pyrmm

# 初始化项目
rmm init .

# 构建并测试
rmm build && rmm test
```

## 许可证

MIT License - 查看 [LICENSE](LICENSE) 文件了解详情。

## 作者

- {author_name}

---

使用 [RMM](https://github.com/LIghtJUNction/RootManage-Module-Model) 构建
"#, project_name = project_name, author_name = author_name);

    let now = chrono::Utc::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    
    let changelog_content = format!(r#"
# 更新日志

所有对该项目的重要更改都会记录在此文件中。

## [未发布]

### 新增
- 初始项目设置
- 基本模块结构

### 变更
- 无

### 修复
- 无

## [1.0.0] - {date_str}

### 新增
- 项目初始版本
- 基本功能实现

---

## 版本格式说明

- **[未发布]** - 即将发布的更改
- **[版本号]** - 已发布的版本及发布日期

### 更改类型

- **新增** - 新功能
- **变更** - 现有功能的更改
- **弃用** - 即将移除的功能
- **移除** - 已移除的功能
- **修复** - Bug 修复
- **安全** - 安全相关的修复
"#, date_str = date_str);

    let license_content = r#"
# LICENSES        
# ADD YOUR LICENSES HERE

# RMM Project License
MIT License

Copyright (c) 2025 LIghtJUNction

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"#;
    
    let files = [
        ("README.MD", readme_content),
        ("CHANGELOG.MD", changelog_content),
        ("LICENSE", license_content.to_string()),
    ];
    
    for (filename, content) in &files {
        let file_path = project_path.join(filename);
        if !file_path.exists() {
            fs::write(&file_path, content)
                .map_err(|e| anyhow!("创建文件 {} 失败: {}", filename, e))?;
            println!("✅ 创建文件: {}", filename);
        } else {
            println!("ℹ️  文件已存在，跳过: {}", filename);
        }
    }
    
    Ok(())
}

/// 转换 TOML 值为 Python 对象
fn toml_value_to_py_object(py: Python, value: &toml::Value) -> PyResult<PyObject> {
    match value {        toml::Value::String(s) => Ok(s.into_pyobject(py)?.into_any().unbind()),
        toml::Value::Integer(i) => Ok(i.into_pyobject(py)?.into_any().unbind()),
        toml::Value::Float(f) => Ok(f.into_pyobject(py)?.into_any().unbind()),
        toml::Value::Boolean(b) => Ok(b.into_pyobject(py)?.into_any().unbind()),
        toml::Value::Array(arr) => {
            let py_list = PyList::empty(py);
            for item in arr {
                py_list.append(toml_value_to_py_object(py, item)?)?;
            }            Ok(py_list.into_pyobject(py)?.into_any().unbind())
        },
        toml::Value::Table(table) => {
            let py_dict = PyDict::new(py);
            for (key, value) in table {
                py_dict.set_item(key, toml_value_to_py_object(py, value)?)?;
            }
            Ok(py_dict.into_pyobject(py)?.into_any().unbind())
        },
        toml::Value::Datetime(dt) => Ok(dt.to_string().into_pyobject(py)?.into_any().unbind()),
    }
}

/// 转换 Python 对象为 TOML 值
fn py_object_to_toml_value(py: Python, obj: &PyObject) -> PyResult<toml::Value> {
    if let Ok(s) = obj.extract::<String>(py) {
        Ok(toml::Value::String(s))
    } else if let Ok(i) = obj.extract::<i64>(py) {
        Ok(toml::Value::Integer(i))
    } else if let Ok(f) = obj.extract::<f64>(py) {
        Ok(toml::Value::Float(f))
    } else if let Ok(b) = obj.extract::<bool>(py) {
        Ok(toml::Value::Boolean(b))    } else if let Ok(list) = obj.downcast_bound::<PyList>(py) {
        let mut arr = Vec::new();
        for item in list.iter() {
            arr.push(py_object_to_toml_value(py, &item.into_pyobject(py)?.into_any().unbind())?);
        }
        Ok(toml::Value::Array(arr))
    } else if let Ok(dict) = obj.downcast_bound::<PyDict>(py) {
        let mut table = toml::value::Table::new();
        for (key, value) in dict.iter() {
            let key_str = key.extract::<String>()?;
            table.insert(key_str, py_object_to_toml_value(py, &value.into_pyobject(py)?.into_any().unbind())?);
        }
        Ok(toml::Value::Table(table))
    } else {        // 尝试转换为字符串
        Ok(toml::Value::String(obj.bind(py).str()?.to_cow()?.into_owned()))
    }
}

/// 转换 serde_json::Value 为 Python 对象
fn serde_json_to_py_object(py: Python, value: &serde_json::Value) -> PyResult<PyObject> {
    match value {
        serde_json::Value::Null => Ok(py.None()),        serde_json::Value::Bool(b) => Ok(b.into_pyobject(py)?.into_any().unbind()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_pyobject(py)?.into_any().unbind())
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_pyobject(py)?.into_any().unbind())
            } else {
                Ok(n.to_string().into_pyobject(py)?.into_any().unbind())
            }
        },
        serde_json::Value::String(s) => Ok(s.into_pyobject(py)?.into_any().unbind()),        serde_json::Value::Array(arr) => {
            let py_list = PyList::empty(py);
            for item in arr {
                py_list.append(serde_json_to_py_object(py, item)?)?;
            }
            Ok(py_list.into_pyobject(py)?.into_any().unbind())
        },
        serde_json::Value::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (key, value) in obj {
                py_dict.set_item(key, serde_json_to_py_object(py, value)?)?;
            }
            Ok(py_dict.into_pyobject(py)?.into_any().unbind())
        },
    }
}

/// Python 模块定义
#[pymodule]
fn project(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // 项目元类功能
    m.add_function(wrap_pyfunction!(get_projects_meta, m)?)?;
    m.add_function(wrap_pyfunction!(project_path, m)?)?;
    m.add_function(wrap_pyfunction!(project_info, m)?)?;
    m.add_function(wrap_pyfunction!(set_project_config, m)?)?;
    m.add_function(wrap_pyfunction!(delete_project_config, m)?)?;
    
    // 项目类功能
    m.add_function(wrap_pyfunction!(add_project, m)?)?;
    m.add_function(wrap_pyfunction!(is_valid_item, m)?)?;
    m.add_function(wrap_pyfunction!(get_sync_prompt, m)?)?;
    m.add_function(wrap_pyfunction!(is_rmmproject, m)?)?;
    m.add_function(wrap_pyfunction!(init_project, m)?)?;
    m.add_function(wrap_pyfunction!(sync_project, m)?)?;
    m.add_function(wrap_pyfunction!(init_basic, m)?)?;
    m.add_function(wrap_pyfunction!(init_library, m)?)?;
    m.add_function(wrap_pyfunction!(clean_dist, m)?)?;
    
    Ok(())
}
