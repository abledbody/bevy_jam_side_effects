#!/bin/sh

set -u
shopt -s globstar

NAME='sai_defects'
BUILD_DIR='target/build'

# Web
function web() {
  PLATFORM='web'
  TARGET='wasm32-unknown-unknown'
  OUT_DIR="${BUILD_DIR}/${PLATFORM}"
  OUT_ZIP="${BUILD_DIR}/${NAME}_${PLATFORM}.zip"

  mkdir -p "${OUT_DIR}"
  rm -rf "${OUT_DIR}/*" "${OUT_ZIP}"
  cargo build --profile wasm-release --features wasm --target "${TARGET}"
  wasm-bindgen --no-typescript --out-name "${NAME}" --out-dir "${OUT_DIR}" --target web target/"${TARGET}"/wasm-release/"${NAME}".wasm
  cp -r assets web/* "${OUT_DIR}"
  rm "${OUT_DIR}"/**/*.aseprite
  wasm-opt -O -ol 100 -s 100 -o "${OUT_DIR}/${NAME}"_bg.wasm "${OUT_DIR}/${NAME}"_bg.wasm
  zip -r "${OUT_ZIP}" "${OUT_DIR}"
}

# Linux
function linux() {
  PLATFORM='linux'
  TARGET='x86_64-unknown-linux-gnu'
  OUT_DIR="${BUILD_DIR}/${PLATFORM}"
  OUT_ZIP="${BUILD_DIR}/${NAME}_${PLATFORM}.zip"

  mkdir -p "${OUT_DIR}"
  rm -rf "${OUT_DIR}"/* "${OUT_ZIP}"
  cargo build --release --features bevy/x11 --target "${TARGET}"
  cp -r assets target/"${TARGET}"/release/"${NAME}" "${OUT_DIR}"
  rm "${OUT_DIR}"/**/*.aseprite
  zip -r "${OUT_ZIP}" "${OUT_DIR}"
}

# Windows
function windows() {
  PLATFORM='windows'
  TARGET='x86_64-pc-windows-gnu'
  OUT_DIR="${BUILD_DIR}/${PLATFORM}"
  OUT_ZIP="${BUILD_DIR}/${NAME}_${PLATFORM}.zip"

  mkdir -p "${OUT_DIR}"
  rm -rf "${OUT_DIR}"/* "${OUT_ZIP}"
  cargo build --release --target "${TARGET}"
  cp -r assets target/"${TARGET}"/release/"${NAME}".exe "${OUT_DIR}"
  rm "${OUT_DIR}"/**/*.aseprite
  zip -r "${OUT_ZIP}" "${OUT_DIR}"
}

# Mac
function mac() {
  PLATFORM='mac'
  TARGET='x86_64-apple-darwin'
  OUT_DIR="${BUILD_DIR}/${PLATFORM}"
  OUT_ZIP="${BUILD_DIR}/${NAME}_${PLATFORM}.zip"

  mkdir -p "${OUT_DIR}"
  rm -rf "${OUT_DIR}"/* "${OUT_ZIP}"
  cargo build --release --target "${TARGET}"
  cp -r assets target/"${TARGET}"/release/"${NAME}".exe "${OUT_DIR}"
  rm "${OUT_DIR}"/**/*.aseprite
  zip -r "${OUT_ZIP}" "${OUT_DIR}"
}

mac

exit 0
