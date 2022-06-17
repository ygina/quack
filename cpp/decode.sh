#!/bin/sh
for bits in 16 24 32 64;
do
	if [[ "$bits" -eq 16 ]]
	then
		tables_flag=--use-tables
	else
		tables_flag=
	fi
	echo "$bits bits"
	echo "t\ttime_us ($tables_flag)"
	for dropped in $(seq 0 1 20);
	do
		time_ns=$(./TestProgram -b $bits --dropped $dropped --decode \
			$tables_flag | grep SUMMARY | awk '{print $7}')
		time_us=$(($time_ns/1000))
		echo "$dropped\t$time_us"
	done
	echo ""
done
