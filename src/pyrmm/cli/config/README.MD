# RMM Config

from rmm.config import Config

config = Config()

config.key = value # 赋值就是如此简单（主动保存到文件）

del config.key # 删除配置项

value = config.key # 获取配置项的值





# RMM Config CLI

## Usage
rmm config --help

rmm config ls

rmm config get <key> <key2> ...
rmm config set <key>=<value> <key2>=<value2> ...
rmm config del <key> <key2> ...
