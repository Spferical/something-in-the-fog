ROOT=$(cargo metadata --format-version=1 | jq -r .resolve.root)
PROJECT_NAME=$(cargo metadata --format-version=1 \
               | jq -r ".packages[] | select(.id==\"${ROOT}\") | .name")
TARGET_DIR=$(cargo metadata --format-version=1 | jq -r .target_directory)/wasm32-unknown-unknown
# Build
if [ -n "$RELEASE" ]; then
	cargo build --release --target wasm32-unknown-unknown
	TARGET_DIR="$TARGET_DIR/release"
else
	cargo build --target wasm32-unknown-unknown
	TARGET_DIR="$TARGET_DIR/debug"
fi

# Generate bindgen outputs
mkdir -p dist
wasm-bindgen $TARGET_DIR/"$PROJECT_NAME".wasm --out-dir dist --target web --no-typescript
cp -r wasm/* dist/
