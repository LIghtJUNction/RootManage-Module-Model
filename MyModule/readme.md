# 搜索下载NDK - 解压 把下面这个路径改成你的实际NDK路径
set ANDROID_NDK_ROOT=D:\HOME\APP\Programmer\NDK\android-ndk-r27c

set GOOS=android
set GOARCH=arm64
set CC=%ANDROID_NDK_ROOT%\toolchains\llvm\prebuilt\windows-x86_64\bin\aarch64-linux-android21-clang.cmd

go build -o UniCrond UniCrond.go