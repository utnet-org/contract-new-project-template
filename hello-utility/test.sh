#!/bin/bash
set -e

pushd $(dirname ${BASH_SOURCE[0]})

cargo test --workspace -- --nocapture

popd