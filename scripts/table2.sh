#!/bin/sh
./Strawman1 -t 20 -n 1000 -b 32 --construct
./Strawman1 -t 20 -n 1000 -b 32 --decode
./Strawman2 -t 20 -n 1000 -b 32 --construct
./Strawman2 -t 20 -n 1000 -b 32 --decode
./QuACK -t 20 -n 1000 -b 32 --construct
./QuACK -t 20 -n 1000 -b 32 --decode

