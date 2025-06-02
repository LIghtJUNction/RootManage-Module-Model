gogogo -s "../gogogo/gogogo.go" -o "./dist" -p "android/arm64" -v 2 

mv "./dist/gogogo_android_arm64" "../gogogo/bin/gogogo"

rm -r "./dist"