#!/usr/bin/python3

## Outputs bytes for testing aserial transmission quality
# It works by creating three sub-datasets, 256 random bytes, and two sequences:
# [0, 255, 1, 254, ..., 127, 128] and [0, 128, 1, 129, ..., 127, 255]. Then those
# three lists are concatenated into one, which is then shuffled in chunks of 32 bytes
# to mix up the properties of used datasets.
## Example (dumping the bytes to a file):
# ./datagen.py > data

import random
import sys

def random_bytes(n):
	return [random.getrandbits(8) for _ in range(n)]

def decreasing_jump_len():
	result = []
	for k in range(0, 256):
		if k % 2 == 0:
			result.append(k//2)
		else:
			result.append(255 - (k-1) // 2)
	return result

def mid_jump_len():
	result = []
	for k in range(0, 256):
		if k % 2 == 0:
			result.append(k//2)
		else:
			result.append(128 + (k-1) // 2)
	return result

def chunks(lst, n):
    """Yield successive n-sized chunks from lst."""
    for i in range(0, len(lst), n):
        yield lst[i:i + n]

def flatten(lst):
	return [item for sublst in lst for item in sublst]

result = random_bytes(256) + decreasing_jump_len() + mid_jump_len()
chks = list(chunks(result, 32))
random.shuffle(chks)

sys.stdout.buffer.write(bytes(flatten(chks)))
