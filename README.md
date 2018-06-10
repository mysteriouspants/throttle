# Throttle

[![Crates.io](https://img.shields.io/crates/v/mysteriouspants-throttle.svg)](https://crates.io/crates/mysteriouspants-throttle)
[![Documentation](https://docs.rs/mysteriouspants-throttle/badge.svg)](https://docs.rs/mysteriouspants-throttle/)
[![Build Status](https://travis-ci.org/mysteriouspants/throttle.svg?branch=master)](https://travis-ci.org/mysteriouspants/throttle)

A simple, configurable throttle for slowing down code. When do you actually want to slow down code? To avoid resource
contention and browning out downstream services.

```rust
// simple throttle configured for 10 TPS
let throttle = Throttle::new_tps_throttle(10.0);

let iteration_start = Instant::now();

// the first one is free!
throttle.acquire(());

// the first iteration is free, subsequent iterations
// will be slowed down to a rate of 10 TPS, or one iteration
// every 100 milliseconds
for _i in 0..10 {
    throttle.acquire(());
}

println!("elapsed time: {:?}", iteration_start.elapsed());

assert_eq!(iteration_start.elapsed().as_secs() == 1, true);
```

Throttle is based on a functional interface, so it can go beyond constant tps rate limiting to facilitating
variable-rate throttling based on conditions entirely up to your program.

```rust
let throttle = Throttle::new_variable_throttle(
    |iteration: u32, _| Duration::from_millis(arg));

// the first one is free, so the number won't get used
throttle.acquire(0);

let iteration_start = Instant::now();

for i in 1..5 {
    throttle.acquire(i * 100);
}

assert_eq!(iteration_start.elapsed().as_secs() == 1, true);
```

# License

Throttle is licensed under the [2-clause BSD](https://opensource.org/licenses/BSD-2-Clause) license.