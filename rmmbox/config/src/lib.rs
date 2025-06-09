use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};
use pyo3::IntoPyObjectExt;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

/// Cache structure for Config metadata
#[derive(Debug)]
struct ConfigCache {
    data: Option<HashMap<String, toml::Value>>,
    last_modified: f64,
    current_modified: f64,
}

impl ConfigCache {
    fn new() -> Self {
        Self {
            data: None,
            last_modified: 0.0,
            current_modified: 0.0,
        }
    }
}

/// Configuration class for the RMM application
#[pyclass]
struct Config {
    cache: Mutex<ConfigCache>,
}

#[pymethods]
impl Config {    
    #[new]
    fn new() -> Self {
        Self {
            cache: Mutex::new(ConfigCache::new()),
        }
    }

    /// Save the metadata to the metadata file
    #[staticmethod]
    fn __save__(py: Python, meta_dict: &Bound<'_, PyDict>) -> PyResult<()> {
        // Import RmmFileSystem from fs module
        let fs_module = PyModule::import(py, "fs")?;
        let rmm_fs_class = fs_module.getattr("RmmFileSystem")?;
        let meta_path: PathBuf = rmm_fs_class.getattr("META")?.extract()?;
        
        // Convert PyDict to HashMap<String, toml::Value>
        let mut meta = HashMap::new();
        for (key, value) in meta_dict.iter() {
            let key_str: String = key.extract()?;
            let toml_value = Self::python_object_to_toml_static(py, &value.into())?;
            meta.insert(key_str, toml_value);
        }
        
        let toml_string = toml::to_string(&meta)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to serialize TOML: {}", e)))?;
        fs::write(&meta_path, toml_string)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to write META file: {}", e)))?;
        Ok(())
    }    /// Get the metadata with caching
    #[getter]
    #[allow(non_snake_case)]
    fn META(&self, py: Python) -> PyResult<PyObject> {
        // Import RmmFileSystem from fs module
        let fs_module = PyModule::import(py, "fs")?;
        let rmm_fs_class = fs_module.getattr("RmmFileSystem")?;
        let meta_path: PathBuf = rmm_fs_class.getattr("META")?.extract()?;
        
        let mut cache = self.cache.lock().unwrap();
        
        // Get current modification time
        cache.current_modified = if meta_path.exists() {
            meta_path.metadata()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to get metadata: {}", e)))?
                .modified()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to get modification time: {}", e)))?
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid timestamp: {}", e)))?
                .as_secs_f64()
        } else {
            0.0
        };

        // Return cached data if available and file hasn't changed
        if cache.data.is_some() && cache.last_modified == cache.current_modified {
            return self.hashmap_to_pydict(py, cache.data.as_ref().unwrap());
        }

        // Read and parse the file
        let content = if meta_path.exists() {
            fs::read_to_string(&meta_path)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to read META file: {}", e)))?
        } else {
            String::new()
        };

        let meta: HashMap<String, toml::Value> = if content.trim().is_empty() {
            HashMap::new()
        } else {
            toml::from_str(&content)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse TOML: {}", e)))?
        };

        // Update cache
        cache.data = Some(meta.clone());
        cache.last_modified = cache.current_modified;
        self.hashmap_to_pydict(py, &meta)
    }    /// Initialize the configuration
    #[classmethod]
    fn init(_cls: &Bound<'_, PyType>, py: Python) -> PyResult<()> {
        // Import and call RmmFileSystem.init() from fs module
        let fs_module = PyModule::import(py, "fs")?;
        let rmm_fs_class = fs_module.getattr("RmmFileSystem")?;
        rmm_fs_class.call_method0("init")?;
        
        let config = Config::new();
        let meta_obj = config.META(py)?;
        let meta_dict = meta_obj.downcast_bound::<PyDict>(py)?;
        
        // Create a new dict with the values we want to update
        let new_dict = PyDict::new(py);
        
        // Copy existing values
        for (key, value) in meta_dict.iter() {
            new_dict.set_item(key, value)?;
        }
        
        // Update with default values
        new_dict.set_item("username", "username")?;
        new_dict.set_item("email", "email")?;
        new_dict.set_item("version", "0.1.0")?; // You can update this to get from __version__
        
        // Create projects table if it doesn't exist
        if !new_dict.contains("projects")? {
            let projects_dict = PyDict::new(py);
            new_dict.set_item("projects", projects_dict)?;
        }
        
        Self::__save__(py, &new_dict)?;
        Ok(())
    }    /// Get an attribute from the metadata
    fn __getattr__(&self, py: Python, name: &str) -> PyResult<PyObject> {
        if name.starts_with('_') {
            return Err(PyErr::new::<pyo3::exceptions::PyAttributeError, _>(format!("'Config' object has no attribute '{}'", name)));
        }

        let meta_obj = self.META(py)?;
        let meta_dict = meta_obj.downcast_bound::<PyDict>(py)?;
        
        if let Ok(value) = meta_dict.get_item(name) {
            if let Some(val) = value {
                Ok(val.into())
            } else {
                // Get META path for error message
                let fs_module = PyModule::import(py, "fs")?;
                let rmm_fs_class = fs_module.getattr("RmmFileSystem")?;
                let meta_path: PathBuf = rmm_fs_class.getattr("META")?.extract()?;
                let keys: Vec<String> = meta_dict.keys().iter()
                    .map(|k| k.extract::<String>().unwrap_or_default())
                    .collect();
                Err(PyErr::new::<pyo3::exceptions::PyAttributeError, _>(
                    format!("找不到配置项： '{}' 请检查：{} , 当前配置项：{:?}", name, meta_path.display(), keys)
                ))
            }
        } else {
            // Get META path for error message
            let fs_module = PyModule::import(py, "fs")?;
            let rmm_fs_class = fs_module.getattr("RmmFileSystem")?;
            let meta_path: PathBuf = rmm_fs_class.getattr("META")?.extract()?;
            let keys: Vec<String> = meta_dict.keys().iter()
                .map(|k| k.extract::<String>().unwrap_or_default())
                .collect();
            Err(PyErr::new::<pyo3::exceptions::PyAttributeError, _>(
                format!("找不到配置项： '{}' 请检查：{} , 当前配置项：{:?}", name, meta_path.display(), keys)
            ))
        }
    }

