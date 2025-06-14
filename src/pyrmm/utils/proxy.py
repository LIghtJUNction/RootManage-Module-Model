import json
import os
import requests
from pathlib import Path
from datetime import datetime, timedelta
from typing import Any

class ProxyManagerMeta(type):
    """
    Metaclass for ProxyManager to ensure singleton behavior.
    """

    @property
    def ROOT(cls):
        """
        Returns the root directory of the proxy manager.
        """
        return Path(os.getenv("RMM_ROOT", Path.home() / "data" / "adb" / ".rmm"))

    @property
    def CACHE(cls):
        """
        Returns the cache dictionary for storing proxies.
        """
        CACHE = cls.ROOT / "CACHE"
        if not CACHE.exists():
            CACHE.mkdir(parents=True, exist_ok=True)
        return CACHE

    @property
    def PROXY_CACHE_FILE(cls):
        """
        Returns the path to the proxy cache file.
        """
        return cls.CACHE / "github_proxy.json"


class ProxyManager(metaclass=ProxyManagerMeta):
    """
    GitHub代理管理器
    支持自动获取和缓存GitHub代理列表
    """
    
    API_URL = "https://api.akams.cn/github"
    CACHE_DURATION = timedelta(hours=10)  # 缓存10小时
    
    @classmethod
    def _load_cache(cls) -> dict[str, Any] | None:
        """
        加载缓存数据
        
        Returns:
            dict[str, object] | None: 缓存的代理数据，如果文件不存在或无效则返回None
        """
        try:
            if cls.PROXY_CACHE_FILE.exists():
                with open(cls.PROXY_CACHE_FILE, "r", encoding="utf-8") as f:
                    cache_data = json.load(f)
                    return cache_data
        except (json.JSONDecodeError, FileNotFoundError, KeyError) as e:
            print(f"⚠️  加载代理缓存失败: {e}")
        return None
    
    @classmethod
    def _save_cache(cls, data: dict[str, Any]) -> None:
        """
        保存数据到缓存
        
        Args:
            data: 要缓存的代理数据
        """
        try:
            with open(cls.PROXY_CACHE_FILE, "w", encoding="utf-8") as f:
                json.dump(data, f, ensure_ascii=False, indent=2)
            print(f"✅ 代理缓存已保存到: {cls.PROXY_CACHE_FILE}")
        except Exception as e:
            print(f"❌ 保存代理缓存失败: {e}")
    
    @classmethod
    def _is_cache_valid(cls, cache_data: dict[str, object]) -> bool:
        """
        检查缓存是否仍然有效
        
        Args:
            cache_data: 缓存的数据
            
        Returns:
            bool: 缓存是否有效
        """
        try:
            # 检查必要字段
            if not all(key in cache_data for key in ["cached_at", "data", "update_time"]):
                return False
            
            # 检查本地缓存时间（10小时内）
            cached_at = datetime.fromisoformat(cache_data["cached_at"])
            if datetime.now() - cached_at > cls.CACHE_DURATION:
                print("🕒 本地缓存已超过10小时，需要更新")
                return False
            
            # 检查API更新时间（比较服务器更新时间）
            api_update_time = cache_data.get("update_time", "")
            if api_update_time:
                try:
                    # 解析API返回的更新时间
                    api_time = datetime.strptime(api_update_time, "%Y-%m-%d %H:%M:%S")
                    cached_time = datetime.strptime(cache_data.get("api_update_time", ""), "%Y-%m-%d %H:%M:%S")
                    
                    if api_time > cached_time:
                        print(f"🔄 服务器数据已更新（{api_update_time}），需要刷新缓存")
                        return False
                except ValueError:
                    # 时间解析失败，认为缓存无效
                    return False
            
            print(f"✅ 缓存有效，最后更新时间: {cache_data.get('update_time', 'Unknown')}")
            return True
            
        except Exception as e:
            print(f"⚠️  检查缓存有效性时出错: {e}")
            return False
    
    @classmethod
    def _fetch_from_api(cls, timeout: int = 10) -> dict[str, object] | None:
        """
        从API获取最新的代理列表
        
        Args:
            timeout: 请求超时时间（秒）
            
        Returns:
            dict[str, object] | None: API返回的数据，失败则返回None
        """
        try:
            print(f"🌐 正在从API获取GitHub代理列表: {cls.API_URL}")
            
            response = requests.get(cls.API_URL, timeout=timeout)
            response.raise_for_status()
            
            api_data = response.json()
            
            # 验证API响应格式
            if api_data.get("code") != 200:
                print(f"❌ API返回错误: {api_data.get('msg', 'Unknown error')}")
                return None
            
            if "data" not in api_data:
                print("❌ API响应缺少data字段")
                return None
            
            print(f"✅ 成功获取 {api_data.get('total', 0)} 个代理")
            return api_data
            
        except requests.exceptions.Timeout:
            print(f"❌ API请求超时（超过{timeout}秒）")
        except requests.exceptions.RequestException as e:
            print(f"❌ API请求失败: {e}")
        except json.JSONDecodeError:
            print("❌ API响应不是有效的JSON格式")
        except Exception as e:
            print(f"❌ 获取代理列表时发生未知错误: {e}")
        
        return None
    
    @classmethod
    def get_proxy_list(cls, force_update: bool = False, timeout: int = 10) -> list[dict[str, object]]:
        """
        获取GitHub代理列表
        
        Args:
            force_update: 是否强制从API更新，忽略缓存
            timeout: API请求超时时间（秒）
            
        Returns:
            list[dict[str, object]]: 代理列表，包含url、server、ip、location、latency、speed等字段
        """
        # 1. 尝试加载缓存
        cache_data = None
        if not force_update:
            cache_data = cls._load_cache()
            
            # 2. 检查缓存有效性
            if cache_data and cls._is_cache_valid(cache_data):
                return cache_data["data"]
        
        # 3. 缓存无效或强制更新，从API获取
        api_data = cls._fetch_from_api(timeout)
        
        if api_data:
            # 4. 保存新的缓存
            cache_entry = {
                "cached_at": datetime.now().isoformat(),
                "api_update_time": api_data.get("update_time", ""),
                "update_time": api_data.get("update_time", ""),
                "total": api_data.get("total", 0),
                "data": api_data["data"]
            }
            cls._save_cache(cache_entry)
            return api_data["data"]
        
        # 5. API失败，尝试使用过期缓存
        if cache_data and "data" in cache_data:
            print("⚠️  API获取失败，使用过期缓存数据")
            return cache_data["data"]
        
        # 6. 完全失败，返回空列表
        print("❌ 无法获取代理列表，返回空列表")
        return []
    
    @classmethod
    def get_best_proxy(cls, force_update: bool = False) -> str | None:
        """
        获取最佳的GitHub代理URL（基于延迟和速度）
        
        Args:
            force_update: 是否强制更新代理列表
            
        Returns:
            Optional[str]: 最佳代理URL，如果没有可用代理则返回None
        """
        proxy_list = cls.get_proxy_list(force_update)
        
        if not proxy_list:
            return None
          # 根据延迟和速度排序（延迟越低越好，速度越高越好）
        def score_proxy(proxy: dict[str, object]) -> float:
            latency = int(proxy.get("latency", 9999) or 9999)
            speed = float(proxy.get("speed", 0) or 0)
            # 简单评分算法：速度/延迟，延迟为0时设为1避免除零
            return speed / max(latency, 1)
        
        best_proxy = max(proxy_list, key=score_proxy)
        print(f"🚀 选择最佳代理: {best_proxy['url']} (延迟: {best_proxy.get('latency', 'N/A')}ms, 速度: {best_proxy.get('speed', 'N/A')}MB/s)")
        
        return best_proxy["url"]
    
    @classmethod
    def clear_cache(cls) -> None:
        """
        清除代理缓存
        """
        try:
            if cls.PROXY_CACHE_FILE.exists():
                cls.PROXY_CACHE_FILE.unlink()
                print(f"✅ 已清除代理缓存: {cls.PROXY_CACHE_FILE}")
            else:
                print("ℹ️  缓存文件不存在，无需清除")
        except Exception as e:
            print(f"❌ 清除缓存失败: {e}")
    
    @classmethod
    def get_cache_info(cls) -> dict[str, object]:
        """
        获取缓存信息
        
        Returns:
            dict[str, object]: 缓存信息，包含文件路径、大小、更新时间等
        """
        info = {
            "cache_file": str(cls.PROXY_CACHE_FILE),
            "exists": cls.PROXY_CACHE_FILE.exists(),
            "size": 0,
            "cached_at": None,
            "update_time": None,
            "total_proxies": 0
        }
        # 检查缓存文件是否存在  
        if info["exists"]:
            try:
                info["size"] = cls.PROXY_CACHE_FILE.stat().st_size
                cache_data = cls._load_cache()
                if cache_data:
                    info["cached_at"] = cache_data.get("cached_at")
                    info["update_time"] = cache_data.get("update_time")
                    info["total_proxies"] = len(cache_data.get("data", []))
            except Exception as e:
                info["error"] = str(e)
        
        return info


