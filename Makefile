# Full Android release: dx bundle + icons + gradle build + sign + align
# Loads keystore path (game-release.keystore) and password (.env) automatically
release-android:
	@./scripts/build-android.sh

# Just bundle with dx (no signing)
dx-bundle:
	dx bundle --platform android --release --target aarch64-linux-android

.PHONY: release-android dx-bundle
