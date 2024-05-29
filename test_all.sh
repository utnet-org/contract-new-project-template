#!/bin/bash
set -e

pushd $(dirname ${BASH_SOURCE[0]})

for d in */test.sh ; do
    d=$(dirname "$d");
    echo "Testing $d";
    (cd $d;./test.sh)
done

popd