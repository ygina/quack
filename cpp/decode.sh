#!/bin/sh
for bits in 16 32 64;
do
	echo "$bits bits"
	echo "t\ttime_us"
	for dropped in $(seq 0 1 20);
	do
		time_ns=$(./TestProgram -b $bits --dropped $dropped --decode \
			| grep SUMMARY | awk '{print $7}')
		time_us=$(($time_ns/1000))
		echo "$dropped\t$time_us"
	done
	echo ""
done

