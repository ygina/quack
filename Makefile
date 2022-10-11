# MCPU_FLAG = -mcpu=apple-m1
MCPU_FLAG =
FLAGS = -std=gnu++20 -Wall -Wextra -pedantic

build:
	clang++ -O3 $(MCPU_FLAG) $(FLAGS) -o Quack Quack.cpp
	clang++ -O3 $(MCPU_FLAG) $(FLAGS) -o Strawman1 strawman/Strawman1.cpp
	clang++ -O3 $(MCPU_FLAG) -msha $(FLAGS) -o Strawman2 strawman/Strawman2.cpp

benchmark: build
	./scripts/construction.sh graphs/construct.txt  # figure5
	./scripts/decode.sh graphs/decode.txt           # figure6

table: build
	./scripts/table2.sh                             # table2

clean:
	rm -f graphs/*.txt.raw graphs/*.raw Quack Strawman1 Strawman2
