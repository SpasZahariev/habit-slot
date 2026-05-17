include .env
export

release-android:
	dx bundle --platform android --release --target aarch64-linux-android
