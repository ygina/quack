#!/bin/sh
for bits in 16 32 64;
do
	echo "$bits bits"
	echo "t\ttime_us"
	for threshold in $(seq 5 5 50);
	do
		time_ns=$(./TestProgram -t $threshold -b $bits --dropped 0 --insertion \
			| grep SUMMARY | awk '{print $7}')
		time_us=$(($time_ns/1000))
		echo "$threshold\t$time_us"
	done
	echo ""
done

