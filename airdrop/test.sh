#!/bin/bash
set -e

pushd $(dirname ${BASH_SOURCE[0]})

cargo nextest run --nocapture

popd