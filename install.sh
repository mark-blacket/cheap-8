#!/bin/sh

# This needs root privileges to set guid on binaries for libinput

cargo install --path . --root .
for file in ./bin/*; do
    chown :input $file
    chmod g+s $file
done
