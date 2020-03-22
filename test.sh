#!/usr/bin/bash
trap "kill 0" EXIT
cargo build --release

echo testing $1 bytes...
./datagen.py | head -c $1 > tx
./listen.sh >rx 2>listen.log &
sleep 1
cat tx | ./talk.sh &>talk.log
sleep 1

printf 'status: '
cmp -s rx tx && echo 'SUCCESS' || echo 'FAILURE'

echo '## TX ##'
xxd tx

echo '## RX ##'
xxd rx
