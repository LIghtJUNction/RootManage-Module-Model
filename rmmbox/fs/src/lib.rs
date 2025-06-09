use pyo3::prelude::*;
use pyo3::types::PyString;
use std::env;
use std::fs as std_fs;
use std::path::PathBuf;

/// RMM File System class implemented in Rust
#[pyclass]
struct RmmFileSystem;

#[pymethods]
impl RmmFileSystem {
    #[new]
    fn new() -> Self {
        Self
    }
    
    #[staticmethod]
    #[pyo3(name = "ROOT")]
    fn root(_py: Python) -> PyResult<PathBuf> {
        let root = match env::var("RMM_ROOT") {
            Ok(path) => PathBuf::from(path),
            Err(_) => {
                let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
                home.join("data").join("adb").join(".rmm")
            }
        };
        Ok(root.canonicalize().unwrap_or(root))
    }

    #[staticmethod]
    #[pyo3(name = "TMP")]
    fn tmp(_py: Python) -> PyResult<PathBuf> {
        let root = Self::root(_py)?;
        Ok(root.join("tmp"))
    }

    #[staticmethod]
    #[pyo3(name = "CACHE")]
    fn cache(_py: Python) -> PyResult<PathBuf> {
        let root = Self::root(_py)?;
        Ok(root.join("cache"))
    }    #[staticmethod]
    #[pyo3(name = "DATA")]
    fn data(_py: Python) -> PyResult<PathBuf> {
        let root = Self::root(_py)?;
        Ok(root.join("data"))
    }    

    #[staticmethod]
    #[pyo3(name = "BIN")]
    fn bin(_py: Python) -> PyResult<PathBuf> {
        let root = Self::root(_py)?;
        Ok(root.join("bin"))
    }
    
    #[staticmethod]
    #[pyo3(name = "META")]
    fn meta(_py: Python) -> PyResult<PathBuf> {
        let root = Self::root(_py)?;
        Ok(root.join("meta.toml"))
    }

    #[staticmethod]
     fn init(py: Python) -> PyResult<()> {
        let root = Self::root(py)?;
        let tmp = Self::tmp(py)?;
        let cache = Self::cache(py)?;
        let data = Self::data(py)?;
        let bin = Self::bin(py)?;
        let meta = Self::meta(py)?;

        std_fs::create_dir_all(&root).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create ROOT directory: {}", e)))?;
        std_fs::create_dir_all(&tmp).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create TMP directory: {}", e)))?;
        std_fs::create_dir_all(&cache).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create CACHE directory: {}", e)))?;
        std_fs::create_dir_all(&data).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create DATA directory: {}", e)))?;
        std_fs::create_dir_all(&bin).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create BIN directory: {}", e)))?;
        
        if !meta.exists() {
            std_fs::File::create(&meta).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create META file: {}", e)))?;
        }

        Ok(())
    }

    #[staticmethod]
    #[pyo3(signature = (dir = "TMP"))]
    fn rm(py: Python, dir: &str) -> PyResult<()> {
        match dir {            
                        "ROOT" => {
                let root = Self::root(py)?;
                if root.exists() {
                    std_fs::remove_dir_all(&root).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to remove ROOT directory: {}", e)))?;
                }
            }            
                        "DATA" => {
                let data = Self::data(py)?;
                if data.exists() {
                    std_fs::remove_dir_all(&data).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to remove DATA directory: {}", e)))?;
                }
            }            "TMP" => {
                let tmp = Self::tmp(py)?;
                if tmp.exists() {
                    std_fs::remove_dir_all(&tmp).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to remove TMP directory: {}", e)))?;
                }
            }            "CACHE" => {
                let cache = Self::cache(py)?;
                if cache.exists() {
                    std_fs::remove_dir_all(&cache).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to remove CACHE directory: {}", e)))?;
                }
            }
            "BIN" => {
                let bin = Self::bin(py)?;
                if bin.exists() {
                    std_fs::remove_dir_all(&bin).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to remove BIN directory: {}", e)))?;
                }
            }
            "META" => {
                let meta = Self::meta(py)?;
                if meta.exists() {
                    std_fs::remove_file(&meta).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to remove META file: {}", e)))?;
                }
            }
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Unknown directory: {}", dir)));
            }
        }
        Ok(())
    }    fn __getattr__(&self, py: Python, name: &str) -> PyResult<PyObject> {
        let meta_path = Self::meta(py)?;
        
        if !meta_path.exists() {
            return Err(PyErr::new::<pyo3::exceptions::PyAttributeError, _>(format!("'{}' object has no attribute '{}'!!!", "RmmFileSystem", name)));
        }        let content = std_fs::read_to_string(&meta_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to read META file: {}", e)))?;
        
        let toml_value: toml::Value = toml::from_str(&content)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse TOML: {}", e)))?;        if let Some(projects) = toml_value.get("projects").and_then(|p| p.as_table()) {
            if let Some(project_path) = projects.get(name).and_then(|p| p.as_str()) {
                return Ok(PyString::new(py, project_path).into_any().unbind());
            }
        }

        Err(PyErr::new::<pyo3::exceptions::PyAttributeError, _>(format!("'{}' object has no attribute '{}'!!!", "RmmFileSystem", name)))
    }
}

/// 确保目录存在
#[pyfunction]
fn ensure_dir(path: &str) -> PyResult<()> {
    let path = PathBuf::from(path);
    std_fs::create_dir_all(&path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create directory: {}", e)))?;
    Ok(())
}

/// A Python module implemented in Rust.
#[pymodule]
fn fs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RmmFileSystem>()?;
    m.add_function(wrap_pyfunction!(ensure_dir, m)?)?;
    
    // 导出常量 - 使用错误处理避免导出失败
    let py = m.py();
    
    if let Ok(root) = RmmFileSystem::root(py) {
        m.add("ROOT", root.to_string_lossy().to_string())?;
    }
    
    if let Ok(tmp) = RmmFileSystem::tmp(py) {
        m.add("TMP", tmp.to_string_lossy().to_string())?;
    }
    
    if let Ok(cache) = RmmFileSystem::cache(py) {
        m.add("CACHE", cache.to_string_lossy().to_string())?;
    }
    
    if let Ok(data) = RmmFileSystem::data(py) {
        m.add("DATA", data.to_string_lossy().to_string())?;
    }
    
    if let Ok(bin) = RmmFileSystem::bin(py) {
        m.add("BIN", bin.to_string_lossy().to_string())?;
    }
    
    if let Ok(meta) = RmmFileSystem::meta(py) {
        m.add("META", meta.to_string_lossy().to_string())?;
    }
    
    Ok(())
}
