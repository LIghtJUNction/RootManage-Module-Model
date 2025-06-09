use pyo3::prelude::*;
use pyo3::IntoPyObjectExt;
use std::env;
use std::fs as std_fs;
use std::path::PathBuf;

/// RMM File System class implemented in Rust
#[pyclass]
struct RmmFileSystem;

#[pymethods]
impl RmmFileSystem {
    #[classattr]
    fn ROOT(_py: Python) -> PyResult<PathBuf> {
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
    fn TMP(_py: Python) -> PyResult<PathBuf> {
        let root = Self::ROOT(_py)?;
        Ok(root.join("tmp"))
    }

    #[classattr]
    fn CACHE(_py: Python) -> PyResult<PathBuf> {
        let root = Self::ROOT(_py)?;
        Ok(root.join("cache"))
    }

    #[classattr]
    fn DATA(_py: Python) -> PyResult<PathBuf> {
        let root = Self::ROOT(_py)?;
        Ok(root.join("data"))
    }

    #[classattr]
    fn META(_py: Python) -> PyResult<PathBuf> {
        let root = Self::ROOT(_py)?;
        Ok(root.join("meta.toml"))
    }

    #[staticmethod]
    fn init(py: Python) -> PyResult<()> {
        let root = Self::ROOT(py)?;
        let tmp = Self::TMP(py)?;
        let cache = Self::CACHE(py)?;
        let data = Self::DATA(py)?;
        let meta = Self::META(py)?;        std_fs::create_dir_all(&root).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create ROOT directory: {}", e)))?;
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
