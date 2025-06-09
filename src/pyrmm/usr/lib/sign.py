# 签名用
"""
GPG 签名管理器
基于项目配置系统的 GPG 签名和验证功能
"""

from __future__ import annotations
from pathlib import Path
from typing import Any

try:
    import gnupg # type: ignore[import]
except ImportError:
    gnupg = None

from .config import Config
from .base import RmmBaseMeta, RmmBase
from ...__about__ import __PubKey__


class RmmSignMeta(RmmBaseMeta):
    """Meta class for RMM Sign Manager"""
    @property
    def META(cls) -> dict[str, Any]:
        """Get the GPG metadata."""
        meta: dict[str, str | dict[str, str]] = Config.META
        gpg_config = meta.get("gpg", {
            "key_id": "",
            "email": "",
            "passphrase": "",
            "gnupg_home": "",
            "auto_import": "true"
        })
        if isinstance(gpg_config, str):
            raise AttributeError(f"GPG 配置错误!： '{gpg_config}' 请检查：{Config.META}")
        return gpg_config
    
    @property
    def key_id(cls) -> str:
        """获取密钥 ID"""
        return cls.META.get("key_id", "")
    
    @property
    def email(cls) -> str:
        """获取邮箱"""
        return cls.META.get("email", "")
    
    @property
    def passphrase(cls) -> str:
        """获取密码"""
        return cls.META.get("passphrase", "")
    
    @property
    def gnupg_home(cls) -> str:
        """获取 GnuPG 主目录"""
        return cls.META.get("gnupg_home", "")
    
    @property
    def auto_import(cls) -> bool:
        """获取自动导入设置"""
        return cls.META.get("auto_import", "true").lower() == "true"
    
    def get_config_key(cls) -> str:
        """获取配置键名"""
        return "gpg"
    
    def get_reserved_key(cls) -> str:
        """获取保留关键字"""
        return "default"


