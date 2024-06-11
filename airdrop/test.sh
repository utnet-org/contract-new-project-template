#!/bin/bash
set -e

./build.sh
pushd $(dirname ${BASH_SOURCE[0]})

cargo nextest run --nocapture

popd