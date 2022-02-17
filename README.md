# subset-digest

```
$ cd benchmark
$ cargo r -- --accumulator naive -p 0.02 -n 100 --malicious
[2022-02-17T21:53:03Z DEBUG benchmark] num_logged = 100
[2022-02-17T21:53:03Z DEBUG benchmark] p_dropped = 0.02
[2022-02-17T21:53:03Z DEBUG benchmark] malicious = true
[2022-02-17T21:53:03Z DEBUG benchmark] dropped 2/100 elements
[2022-02-17T21:53:03Z INFO  benchmark] processed 98 elements in 85.915µs
[2022-02-17T21:53:03Z INFO  benchmark] validation took 84.428982ms
[2022-02-17T21:53:03Z INFO  benchmark] validation is correct (false)
```