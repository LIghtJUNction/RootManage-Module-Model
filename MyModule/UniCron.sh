MODDIR=${0%/*}
source $MODDIR/tools.sh # 导入工具函数
# 1分钟检查一次
UniCronMain
set_module_description "$(check)"

if [ -f $MODDIR/disable ]; then
    crond_kill
    crontab_kill
fi