#!/usr/bin/bash
trap "kill 0" EXIT
cargo build --release

echo testing $1 bytes...
./datagen.py | head -c $1 > tx
./listen.sh > rx &
sleep 1
cat tx | ./talk.sh
sleep 1

md5sum tx rx
