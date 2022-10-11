#!/bin/sh
./Strawman1 -t 20 -n 1000 -b 32 --insertion
./Strawman1 -t 20 -n 1000 -b 32 --decode
./Strawman2 -t 20 -n 1000 -b 32 --insertion
./Strawman2 -t 20 -n 1000 -b 32 --decode
./QuACK -t 20 -n 1000 -b 32 --insertion
./QuACK -t 20 -n 1000 -b 32 --decode

