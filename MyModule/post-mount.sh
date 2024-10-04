 
# 这个脚本将在 post-mount 模式下运行。
# 
# post-mount 模式说明：
# - 这个阶段是阻塞的。在执行完成之前或者 10 秒钟之后，启动过程会暂停。
# - 脚本在任何模块被挂载之前运行。这使得模块开发者可以在模块被挂载之前动态地调整它们的模块。
# - 这个阶段发生在 Zygote 启动之前。
# - 使用 setprop 会导致启动过程死锁！请使用 `resetprop -n <prop_name> <prop_value>` 代替。
# - **只有在必要时才在此模式下运行脚本**。
# 
# 变量：
# - `MODDIR=${0%/*}`：获取模块的基本目录路径。
# - `KSU` (bool)：标记此脚本运行在 KernelSU 环境下，此变量的值将永远为 `true`。
# - `KSU_VER` (string)：KernelSU 当前的版本名字 (如： `v0.4.0`)。
# - `KSU_VER_CODE` (int)：KernelSU 用户空间当前的版本号 (如. `10672`)。
# - `KSU_KERNEL_VER_CODE` (int)：KernelSU 内核空间当前的版本号 (如. `10672`)。
# - `BOOTMODE` (bool)：此变量在 KernelSU 中永远为 `true`。
# - `MODPATH` (path)：当前模块的安装目录。
# - `TMPDIR` (path)：可以存放临时文件的目录。
# - `ZIPFILE` (path)：当前模块的安装包文件。
# - `ARCH` (string)：设备的 CPU 构架，有如下几种： `arm`, `arm64`, `x86`, or `x64`。
# - `IS64BIT` (bool)：是否是 64 位设备。
# - `API` (int)：当前设备的 Android API 版本 (如：Android 6.0 上为 `23`)。
# 
# 函数：
# - `ui_print <msg>`：打印 <msg> 到控制台。避免使用 'echo'，因为它不会显示在自定义恢复的控制台中。
# - `abort <msg>`：打印错误信息 <msg> 到控制台并终止安装。避免使用 'exit'，因为它会跳过终止清理步骤。
# - `set_perm <target> <owner> <group> <permission> [context]`：设置目标文件的权限和所有者。
# - `set_perm_recursive <directory> <owner> <group> <dirpermission> <filepermission> [context]`：递归设置目录及其内容的权限和所有者。
#这个脚本将会在 post-mount 模式下运行