    /// Set an attribute in the metadata
    fn __setattr__(&self, py: Python, name: &str, value: PyObject) -> PyResult<()> {
        // If it's an internal attribute, we can't handle it in Rust the same way as Python
        if name.starts_with('_') {
            return Err(PyErr::new::<pyo3::exceptions::PyAttributeError, _>("Cannot set internal attributes"));
        }

        let meta_obj = self.META(py)?;
        let meta_dict = meta_obj.downcast_bound::<PyDict>(py)?;
          // Create a new dict and copy existing values
        let new_dict = PyDict::new(py);
        for (key, val) in meta_dict.iter() {
            new_dict.set_item(key, val)?;
        }
        
        // Set the new value
        new_dict.set_item(name, value)?;
        
        Self::__save__(py, &new_dict)?;

        // Clear cache
        let mut cache = self.cache.lock().unwrap();
        cache.data = None;
        cache.last_modified = 0.0;        Ok(())
    }

    /// Delete an attribute from the metadata
    fn __delattr__(&self, py: Python, name: &str) -> PyResult<()> {
        if name.starts_with('_') {
            return Err(PyErr::new::<pyo3::exceptions::PyAttributeError, _>("Cannot delete internal attributes"));
        }

        let meta_obj = self.META(py)?;
        let meta_dict = meta_obj.downcast_bound::<PyDict>(py)?;
        
        if meta_dict.contains(name)? {            // Create a new dict without the deleted key
            let new_dict = PyDict::new(py);
            for (key, val) in meta_dict.iter() {
                let key_str: String = key.extract()?;
                if key_str != name {
                    new_dict.set_item(key, val)?;
                }
            }
            
            Self::__save__(py, &new_dict)?;
            
            // Clear cache
            let mut cache = self.cache.lock().unwrap();
            cache.data = None;
            cache.last_modified = 0.0;
            Ok(())
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyAttributeError, _>(format!("配置项 '{}' 不存在", name)))
        }
    }

    /// Return the list of attributes in the metadata
    fn __dir__(&self, py: Python) -> PyResult<Vec<String>> {
        let meta_obj = self.META(py)?;
        let meta_dict = meta_obj.downcast_bound::<PyDict>(py)?;
        
        let mut keys = Vec::new();
        for (key, _) in meta_dict.iter() {
            let key_str: String = key.extract()?;
            if !key_str.starts_with('_') && key_str != "init" {
                keys.push(key_str);
            }
        }
        Ok(keys)
    }
}

