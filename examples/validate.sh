#!/bin/sh
set -e
echo "Home: $HOME"

cd src/$1
mm.py README.md
