#!/bin/sh
./strawman/Strawman1 -t 20 -n 1000 -b 32 --insertion
./strawman/Strawman1 -t 20 -n 1000 -b 32 --decode
./strawman/Strawman2 -t 20 -n 1000 -b 32 --insertion
./strawman/Strawman2 -t 20 -n 1000 -b 32 --decode
./QuACK -t 20 -n 1000 -b 32 --insertion
./QuACK -t 20 -n 1000 -b 32 --decode

