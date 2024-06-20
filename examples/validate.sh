#!/bin/sh
set -e
echo "Home: $HOME"

cd $1
mm.py README.md