class RmmSign(RmmBase, metaclass=RmmSignMeta):
    """RMM GPG 签名管理器 - 基于配置系统，无实例化"""
    
    
    @classmethod
    def _get_gpg_instance(cls, gpg_home: str | None = None) -> Any:
        """获取 GPG 实例"""
        
        if gpg_home is None:
            gpg_home = cls.gnupg_home or None

        if gnupg is None:
            raise ImportError("gnupg 模块未安装，请安装 python-gnupg 库以使用 GPG 功能。")
        
        return gnupg.GPG(gnupghome=gpg_home) 
    
    @classmethod
    def setup_key(
        cls,
        key_id: str,
        email: str,
        passphrase: str | None = None,
        gnupg_home: str | None = None,
        auto_import: bool = True
    ) -> dict[str, Any]:
        """
        设置 GPG 密钥配置
        
        Args:
            key_id: GPG 密钥 ID
            email: 邮箱地址
            passphrase: 私钥密码（可选）
            gnupg_home: GPG 主目录（可选）
            auto_import: 是否自动导入项目中的公钥
            
        Returns:
            设置结果
        """
        try:
            # 更新配置
            config = cls.META.copy()
            config.update({
                "key_id": key_id,
                "email": email,
                "passphrase": passphrase or "",
                "gnupg_home": gnupg_home or "",
                "auto_import": str(auto_import).lower()
            })
            
            # 保存配置
            setattr(Config, "gpg", config)
            
            # 自动导入项目公钥
            if auto_import and __PubKey__:
                cls.import_key(__PubKey__)
                # 可以选择性地记录导入结果，但不影响主要流程
                
            return {
                "success": True,
                "message": "GPG 密钥配置已设置",
                "key_id": key_id,
                "email": email,
                "auto_import": auto_import
            }
            
        except Exception as e:
            return {
                "success": False,
                "error": str(e)
            }
    
    @classmethod
    def import_key(cls, key_data: str, gpg_home: str | None = None) -> dict[str, Any]:
        """
        导入 GPG 密钥
        
        Args:
            key_data: GPG 密钥数据（ASCII 格式）
            gpg_home: GPG 主目录（可选）
            
        Returns:
            导入结果
        """
        try:
            gpg = cls._get_gpg_instance(gpg_home)
            
            # 导入密钥
            import_result = gpg.import_keys(key_data)
            
            if import_result.count == 0 and import_result.not_imported > 0:
                return {
                    "success": False,
                    "error": "密钥导入失败",
                    "details": import_result.results
                }
            
            return {
                "success": True,
                "imported": import_result.count,
                "not_imported": import_result.not_imported,
                "fingerprints": import_result.fingerprints,
                "results": import_result.results
            }
            
        except Exception as e:
            return {
                "success": False,
                "error": str(e)
            }
    
    @classmethod
    def export_key(cls, key_id: str | None = None, gpg_home: str | None = None) -> dict[str, Any]:
        """
        导出 GPG 公钥
        
        Args:
            key_id: 密钥 ID（可选，默认使用配置中的）
            gpg_home: GPG 主目录（可选）
            
        Returns:
            导出结果
        """
        try:
            if key_id is None:
                key_id = cls.key_id
                if not key_id:
                    return {
                        "success": False,
                        "error": "未指定密钥 ID，请先设置 GPG 配置"
                    }
            
            gpg = cls._get_gpg_instance(gpg_home)
            
            # 导出公钥
            public_key = gpg.export_keys(key_id)
            
            if not public_key:
                return {
                    "success": False,
                    "error": f"未找到密钥 ID: {key_id}"
                }
            
            return {
                "success": True,
                "key_id": key_id,
                "public_key": public_key
            }
            
        except Exception as e:
            return {
                "success": False,
                "error": str(e)
            }
    
    @classmethod
    def sign_file(
        cls,
        file_path: str | Path,
        key_id: str | None = None,
        passphrase: str | None = None,
        detach: bool = True,
        output_path: str | Path | None = None
    ) -> dict[str, Any]:
        """
        对文件进行 GPG 签名
        
        Args:
            file_path: 要签名的文件路径
            key_id: 用于签名的密钥 ID（可选，默认使用配置中的）
            passphrase: 私钥密码（可选，默认使用配置中的）
            detach: 是否创建分离签名（默认True）
            output_path: 输出路径（可选）
            
        Returns:
            签名结果
        """
        try:
            file_path = Path(file_path)
            
            if not file_path.exists():
                return {
                    "success": False,
                    "error": f"文件不存在: {file_path}"
                }
            
            # 获取配置
            if key_id is None:
                key_id = cls.key_id
                if not key_id:
                    return {
                        "success": False,
                        "error": "未指定密钥 ID，请先设置 GPG 配置"
                    }
            
            if passphrase is None:
                passphrase = cls.passphrase
            
            gpg = cls._get_gpg_instance()
            
            # 读取文件内容
            with open(file_path, 'rb') as f:
                file_data = f.read()
            
            # 进行签名
            signed_data = gpg.sign(
                file_data,
                keyid=key_id,
                passphrase=passphrase or None,
                detach=detach,
                binary=False  # 生成 ASCII 格式的签名
            )
            
            if signed_data.status != 'signature created':
                return {
                    "success": False,
                    "error": f"签名失败: {signed_data.status}",
                    "status": signed_data.status
                }
            
            # 确定输出路径
            if output_path is None:
                if detach:
                    output_path = file_path.with_suffix(file_path.suffix + '.sig')
                else:
                    output_path = file_path.with_suffix(file_path.suffix + '.signed')
            else:
                output_path = Path(output_path)
            
            # 保存签名
            with open(output_path, 'w') as f:
                f.write(str(signed_data))
            
            return {
                "success": True,
                "signed_file": str(file_path),
                "signature_file": str(output_path),
                "key_id": key_id,
                "fingerprint": signed_data.fingerprint,
                "detach": detach
            }
            
        except Exception as e:
            return {
                "success": False,
                "error": str(e)
            }
    
    @classmethod
    def verify_file(
        cls,
        file_path: str | Path,
        signature_path: str | Path | None = None
    ) -> dict[str, Any]:
        """
        验证文件签名
        
        Args:
            file_path: 原始文件路径
            signature_path: 签名文件路径（分离签名时需要）
            
        Returns:
            验证结果
        """
        try:
            file_path = Path(file_path)
            
            if not file_path.exists():
                return {
                    "success": False,
                    "error": f"文件不存在: {file_path}"
                }
            
            gpg = cls._get_gpg_instance()
            
            if signature_path:
                # 验证分离签名
                signature_path = Path(signature_path)
                if not signature_path.exists():
                    return {
                        "success": False,
                        "error": f"签名文件不存在: {signature_path}"
                    }
                
                with open(file_path, 'rb') as f:
                    file_data = f.read()
                with open(signature_path, 'r') as f:
                    sig_data = f.read()
                
                verified = gpg.verify_data(sig_data, file_data)
            else:
                # 验证内嵌签名
                with open(file_path, 'r') as f:
                    signed_data = f.read()
                
                verified = gpg.verify(signed_data)
            
            return {
                "success": True,
                "valid": verified.valid,
                "fingerprint": verified.fingerprint,
                "key_id": verified.key_id,
                "username": verified.username,
                "trust_level": verified.trust_level,
                "trust_text": verified.trust_text,
                "status": verified.status,
                "signature_path": str(signature_path) if signature_path else None
            }
            
        except Exception as e:
            return {
                "success": False,
                "error": str(e)
            }
    
    @classmethod
    def list_keys(cls, secret: bool = False) -> dict[str, Any]:
        """
        列出密钥
        
        Args:
            secret: 是否列出私钥（默认False，列出公钥）
            
        Returns:
            密钥列表
        """
        try:
            gpg = cls._get_gpg_instance()
            
            if secret:
                keys = gpg.list_keys(True)  # 私钥
            else:
                keys = gpg.list_keys()  # 公钥
            
            return {
                "success": True,
                "keys": keys,
                "count": len(keys)
            }
            
        except Exception as e:
            return {
                "success": False,
                "error": str(e)
            }
    
    @classmethod
    def get_status(cls) -> dict[str, Any]:
        """
        获取 GPG 配置状态
        
        Returns:
            状态信息
        """
        try:
            key_id = cls.key_id
            email = cls.email
            gnupg_home = cls.gnupg_home
            
            status = {
                "success": True,
                "configured": bool(key_id and email),
                "key_id": key_id,
                "email": email,
                "gnupg_home": gnupg_home or "默认",
                "gnupg_available": gnupg is not None
            }
            
            # 如果配置了密钥，检查密钥是否存在
            if key_id:
                try:
                    gpg = cls._get_gpg_instance()
                    keys = gpg.list_keys()
                    key_exists = any(key_id in key['keyid'] for key in keys)
                    status["key_exists"] = key_exists
                except:
                    status["key_exists"] = False
            
            return status
            
        except Exception as e:
            return {
                "success": False,
                "error": str(e)
            }
    
    @classmethod
    def is_valid_item(cls, item_name: str) -> bool:
        """检查是否是有效的 GPG 配置项"""
        valid_items = ["key_id", "email", "passphrase", "gnupg_home", "auto_import"]
        return item_name in valid_items
    
    @classmethod
    def get_sync_prompt(cls, item_name: str) -> str:
        """获取同步提示信息"""
        return f"GPG 配置项 '{item_name}'。"


# 便捷函数（类方法调用）
def setup_gpg_key(key_id: str, email: str, passphrase: str | None = None) -> dict[str, Any]:
    """设置 GPG 密钥配置的便捷函数"""
    return RmmSign.setup_key(key_id, email, passphrase)

def sign_file(file_path: str | Path, detach: bool = True) -> dict[str, Any]:
    """签名文件的便捷函数"""
    return RmmSign.sign_file(file_path, detach=detach)

def verify_file(file_path: str | Path, signature_path: str | Path | None = None) -> dict[str, Any]:
    """验证文件签名的便捷函数"""
    return RmmSign.verify_file(file_path, signature_path)

def get_gpg_status() -> dict[str, Any]:
    """获取 GPG 状态的便捷函数"""
    return RmmSign.get_status()