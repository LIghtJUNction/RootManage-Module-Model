use crate::core::rmm_core::*;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString};
use std::collections::HashMap;
use std::path::Path;

/// Python 包装器 - RmmCore
#[pyclass(name = "RmmCore")]
pub struct PyRmmCore {
    inner: RmmCore,
}

#[pymethods]
impl PyRmmCore {
    #[new]
    fn new() -> Self {
        Self {
            inner: RmmCore::new(),
        }
    }

    /// 获取 RMM_ROOT 路径
    fn get_rmm_root(&self) -> String {
        self.inner.get_rmm_root().to_string_lossy().to_string()
    }

    /// 获取 meta 配置
    fn get_meta_config(&self, py: Python) -> PyResult<PyObject> {
        match self.inner.get_meta_config() {
            Ok(meta) => {
                let dict = PyDict::new(py);
                dict.set_item("email", meta.email)?;
                dict.set_item("username", meta.username)?;
                dict.set_item("version", meta.version)?;
                
                let projects_dict = PyDict::new(py);
                for (name, path) in meta.projects {
                    projects_dict.set_item(name, path)?;
                }
                dict.set_item("projects", projects_dict)?;
                
                Ok(dict.into())
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                e.to_string(),
            )),
        }
    }

    /// 更新 meta 配置
    fn update_meta_config(
        &self,
        email: String,
        username: String,
        version: String,
        projects: HashMap<String, String>,
    ) -> PyResult<()> {
        let meta = MetaConfig {
            email,
            username,
            version,
            projects,
        };
        
        self.inner.update_meta_config(&meta).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
        })
    }

    /// 根据项目名获取路径
    fn get_project_path(&self, project_name: &str) -> PyResult<Option<String>> {
        match self.inner.get_project_path(project_name) {
            Ok(Some(path)) => Ok(Some(path.to_string_lossy().to_string())),
            Ok(None) => Ok(None),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                e.to_string(),
            )),
        }
    }

    /// 检查项目有效性
    fn check_projects_validity(&self, py: Python) -> PyResult<PyObject> {
        match self.inner.check_projects_validity() {
            Ok(validity) => {
                let dict = PyDict::new(py);
                for (name, is_valid) in validity {
                    dict.set_item(name, is_valid)?;
                }
                Ok(dict.into())
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                e.to_string(),
            )),
        }
    }    /// 扫描项目 - 简化版本
    fn scan_projects(
        &self,
        py: Python,
        scan_path: String,
        max_depth: Option<usize>,
    ) -> PyResult<PyObject> {
        // 🔧 修复：添加路径验证，防止无效路径导致崩溃
        let path = Path::new(&scan_path);
        
        // 验证路径是否存在且可访问
        if !path.exists() {
            return Err(PyErr::new::<pyo3::exceptions::PyFileNotFoundError, _>(
                format!("扫描路径不存在: {}", scan_path),
            ));
        }
        
        if !path.is_dir() {
            return Err(PyErr::new::<pyo3::exceptions::PyNotADirectoryError, _>(
                format!("扫描路径不是目录: {}", scan_path),
            ));
        }
        
        // 验证最大深度参数
        if let Some(depth) = max_depth {
            if depth > 20 {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "最大深度不能超过20层，以防止性能问题",
                ));
            }
        }
        
        match self.inner.scan_projects(path, max_depth) {            
            Ok(results) => {
                let list = PyList::empty(py);
                for result in results {
                    let dict = PyDict::new(py);
                    dict.set_item("name", result.name)?;
                    dict.set_item("path", result.path.to_string_lossy().to_string())?;
                    dict.set_item("is_valid", result.is_valid)?;
                    list.append(dict)?;
                }
                Ok(list.into())
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                e.to_string(),
            )),
        }
    }

    /// 同步项目
    fn sync_projects(
        &self,
        scan_paths: Vec<String>,
        max_depth: Option<usize>,
    ) -> PyResult<()> {
        let paths: Vec<&Path> = scan_paths.iter().map(|p| Path::new(p)).collect();
        self.inner.sync_projects(&paths, max_depth).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
        })
    }

    /// 获取项目配置
    fn get_project_config(&self, py: Python, project_path: String) -> PyResult<PyObject> {
        let path = Path::new(&project_path);
        match self.inner.get_project_config(path) {
            Ok(project) => {
                let dict = PyDict::new(py);
                
                // Project info
                let project_info = PyDict::new(py);
                project_info.set_item("id", project.project.id)?;
                project_info.set_item("description", project.project.description)?;
                project_info.set_item("readme", project.project.readme)?;
                project_info.set_item("changelog", project.project.changelog)?;
                project_info.set_item("license", project.project.license)?;
                project_info.set_item("dependencies", project.project.dependencies)?;
                
                dict.set_item("project", project_info)?;                // Authors - 简化处理
                let authors_list_str = serde_json::to_string(&project.authors).unwrap_or_else(|_| "[]".to_string());
                dict.set_item("authors", authors_list_str)?;

                Ok(dict.into())
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                e.to_string(),
            )),
        }
    }

    /// 获取 module.prop
    fn get_module_prop(&self, py: Python, project_path: String) -> PyResult<PyObject> {
        let path = Path::new(&project_path);
        match self.inner.get_module_prop(path) {
            Ok(prop) => {
                let dict = PyDict::new(py);
                dict.set_item("id", prop.id)?;
                dict.set_item("name", prop.name)?;
                dict.set_item("version", prop.version)?;
                dict.set_item("versionCode", prop.version_code)?;
                dict.set_item("author", prop.author)?;
                dict.set_item("description", prop.description)?;
                dict.set_item("updateJson", prop.update_json)?;
                Ok(dict.into())
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                e.to_string(),
            )),
        }
    }

    /// 更新 module.prop
    fn update_module_prop(
        &self,
        project_path: String,
        id: String,
        name: String,
        version: String,
        version_code: String,
        author: String,
        description: String,
        update_json: String,
    ) -> PyResult<()> {
        let path = Path::new(&project_path);
        let prop = ModuleProp {
            id,
            name,
            version,
            version_code,
            author,
            description,
            update_json,
        };
        
        self.inner.update_module_prop(path, &prop).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
        })
    }

    /// 获取 Git 信息
    fn get_git_info(&self, py: Python, project_path: String) -> PyResult<PyObject> {
        let path = Path::new(&project_path);
        match self.inner.get_git_info(path) {
            Ok(git_info) => {
                let dict = PyDict::new(py);
                dict.set_item("repo_root", git_info.repo_root.to_string_lossy().to_string())?;
                dict.set_item("relative_path", git_info.relative_path.to_string_lossy().to_string())?;
                dict.set_item("branch", git_info.branch)?;
                dict.set_item("remote_url", git_info.remote_url)?;
                dict.set_item("has_uncommitted_changes", git_info.has_uncommitted_changes)?;
                dict.set_item("last_commit_hash", git_info.last_commit_hash)?;
                dict.set_item("last_commit_message", git_info.last_commit_message)?;
                Ok(dict.into())
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                e.to_string(),
            )),
        }
    }

    /// 移除项目
    fn remove_project_from_meta(&self, project_name: String) -> PyResult<bool> {
        self.inner.remove_project_from_meta(&project_name).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
        })
    }    /// 移除多个项目
    fn remove_projects_from_meta(&self, py: Python, project_names: Vec<String>) -> PyResult<PyObject> {
        let names: Vec<&str> = project_names.iter().map(|s| s.as_str()).collect();
        match self.inner.remove_projects_from_meta(&names) {
            Ok(removed) => {
                let json_str = serde_json::to_string(&removed).unwrap_or_else(|_| "[]".to_string());
                Ok(PyString::new(py, &json_str).into())
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                e.to_string(),
            )),
        }
    }    /// 移除无效项目
    fn remove_invalid_projects(&self, py: Python) -> PyResult<PyObject> {
        match self.inner.remove_invalid_projects() {
            Ok(removed) => {
                let json_str = serde_json::to_string(&removed).unwrap_or_else(|_| "[]".to_string());
                Ok(PyString::new(py, &json_str).into())
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                e.to_string(),
            )),
        }
    }

    /// 获取缓存统计
    fn get_cache_stats(&self, py: Python) -> PyResult<PyObject> {
        let (meta_cached, project_count) = self.inner.get_cache_stats();
        let dict = PyDict::new(py);
        dict.set_item("meta_cached", meta_cached)?;
        dict.set_item("project_count", project_count)?;
        Ok(dict.into())
    }

    /// 清理所有缓存
    fn clear_all_cache(&self) -> PyResult<()> {
        self.inner.clear_all_cache();
        Ok(())
    }    /// 清理过期缓存
    fn cleanup_expired_cache(&self) -> PyResult<()> {
        self.inner.cleanup_expired_cache();
        Ok(())
    }

    /// 创建默认的 meta 配置
    fn create_default_meta(&self, py: Python, email: String, username: String, version: String) -> PyResult<PyObject> {
        let meta = MetaConfig {
            email,
            username,
            version,
            projects: HashMap::new(),
        };
        
        let dict = PyDict::new(py);
        dict.set_item("email", meta.email)?;
        dict.set_item("username", meta.username)?;
        dict.set_item("version", meta.version)?;
        
        let projects_dict = PyDict::new(py);
        dict.set_item("projects", projects_dict)?;
        
        Ok(dict.into())
    }    /// 创建默认的项目配置
    fn create_default_project(&self, py: Python, project_id: String, username: String, email: String) -> PyResult<PyObject> {
        let project_config = RmmProject {
            project: ProjectInfo {
                id: project_id.clone(),
                description: format!("{} - A Magisk Module", project_id),
                readme: "README.md".to_string(),
                changelog: "CHANGELOG.md".to_string(),
                license: "LICENSE".to_string(),
                dependencies: Vec::new(),
                scripts: None,
            },
            authors: vec![Author {
                name: username,
                email,
            }],
            urls: None,
            build_system: None,
            tool: None,
        };
        
        let dict = PyDict::new(py);
        
        // Project info
        let project_info = PyDict::new(py);
        project_info.set_item("id", project_config.project.id)?;
        project_info.set_item("description", project_config.project.description)?;
        project_info.set_item("readme", project_config.project.readme)?;
        project_info.set_item("changelog", project_config.project.changelog)?;
        project_info.set_item("license", project_config.project.license)?;
        project_info.set_item("dependencies", project_config.project.dependencies)?;
        
        dict.set_item("project", project_info)?;
        
        // Authors
        let authors_list = PyList::empty(py);
        for author in project_config.authors {
            let author_dict = PyDict::new(py);
            author_dict.set_item("name", author.name)?;
            author_dict.set_item("email", author.email)?;
            authors_list.append(author_dict)?;
        }
        dict.set_item("authors", authors_list)?;
        
        Ok(dict.into())
    }

    /// 创建默认的 module.prop 配置
    fn create_default_module_prop(&self, py: Python, module_id: String, username: String) -> PyResult<PyObject> {
        let prop = ModuleProp {
            id: module_id.clone(),
            name: format!("{} Module", module_id),
            version: "v1.0.0".to_string(),
            version_code: "1000000".to_string(),
            author: username,
            description: format!("A Magisk module: {}", module_id),
            update_json: "https://example.com/update.json".to_string(),
        };
        
        let dict = PyDict::new(py);
        dict.set_item("id", prop.id)?;
        dict.set_item("name", prop.name)?;
        dict.set_item("version", prop.version)?;
        dict.set_item("versionCode", prop.version_code)?;
        dict.set_item("author", prop.author)?;
        dict.set_item("description", prop.description)?;
        dict.set_item("updateJson", prop.update_json)?;
        
        Ok(dict.into())
    }

    /// 创建默认的 Rmake 配置
    fn create_default_rmake(&self, py: Python) -> PyResult<PyObject> {
        let config = RmakeConfig {
            build: BuildConfig {
                include: vec!["rmm".to_string()],
                exclude: vec![".git".to_string(), ".rmmp".to_string(), "*.tmp".to_string()],
                prebuild: vec!["echo 'Starting build'".to_string()],
                build: vec!["rmm".to_string()],
                postbuild: vec!["echo 'Build completed'".to_string()],
                src: Some(SrcConfig {
                    include: Vec::new(),
                    exclude: Vec::new(),
                }),
                scripts: Some(HashMap::new()),
            },
        };
        
        let dict = PyDict::new(py);
        
        // Build config
        let build_dict = PyDict::new(py);
        build_dict.set_item("include", config.build.include)?;
        build_dict.set_item("exclude", config.build.exclude)?;
        build_dict.set_item("prebuild", config.build.prebuild)?;
        build_dict.set_item("build", config.build.build)?;
        build_dict.set_item("postbuild", config.build.postbuild)?;
        
        // Src config
        let src_dict = PyDict::new(py);
        if let Some(src) = config.build.src {
            src_dict.set_item("include", src.include)?;
            src_dict.set_item("exclude", src.exclude)?;
        }
        build_dict.set_item("src", src_dict)?;
        
        // Scripts
        let scripts_dict = PyDict::new(py);
        scripts_dict.set_item("release", "rmm build --release")?;
        scripts_dict.set_item("debug", "rmm build --debug")?;
        build_dict.set_item("scripts", scripts_dict)?;
        
        dict.set_item("build", build_dict)?;
        
        Ok(dict.into())
    }

    /// 获取 rmake 配置
    fn get_rmake_config(&self, py: Python, project_path: String) -> PyResult<PyObject> {
        let path = Path::new(&project_path);
        match self.inner.get_rmake_config(path) {
            Ok(config) => {
                let dict = PyDict::new(py);
                
                // Build config
                let build_dict = PyDict::new(py);
                build_dict.set_item("include", config.build.include)?;
                build_dict.set_item("exclude", config.build.exclude)?;
                build_dict.set_item("prebuild", config.build.prebuild)?;
                build_dict.set_item("build", config.build.build)?;
                build_dict.set_item("postbuild", config.build.postbuild)?;
                
                // Src config
                let src_dict = PyDict::new(py);
                if let Some(src) = config.build.src {
                    src_dict.set_item("include", src.include)?;
                    src_dict.set_item("exclude", src.exclude)?;
                }
                build_dict.set_item("src", src_dict)?;
                
                // Scripts
                let scripts_dict = PyDict::new(py);
                if let Some(scripts) = config.build.scripts {
                    for (key, value) in scripts {
                        scripts_dict.set_item(key, value)?;
                    }
                }
                build_dict.set_item("scripts", scripts_dict)?;
                
                dict.set_item("build", build_dict)?;
                
                Ok(dict.into())
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                e.to_string(),
            )),
        }
    }

    /// 更新 meta 配置（从字典）
    fn update_meta_config_from_dict(&self, config_dict: &Bound<'_, PyDict>) -> PyResult<()> {
        let email: String = config_dict.get_item("email")?.unwrap().extract()?;
        let username: String = config_dict.get_item("username")?.unwrap().extract()?;
        let version: String = config_dict.get_item("version")?.unwrap().extract()?;
        
        let projects_item = config_dict.get_item("projects")?.unwrap();
        let projects_dict = projects_item.downcast::<PyDict>()?;
        let mut projects = HashMap::new();
        
        for (key, value) in projects_dict.iter() {
            let project_name: String = key.extract()?;
            let project_path: String = value.extract()?;
            projects.insert(project_name, project_path);
        }
        
        let meta = MetaConfig {
            email,
            username,
            version,
            projects,
        };
        
        self.inner.update_meta_config(&meta).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
        })
    }

    /// 获取 meta 配置中的特定值
    fn get_meta_value(&self, py: Python, key: String) -> PyResult<PyObject> {
        match self.inner.get_meta_config() {
            Ok(meta) => {
                match key.as_str() {
                    "email" => Ok(PyString::new(py, &meta.email).into()),
                    "username" => Ok(PyString::new(py, &meta.username).into()),
                    "version" => Ok(PyString::new(py, &meta.version).into()),
                    "projects" => {
                        let projects_dict = PyDict::new(py);
                        for (name, path) in meta.projects {
                            projects_dict.set_item(name, path)?;
                        }
                        Ok(projects_dict.into())
                    }
                    _ => Ok(py.None()),
                }
            }
            Err(_) => Ok(py.None()),
        }
    }}


