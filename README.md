# subset-digest

## Dependencies

* [PARI](https://pari.math.u-bordeaux.fr/download.html): factoring polynomials
  in finite fields
* [GLPK](https://www.gnu.org/software/glpk/): solving ILPs

## Tests
```
$ cd accumulator
$ cargo t -- --test-threads 1
```

## Benchmarks
```
$ cd benchmark
$ cargo b --release
$ ./target/release/benchmark --accumulator naive -p 0.02 -n 100 --malicious
[2022-02-17T21:53:03Z DEBUG benchmark] dropped 2/100 elements
[2022-02-17T21:53:03Z INFO  benchmark] processed 98 elements in 85.915µs
[2022-02-17T21:53:03Z INFO  benchmark] validation took 84.428982ms
[2022-02-17T21:53:03Z INFO  benchmark] validation is correct (false)
```

```
$ ./target/release/benchmark -n 50000 --accumulator power_sum --threshold 1000 -p 0.01
[2022-03-12T00:29:53Z DEBUG benchmark] dropped 513/50000 elements
[2022-03-12T00:29:53Z INFO  benchmark] processed 49487 elements in 4.310809933s
[2022-03-12T00:29:53Z DEBUG accumulator::power_sum] found 16 cpus
[2022-03-12T00:29:53Z DEBUG accumulator::power_sum] calculated power sums: 352.48839ms
[2022-03-12T00:29:53Z DEBUG accumulator::power_sum] calculated power sum difference: 47.354µs
[2022-03-12T00:29:53Z DEBUG accumulator::power_sum] computed polynomial coefficients: 15.103627ms
[2022-03-12T00:29:54Z DEBUG accumulator::power_sum] found integer monic polynomial roots: 68.370784ms
[2022-03-12T00:29:54Z DEBUG accumulator::power_sum] checked roots against element list: 1.898837ms
[2022-03-12T00:29:54Z INFO  benchmark] validation took 438.790055ms
```

```
$ ./target/release/benchmark -n 50000 --accumulator cbf --threshold 1000 -p 0.01
[2022-03-12T00:28:57Z DEBUG benchmark] dropped 487/50000 elements
[2022-03-12T00:28:57Z INFO  benchmark] processed 49513 elements in 4.146595ms
[2022-03-12T00:28:57Z DEBUG accumulator::cbf] calculated the difference cbf: 3.398536ms
[2022-03-12T00:28:57Z DEBUG accumulator::cbf] setup the system of equations: 2.139251ms
[2022-03-12T00:28:57Z DEBUG accumulator::cbf] solved an ILP with 628 equations in 19170 variables: 3.387916ms
[2022-03-12T00:26:42Z INFO  benchmark] validation took 11.205708ms
```
