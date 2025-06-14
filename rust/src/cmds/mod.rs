pub mod rmmbox;
pub mod init;
pub mod build;
pub mod run;
pub mod sync;

pub use rmmbox::RmmBox;

use clap::Subcommand;

/// RMM 命令集合
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// 🚀 初始化新的模块项目
    Init {
        /// 项目ID（同时作为文件夹名）
        project_id: String,
    },    /// 🔨 构建模块项目
    Build {
        /// 项目路径（可选，默认为当前目录）
        #[arg(short, long)]
        project_path: Option<String>,
        
        /// 禁用 shellcheck 自动修复（默认启用自动修复）
        #[arg(long, default_value = "false")]
        no_auto_fix: bool,
        
        /// 运行 Rmake.toml 中定义的脚本
        #[arg(value_name = "SCRIPT")]
        script: Option<String>,    },
      /// 🚀 运行脚本命令
    Run {
        /// 项目路径（可选，默认为当前目录）
        #[arg(short, long)]
        project_path: Option<String>,
        
        /// 要执行的脚本名称（省略则显示所有可用脚本）
        #[arg(value_name = "SCRIPT")]
        script: Option<String>,
    },
    
    /// 🔄 同步项目元数据
    Sync {
        /// 特定项目名称（可选，默认同步所有项目）
        #[arg(value_name = "PROJECT")]
        project_name: Option<String>,
        
        /// 仅同步项目列表，跳过依赖同步
        #[arg(long, default_value = "false")]
        projects_only: bool,
        
        /// 指定搜索路径（可多个）
        #[arg(short, long, value_delimiter = ',')]
        search_paths: Option<Vec<String>>,
        
        /// 搜索最大深度
        #[arg(short, long, default_value = "3")]        
        max_depth: Option<usize>,
    },
    
    /// 📜 脚本管理命令
    Script {
        #[command(subcommand)]
        action: ScriptAction,
    },
    
    /// 显示版本信息
    Version,
    
    /// 未匹配命令，外部转发
    #[command(external_subcommand)]
    External(Vec<String>),
}

/// 脚本管理子命令
#[derive(Debug, Subcommand)]
pub enum ScriptAction {
    /// 🚀 初始化新的脚本项目
    Init {
        /// 脚本ID（同时作为文件夹名）
        script_id: String,
        
        /// 脚本类型
        #[arg(value_enum)]
        script_type: ScriptType,
        
        /// 作者名（可选）
        #[arg(short, long)]
        author: Option<String>,
        
        /// 作者邮箱（可选）
        #[arg(short, long)]
        email: Option<String>,
    },
    
    /// 📦 发布脚本到仓库
    Publish {
        /// 脚本路径（可选，默认为当前目录）
        #[arg(short, long)]
        script_path: Option<String>,
    },
    
    /// 🔍 搜索脚本
    Search {
        /// 搜索关键词
        query: String,
    },
    
    /// 📥 安装脚本
    Install {
        /// 脚本标识符（username/script_id）
        script_id: String,
    },
    
    /// 🗑️ 卸载脚本
    Uninstall {
        /// 脚本标识符（username/script_id）
        script_id: String,
    },
    
    /// 📋 列出已安装的脚本
    List,
    
    /// ▶️ 运行脚本
    Run {
        /// 脚本标识符（username/script_id）
        script_id: String,
        
        /// 传递给脚本的参数
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

/// 脚本类型枚举
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum ScriptType {
    /// 构建前脚本
    Prebuild,
    /// 构建脚本
    Build,
    /// 构建后脚本
    Postbuild,
    /// 发布脚本
    Publish,
    /// 更新脚本
    Update,
}

impl std::fmt::Display for ScriptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptType::Prebuild => write!(f, "prebuild"),
            ScriptType::Build => write!(f, "build"),
            ScriptType::Postbuild => write!(f, "postbuild"),
            ScriptType::Publish => write!(f, "publish"),
            ScriptType::Update => write!(f, "update"),
        }
    }
}

