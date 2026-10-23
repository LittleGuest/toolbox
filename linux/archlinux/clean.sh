#!/bin/bash

cd "$(dirname "$0")"

makepkg -fc
rm -rf src pkg toolbox toolbox*.pkg.tar.zst toolbox.svg toolbox.desktop LICENSE README.md
