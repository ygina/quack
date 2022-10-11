# MCPU_FLAG = -mcpu=apple-m1
MCPU_FLAG =

build:
	clang++ -O3 $(MCPU_FLAG) -std=gnu++20 -Wall -Wextra -pedantic -o TestProgram TestProgram.cpp

benchmark: build
	./decode.sh ../../sidecar-paper/graphs/decode.txt
	./insertion.sh ../../sidecar-paper/graphs/insertion.txt
