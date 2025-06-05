"""rmm config sub commands group for pyrmm

rmm config --path,-p <path> # set RMMROOT path(RMMROOT)
rmm config -w <key>=<value>,... # write key-value pairs to config
rmm config -r <key>,<key2>,... # v1, v2 ...

"""

from .__main__ import config

__all__ = ['config']