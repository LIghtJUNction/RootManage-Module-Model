$env:ANDROID_NDK_HOME = "D:\HOME\APP\Programmer\NDK\android-ndk-r27c"
$env:GOOS = "android"
$env:GOARCH = "arm64"


go build -o UniCrond

# 验证 
# vscode打开wsl终端 
# file UniCrond
# ❯ file UniCrond                                                                                                   ─╯ 
# UniCrond: ELF 64-bit LSB pie executable, ARM aarch64, version 1 (SYSV), dynamically linked, interpreter /system/bin/# # # # 
# linker64, Go BuildID=ejU23g1YO_pFZDSeF0lb/H47vPICuV7WIAVvghJDM/GRn5iIBPGdgCY6mQf5vZ/6iwBiFiMGhnGef8gbk-c, with debug_info, 
# not stripped