impl Config {    /// Helper function to convert HashMap to PyDict
    fn hashmap_to_pydict(&self, py: Python, map: &HashMap<String, toml::Value>) -> PyResult<PyObject> {
        let dict = PyDict::new(py);
        for (key, value) in map {
            let py_value = self.toml_value_to_python(py, value)?;
            dict.set_item(key, py_value)?;
        }
        Ok(dict.into())
    }

    /// Helper function to convert toml::Value to Python object
    fn toml_value_to_python(&self, py: Python, value: &toml::Value) -> PyResult<PyObject> {
        match value {
            toml::Value::String(s) => Ok(s.clone().into_py_any(py)?.into()),
            toml::Value::Integer(i) => Ok((*i).into_py_any(py)?.into()),
            toml::Value::Float(f) => Ok((*f).into_py_any(py)?.into()),
            toml::Value::Boolean(b) => Ok((*b).into_py_any(py)?.into()),
            toml::Value::Array(arr) => {
                let py_list: Vec<PyObject> = arr.iter()
                    .map(|v| self.toml_value_to_python(py, v))
                    .collect::<PyResult<Vec<_>>>()?;
                Ok(py_list.into_py_any(py)?.into())
            },            toml::Value::Table(table) => {
                let py_dict = PyDict::new(py);
                for (k, v) in table {
                    let py_value = self.toml_value_to_python(py, v)?;
                    py_dict.set_item(k, py_value)?;
                }
                Ok(py_dict.into())
            },
            _ => Ok(py.None()),
        }
    }

    /// Helper function to convert Python object to toml::Value (static version)
    fn python_object_to_toml_static(py: Python, obj: &PyObject) -> PyResult<toml::Value> {
        if let Ok(s) = obj.extract::<String>(py) {
            Ok(toml::Value::String(s))
        } else if let Ok(i) = obj.extract::<i64>(py) {
            Ok(toml::Value::Integer(i))
        } else if let Ok(f) = obj.extract::<f64>(py) {
            Ok(toml::Value::Float(f))
        } else if let Ok(b) = obj.extract::<bool>(py) {
            Ok(toml::Value::Boolean(b))        } else if let Ok(dict) = obj.downcast_bound::<PyDict>(py) {
            let mut table = toml::map::Map::new();
            for (k, v) in dict.iter() {
                let key: String = k.extract()?;
                let value = Self::python_object_to_toml_static(py, &v.into())?;
                table.insert(key, value);
            }
            Ok(toml::Value::Table(table))
        } else if let Ok(list) = obj.extract::<Vec<PyObject>>(py) {
            let mut array = Vec::new();
            for item in list {
                array.push(Self::python_object_to_toml_static(py, &item)?);
            }
            Ok(toml::Value::Array(array))
        } else {
            // For other types, convert to string
            let repr = obj.call_method0(py, "__str__")?;
            let s = repr.extract::<String>(py)?;
            Ok(toml::Value::String(s))
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn config(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Config>()?;
    Ok(())
}
