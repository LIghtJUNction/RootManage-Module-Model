
class ChangeLogMeta(type):
    """
    抓取Magisk Apatch kernelsu 的更新日志
    并缓存，隶属于mcp模块
    """
    MAGISK : list[str] = ["https://topjohnwu.github.io/Magisk/changes.html",]
    APATCH : list[str] = ["https://apatch.app/99652819.html",]
    KERNELSU : list[str] = ["https://github.com/KernelSU/KernelSU/blob/master/CHANGELOG.md",]