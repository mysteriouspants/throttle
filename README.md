# Throttle

[![Crates.io](https://img.shields.io/crates/v/mysteriouspants-throttle.svg)](https://crates.io/crates/mysteriouspants-throttle)
[![Documentation](https://docs.rs/mysteriouspants-throttle/badge.svg)](https://docs.rs/mysteriouspants-throttle/)
[![Build Status](https://travis-ci.org/mysteriouspants/throttle.svg?branch=master)](https://travis-ci.org/mysteriouspants/throttle)

A simple, configurable throttle for slowing down code. When do you actually want to slow down code? To avoid resource
contention and browning out downstream services.

```rust
// simple throttle configured for 10 TPS
let mut throttle = Throttle::new_tps_throttle(10.0);

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
let mut throttle = Throttle::new_variable_throttle(
    |iteration: u32, _| Duration::from_millis(arg));

let iteration_start = Instant::now();

// the first iteration is free, subsequent iterations
// will be slowed down
for i in 0..5 {
    throttle.acquire(i * 100);
}

assert_eq!(iteration_start.elapsed().as_secs() == 1, true);
```

# License

I want you to be able to use this software regardless of who you may be, what you are working on, or the environment in
which you are working on it - I hope you'll use it for good and not evil! To this end, Throttle is licensed under the
[2-clause BSD][2cbsd] license, with other licenses available by request. Happy coding!

[2cbsd]: https://opensource.org/licenses/BSD-2-Clause
