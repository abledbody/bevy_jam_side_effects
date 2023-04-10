#!/bin/sh

set -u

cargo build --profile wasm-release --features wasm --target wasm32-unknown-unknown
rm -rf wasm wasm.zip
wasm-bindgen --no-typescript --out-name bevy_game --out-dir wasm --target web target/wasm32-unknown-unknown/release/bevy_jam_side_effects.wasm
cp -r assets web/* wasm
rm wasm/**/*.aseprite
zip --recurse-paths wasm.zip wasm

exit 0
