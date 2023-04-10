#!/bin/sh

set -u

cargo build --profile wasm-release --features wasm --target wasm32-unknown-unknown
rm -rf wasm wasm.zip
wasm-bindgen --no-typescript --out-name bevy_game --out-dir wasm --target web target/wasm32-unknown-unknown/release/bevy_jam_side_effects.wasm
cp -r assets web/* wasm
rm wasm/**/*.aseprite
wasm-opt -O -ol 100 -s 100 -o wasm/bevy_game_bg.wasm wasm/bevy_game_bg.wasm
zip -r wasm.zip wasm

exit 0