# 便捷函数  
def get_github_proxies(force_update: bool = False) -> list[dict[str, object]]:
    """
    获取GitHub代理列表的便捷函数
    
    Args:
        force_update: 是否强制更新
        
    Returns:
        list[dict[str, object]]: 代理列表
    """
    return ProxyManager.get_proxy_list(force_update)


def get_best_github_proxy(force_update: bool = False) -> str | None:
    """
    获取最佳GitHub代理的便捷函数
    
    Args:
        force_update: 是否强制更新
        
    Returns:
        Optional[str]: 最佳代理URL
    """
    return ProxyManager.get_best_proxy(force_update)


# 示例用法和测试
if __name__ == "__main__":
    # 测试代理管理器
    print("🧪 测试GitHub代理管理器")
    print("=" * 50)
    
    # 获取缓存信息
    print("📋 缓存信息:")
    cache_info = ProxyManager.get_cache_info()
    for key, value in cache_info.items():
        print(f"   {key}: {value}")
    
    print("\n" + "=" * 50)
    
    # 获取代理列表
    print("📦 获取代理列表:")
    proxies = get_github_proxies()
    
    if proxies:
        print(f"✅ 找到 {len(proxies)} 个代理:")
        for i, proxy in enumerate(proxies, 1):
            print(f"   {i}. {proxy.get('url', 'N/A')} - "
                  f"延迟: {proxy.get('latency', 'N/A')}ms, "
                  f"速度: {proxy.get('speed', 'N/A')}MB/s")
        
        # 获取最佳代理
        print(f"\n🎯 最佳代理: {get_best_github_proxy()}")
    else:
        print("❌ 未找到可用代理")
