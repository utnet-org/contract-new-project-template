#!/bin/bash
set -e
rustup target add wasm32-unknown-unknown

echo $(rustc --version)
pushd $(dirname ${BASH_SOURCE[0]})

for d in $(ls -d */ | grep -v -e "res\/$" -e "target\/$" -e "scripts\/$" -e "state-manipulation\/$"); do
    echo building $d;
    (cd "$d"; ./build.sh);
done

for wasm in res/*.wasm; do
    du -sh "$wasm"
done

popd