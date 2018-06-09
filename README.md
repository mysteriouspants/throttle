# Throttle

A simple, configurable throttle for slowing down code. When do you actually want to slow down code? To avoid resource
contention and browning out downstream services.

```rust
// simple throttle configured for 10 TPS
let throttle = Throttle::new_tps_throttle(10.0);

let iteration_start = Instant::now();

// the first iteration is free, subsequent iterations
// will be slowed down to a rate of 10 TPS, or one iteration
// every 100 milliseconds
for _i in 0..11 {
    throttle.acquire(());
}

println!("elapsed time: {:?}", iteration_start.elapsed());

assert_eq!(iteration_start.elapsed().as_secs() == 1, true);
```

# License

Throttle is licensed under the [2-clause BSD](https://opensource.org/licenses/BSD-2-Clause) license.