

import os
import time
import toml
import requests
from pathlib import Path
from datetime import datetime


class ProxyManager:
    API = "http://api.akams.cn/github"
    
    @classmethod
    def _get_cache_path(cls) -> Path:
        return Path(os.getenv("PYRMM_HOME", Path().home() / "data" / "adb" / ".rmm" / "cache" / "proxy.toml"))
    
    @classmethod
    def _fetch_from_api(cls) -> dict:
        """从API获取代理数据"""
        try:
            response = requests.get(cls.API, timeout=10)
            if response.status_code == 200:
                data = response.json()
                # 直接保存API返回的原始数据
                cls._save_cache(data)
                print(f"Proxy cache updated successfully. -- 写入{cls._get_cache_path()}")
                return data
            else:
                # 如果API失败，返回示例数据
                fallback_data = {
                    "code": 500,
                    "msg": "API request failed",
                    "data": [
                        {
                            "url": "https://ghproxy.cc",
                            "server": "cloudflare", 
                            "ip": "104.21.6.115",
                            "location": "  ",
                            "latency": 416,
                            "speed": 3.22
                        }
                    ],
                    "total": 1,
                    "update_time": datetime.now().strftime("%Y-%m-%d %H:%M:%S")
                }
                cls._save_cache(fallback_data)
                return fallback_data
        except Exception:
            # 网络错误时返回示例数据
            fallback_data = {
                "code": 500,
                "msg": "Network error",
                "data": [
                    {
                        "url": "https://ghproxy.cc",
                        "server": "cloudflare",
                        "ip": "104.21.6.115", 
                        "location": "  ",
                        "latency": 416,
                        "speed": 3.22
                    }
                ],
                "total": 1,
                "update_time": datetime.now().strftime("%Y-%m-%d %H:%M:%S")
            }
            cls._save_cache(fallback_data)
            return fallback_data
    
    @classmethod
    def _save_cache(cls, data: dict):
        """保存缓存"""
        cache_path = cls._get_cache_path()
        cache_path.parent.mkdir(parents=True, exist_ok=True)
        with open(cache_path, "w", encoding="utf-8") as f:
            toml.dump(data, f)
    
    @classmethod
    def _load_cache(cls) -> dict:
        """加载缓存"""
        cache_path = cls._get_cache_path()
        if cache_path.exists():
            with open(cache_path, "r", encoding="utf-8") as f:
                print(f"Proxy cache loaded successfully. -- 读取{cache_path}")
                return toml.load(f)
        return {}
    
    @classmethod
    def _is_cache_expired(cls, data: dict) -> bool:
        """检查缓存是否过期（10小时）"""
        update_time_str = data.get("update_time")
        if not update_time_str:
            return True
        
        try:
            update_time = datetime.strptime(update_time_str, "%Y-%m-%d %H:%M:%S").timestamp()
            return time.time() - update_time > 36000  # 10小时
        except (ValueError, TypeError):
            return True
    
    @classmethod
    def get_proxy_data(cls) -> dict:
        """获取代理数据（从缓存或API）"""
        cache_data = cls._load_cache()
        
        # 如果缓存存在且未过期，返回缓存
        if cache_data and not cls._is_cache_expired(cache_data):
            return cache_data
        
        # 否则从API获取
        return cls._fetch_from_api()
    
    @classmethod
    def get_fastest_proxy(cls) -> str:
        """获取最快的代理URL"""
        data = cls.get_proxy_data()
        
        # 检查数据结构和状态
        if data.get("code") == 200 and data.get("data"):
            return data["data"][0]["url"]
        elif data.get("data"):  # 即使code不是200，但有数据也返回
            return data["data"][0]["url"]
        
        return ""


if __name__ == "__main__":
    print(ProxyManager.get_fastest_proxy())