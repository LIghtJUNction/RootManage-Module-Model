"""GitHub代理管理器 - 获取和解析GitHub代理节点"""
import json
import requests
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Any
from urllib.parse import urlparse

@dataclass
class ProxyNode:
    """代理节点数据类"""
    url: str
    speed: float
    
    def get_domain_main_part(self) -> str:
        """获取域名的主体部分，去除前缀如github."""
        parsed = urlparse(self.url)
        domain = parsed.netloc
        # 移除常见的github相关前缀
        if domain.startswith('github.'):
            return domain[7:]  # 移除 'github.'
        elif domain.startswith('gh.'):
            return domain[3:]   # 移除 'gh.'
        elif domain.startswith('ghproxy.'):
            return domain[8:]   # 移除 'ghproxy.'
        elif domain.startswith('ghp.'):
            return domain[4:]   # 移除 'ghp.'
        elif 'github' in domain and '.' in domain:
            # 对于包含github的域名，取最后的主域名部分
            parts = domain.split('.')
            return '.'.join(parts[-2:]) if len(parts) >= 2 else domain
        return domain

class ProxyManager:
    """GitHub代理管理器"""    
    API_URL = "https://api.akams.cn/github"
    @classmethod
    def get_proxies(cls) -> list[ProxyNode]:
        """
        获取GitHub代理节点列表，按速度排序
        
        Returns:
            list[ProxyNode]: 按速度降序排列的代理节点列表
            
        Raises:
            requests.RequestException: 网络请求失败
            json.JSONDecodeError: JSON解析失败
            ValueError: 数据格式错误
        """
        # 发送HTTP请求
        response: requests.Response = requests.get(cls.API_URL, timeout=30)
        response.raise_for_status()        # 解析JSON响应
        response_data: dict[str, Any] = response.json()

        data_list = response_data['data']

        # 解析代理节点
        proxies: list[ProxyNode] = []        
        for item_data in data_list:  # type: ignore

            url = item_data["url"]
            speed = item_data["speed"]


            proxies.append(ProxyNode(url=url, speed=speed))
        # 按速度降序排序
        proxies.sort(key=lambda x: x.speed, reverse=True)
        
        return proxies
    
    @classmethod
    def save_proxies_to_file(cls, proxies: list[ProxyNode], file_path: Path) -> None:
        """
        将代理列表保存到文件
        
        Args:
            proxies: 代理节点列表
            file_path: 保存文件的路径
            
        Raises:
            OSError: 文件写入失败
            json.JSONEncodeError: JSON编码失败
        """
        # 确保目录存在
        file_path.parent.mkdir(parents=True, exist_ok=True)
        
        # 将代理数据转换为字典列表
        proxy_data = [asdict(proxy) for proxy in proxies]
        
        # 保存到JSON文件
        with open(file_path, 'w', encoding='utf-8') as f:
            json.dump(proxy_data, f, ensure_ascii=False, indent=2)
    
    @classmethod
    def load_proxies_from_file(cls, file_path: Path) -> list[ProxyNode]:
        """
        从文件加载代理列表
        
        Args:
            file_path: 代理文件路径
            
        Returns:
            list[ProxyNode]: 代理节点列表
            
        Raises:
            FileNotFoundError: 文件不存在
            json.JSONDecodeError: JSON解析失败
            ValueError: 数据格式错误
        """
        if not file_path.exists():
            raise FileNotFoundError(f"代理文件不存在: {file_path}")
        
        with open(file_path, 'r', encoding='utf-8') as f:
            proxy_data: list[dict[str,str|float]] = json.load(f)
        # 转换为ProxyNode对象
        proxies: list[ProxyNode] = []
        for item in proxy_data:  # type: ignore
            
            url: str = item['url'] if isinstance(item['url'], str) else str(item['url'])

            speed:float = item['speed'] if isinstance(item['speed'], (int, float)) else float(item['speed'])

            proxies.append(ProxyNode(url=url, speed=speed))
        return proxies

    @classmethod
    def get_and_save_proxies(cls, project_path: Path) -> tuple[list[ProxyNode], Path]:
        """
        获取代理列表并保存到项目的 .rmmp/rmmp.proxys 文件
        
        Args:
            project_path: 项目根目录路径
            
        Returns:
            tuple[list[ProxyNode], Path]: (代理列表, 代理文件路径)
            
        Raises:
            requests.RequestException: 网络请求失败
            OSError: 文件操作失败
        """
        # 获取代理列表
        proxies = cls.get_proxies()
        
        # 确定保存路径
        proxy_file = project_path / ".rmmp" / "rmmp.proxys"
        
        # 保存到文件
        cls.save_proxies_to_file(proxies, proxy_file)
        
        return proxies, proxy_file
    
    @classmethod
    def load_project_proxies(cls, project_path: Path) -> list[ProxyNode]:
        """
        从项目的代理文件加载代理列表
        
        Args:
            project_path: 项目根目录路径
            
        Returns:
            list[ProxyNode]: 代理节点列表，如果文件不存在则返回空列表
        """
        proxy_file = project_path / ".rmmp" / "rmmp.proxys"
        try:
            return cls.load_proxies_from_file(proxy_file)
        except FileNotFoundError:
            return []
    
    @classmethod
    def generate_proxy_download_links(cls, project_path: Path, download_url: str, max_proxies: int = 10) -> str:
        """
        生成代理下载链接的markdown文本，用于发布说明
        
        Args:
            project_path: 项目根目录路径
            download_url: 原始下载链接
            max_proxies: 最多显示的代理数量，默认10个
            
        Returns:
            str: 包含代理下载链接的markdown文本
        """
        proxies = cls.load_project_proxies(project_path)
        if not proxies:
            return ""
        
        # 限制代理数量
        top_proxies = proxies[:max_proxies]
          # 生成markdown文本
        proxy_links: list[str] = []
        proxy_links.append("## 🚀 加速下载链接")
        proxy_links.append("")
        proxy_links.append("代理下载地址列表，已按速度排序：")
        proxy_links.append("")
        
        for i, proxy in enumerate(top_proxies, 1):
            domain_main = proxy.get_domain_main_part()
            speed_text = f"{proxy.speed:.1f}Mb/s" if proxy.speed > 0 else "测速中"
            proxy_download_url = download_url.replace("github.com", proxy.url.replace("https://", "").replace("http://", ""))
            proxy_links.append(f"{i}. [{domain_main} ({speed_text})]({proxy_download_url})")
        
        proxy_links.append("")
        proxy_links.append("*代理节点由第三方提供，速度仅供参考*")
        
        return "\n".join(proxy_links)