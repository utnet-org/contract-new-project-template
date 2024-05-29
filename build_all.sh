#!/bin/bash
set -e
rustup target add wasm32-unknown-unknown

echo $(rustc --version)
pushd $(dirname ${BASH_SOURCE[0]})

for d in $(ls -d */ | grep -v -e "res\/$" -e "target\/$"); do
    echo building $d;
    (cd "$d"; ./build.sh);
done

popd