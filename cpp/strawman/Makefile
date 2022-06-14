# MCPU_FLAG = -mcpu=apple-m1
MCPU_FLAG =
LDFLAGS="-L/usr/local/opt/openssl@3/lib"
CPPFLAGS="-I/usr/local/opt/openssl@3/include"

build: s1 s2

s1:
	clang++ -O3 $(MCPU_FLAG) \
		-std=c++20 -Wall -Wextra -pedantic -o Strawman1 Strawman1.cpp

s2:
	clang++ -O3 $(MCPU_FLAG) $(LDFLAGS) $(CPPFLAGS) -msha \
		-std=c++20 -Wall -Wextra -pedantic -o Strawman2 Strawman2.cpp

