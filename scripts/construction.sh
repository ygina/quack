#!/bin/sh
: ${1?"Usage: $0 OUTFILE"}

OUTFILE=$1
RAWFILE=$OUTFILE.raw
rm -f $OUTFILE $RAWFILE

for bits in 16 24 32 64;
do
	if [[ "$bits" -eq 16 ]]
	then
		tables_flag=--use-tables
	else
		tables_flag=
	fi
	echo "$bits bits" | tee -a $RAWFILE >> $OUTFILE
	echo "t\ttime_us ($tables_flag)" | tee -a $RAWFILE >> $OUTFILE
	for threshold in $(seq 5 5 50);
	do
		time_ns=$(./Quack -t $threshold -b $bits --dropped 0 --construct \
			--trials 100 $tables_flag | tee -a $RAWFILE \
			| grep SUMMARY | awk '{print $7}')
		time_us=$(($time_ns/1000))
		echo "$threshold\t$time_us" | tee -a $RAWFILE >> $OUTFILE
	done
	echo "" | tee -a $RAWFILE >> $OUTFILE
done

