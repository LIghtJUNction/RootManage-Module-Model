use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;
use std::env;
use std::fs as std_fs;
use std::path::PathBuf;

/// RMM File System class implemented in Rust
#[pyclass]
struct RmmFileSystem;

#[pymethods]
impl RmmFileSystem {    #[classattr]
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

    #[classattr]
    #[pyo3(name = "TMP")]
    fn tmp(_py: Python) -> PyResult<PathBuf> {
        let root = Self::root(_py)?;
        Ok(root.join("tmp"))
    }

    #[classattr]
    #[pyo3(name = "CACHE")]
    fn cache(_py: Python) -> PyResult<PathBuf> {
        let root = Self::root(_py)?;
        Ok(root.join("cache"))
    }

    #[classattr]
    #[pyo3(name = "DATA")]
    fn data(_py: Python) -> PyResult<PathBuf> {
        let root = Self::root(_py)?;
        Ok(root.join("data"))
    }

    #[classattr]
    #[pyo3(name = "META")]
    fn meta(_py: Python) -> PyResult<PathBuf> {
        let root = Self::root(_py)?;
        Ok(root.join("meta.toml"))
    }    #[staticmethod]
    fn init(py: Python) -> PyResult<()> {
        let root = Self::root(py)?;
        let tmp = Self::tmp(py)?;
        let cache = Self::cache(py)?;
        let data = Self::data(py)?;
        let meta = Self::meta(py)?;std_fs::create_dir_all(&root).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create ROOT directory: {}", e)))?;
        std_fs::create_dir_all(&tmp).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create TMP directory: {}", e)))?;
        std_fs::create_dir_all(&cache).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create CACHE directory: {}", e)))?;
        std_fs::create_dir_all(&data).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create DATA directory: {}", e)))?;
        
        if !meta.exists() {
            std_fs::File::create(&meta).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create META file: {}", e)))?;
        }

        Ok(())
    }    #[staticmethod]
    #[pyo3(signature = (dir = "TMP"))]
    fn rm(py: Python, dir: &str) -> PyResult<()> {
        match dir {            "ROOT" => {
                let root = Self::ROOT(py)?;
                if root.exists() {
                    std_fs::remove_dir_all(&root).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to remove ROOT directory: {}", e)))?;
                }
            }
            "DATA" => {
                let data = Self::DATA(py)?;
                if data.exists() {
                    std_fs::remove_dir_all(&data).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to remove DATA directory: {}", e)))?;
                }
            }
            "TMP" => {
                let tmp = Self::TMP(py)?;
                if tmp.exists() {
                    std_fs::remove_dir_all(&tmp).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to remove TMP directory: {}", e)))?;
                }
            }
            "CACHE" => {
                let cache = Self::CACHE(py)?;
                if cache.exists() {
                    std_fs::remove_dir_all(&cache).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to remove CACHE directory: {}", e)))?;
                }
            }
            "META" => {
                let meta = Self::META(py)?;
                if meta.exists() {
                    std_fs::remove_file(&meta).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to remove META file: {}", e)))?;
                }
            }
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Unknown directory: {}", dir)));
            }
        }
        Ok(())
    }

    fn __getattr__(&self, py: Python, name: &str) -> PyResult<PyObject> {
        let meta_path = Self::META(py)?;
        
        if !meta_path.exists() {
            return Err(PyErr::new::<pyo3::exceptions::PyAttributeError, _>(format!("'{}' object has no attribute '{}'!!!", "RmmFileSystem", name)));
        }        let content = std_fs::read_to_string(&meta_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to read META file: {}", e)))?;
        
        let toml_value: toml::Value = toml::from_str(&content)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse TOML: {}", e)))?;        if let Some(projects) = toml_value.get("projects").and_then(|p| p.as_table()) {
            if let Some(project_path) = projects.get(name).and_then(|p| p.as_str()) {
                return Ok(PathBuf::from(project_path).into_py_any(py)?.into());
            }
        }

        Err(PyErr::new::<pyo3::exceptions::PyAttributeError, _>(format!("'{}' object has no attribute '{}'!!!", "RmmFileSystem", name)))
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn fs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RmmFileSystem>()?;
    Ok(())
}
