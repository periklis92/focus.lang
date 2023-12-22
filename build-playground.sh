#!/bin/bash

ROOT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
PLAYGROUND_DIR=$ROOT_DIR/playground
BUILD_DIR=$PLAYGROUND_DIR/build

rm -rf $ROOT_DIR/docs

npm run build --prefix $PLAYGROUND_DIR

# cp $BUILD_DIR/index.html $ROOT_DIR
cp -r $BUILD_DIR $ROOT_DIR/docs
