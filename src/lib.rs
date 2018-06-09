//! A simple throttle, used for slowing down repeated code. Use this to avoid
//! drowning out downstream systems. For example, if I were reading the contents
//! of a file repeatedly (polling for data, perhaps), or calling an external
//! network resource, I could use a `Throttle` to slow that down to avoid
//! resource contention or browning out a downstream service.
//!
//! This ranges in utility from a simple TPS throttle, "never go faster than *x*
//! transactions per second,"
//!
//! ```rust
//! # extern crate mysteriouspants_throttle;
//! # use std::time::Instant;
//! # use mysteriouspants_throttle::Throttle;
//! # fn main() {
//! // create a new Throttle that rate limits to 10 TPS
//! let throttle = Throttle::new_tps_throttle(10.0);
//!
//! let iteration_start = Instant::now();
//!
//! // iterate eleven times, which at 10 TPS should take just over 1 second
//! for _i in 0..11 {
//!   throttle.acquire(());
//!   // do the needful
//! }
//!
//! // prove that it did, in fact, take 1 second
//! assert_eq!(iteration_start.elapsed().as_secs() == 1, true);
//! # }
//! ```
//!
//! To more complicated variable-rate throttles, which may be as advanced as to slow
//! in response to backpressure.
//!
//! ```rust
//! # extern crate mysteriouspants_throttle;
//! # use std::time::{Duration, Instant};
//! # use mysteriouspants_throttle::Throttle;
//! # fn main() {
//! let throttle = Throttle::new_variable_throttle(|arg: u64, _| Duration::from_millis(arg));
//!
//! let iteration_start = Instant::now();
//!
//! for i in 0..5 {
//!   throttle.acquire(i * 100);
//! }
//!
//! assert_eq!(iteration_start.elapsed().as_secs() == 1, true);
//! # }

use std::cell::Cell;
use std::time::{Duration, Instant};
use std::thread::sleep;

#[derive(Copy, Clone)]
enum ThrottleState {
    Uninitialized,
    Initialized {
        previous_invocation: Instant
    }
}

// A simple configurable throttle for slowing down code.
pub struct Throttle<TArg> {
    delay_calculator: Box<Fn(TArg, Duration) -> Duration>,
    state: Cell<ThrottleState>
}

impl <TArg> Throttle<TArg> {
    /// Creates a new `Throttle` with a variable delay controlled by a closure. `delay_calculator`
    /// itself is an interesting type, any closure which satisfies `Fn(TArg, Duration) -> Duration`.
    /// It is called to determine how long the `Throttle` ought to wait before resuming execution,
    /// and allows you to create `Throttle`s which respond to changes in the program or environment.
    ///
    /// They range from the simple:
    ///
    /// ```rust
    /// # extern crate mysteriouspants_throttle;
    /// # use std::time::{Duration, Instant};
    /// # use mysteriouspants_throttle::Throttle;
    /// let throttle = Throttle::new_variable_throttle(|_, _| Duration::from_millis(1100));
    ///
    /// // the first one is free!
    /// throttle.acquire(());
    ///
    /// let start = Instant::now();
    /// throttle.acquire(());
    /// assert_eq!(start.elapsed().as_secs() == 1, true);
    /// ```
    ///
    /// To the complex:
    ///
    /// ```rust
    /// # extern crate mysteriouspants_throttle;
    /// # use std::time::{Duration, Instant};
    /// # use mysteriouspants_throttle::Throttle;
    /// let throttle = Throttle::new_variable_throttle(
    ///     |in_backpressure: bool, time_since_previous_acquire: Duration|
    ///         match in_backpressure {
    ///             true => Duration::from_millis(2100),
    ///             false => Duration::from_millis(1100)
    ///         });
    ///
    /// // the first one is free!
    /// throttle.acquire(false);
    ///
    /// let start_nopressure = Instant::now();
    /// throttle.acquire(false);
    /// assert_eq!(start_nopressure.elapsed().as_secs() == 1, true);
    ///
    /// let start_yespressure = Instant::now();
    /// throttle.acquire(true);
    /// assert_eq!(start_yespressure.elapsed().as_secs() == 2, true);
    /// ```
    pub fn new_variable_throttle<TDelayCalculator>(delay_calculator: TDelayCalculator) -> Throttle<TArg>
            where TDelayCalculator: Fn(TArg, Duration) -> Duration + 'static {
        return Throttle {
            delay_calculator: Box::new(delay_calculator),
            state: Cell::new(ThrottleState::Uninitialized)
        };
    }

    /// Creates a new `Throttle` with a constant delay of `tps`^-1 * 1000, or `tps`-transactions per
    /// second.
    pub fn new_tps_throttle(tps: f32) -> Throttle<TArg> {
        return Throttle {
            delay_calculator: Box::new(move |_, _|
                Duration::from_millis(((1.0 / tps) * 1000.0) as u64)),
            state: Cell::new(ThrottleState::Uninitialized)
        };
    }

    /// Acquires the throttle, waiting (sleeping the current thread) until enough time has passed
    /// for the running code to be at or slower than the throttle allows. The first call to
    /// `acquire` will never wait because there has been an undefined or arguably infinite amount
    /// of time from the previous time acquire was called. The argument `arg` is passed to the
    /// closure governing the wait time.
    pub fn acquire(&self, arg: TArg) {
        match self.state.get() {
            ThrottleState::Initialized { previous_invocation } => {
                let time_since_previous_acquire =
                    Instant::now().duration_since(previous_invocation);
                let delay_time = (self.delay_calculator)(arg, time_since_previous_acquire);
                let additional_delay_required = delay_time - time_since_previous_acquire;

                if additional_delay_required > Duration::from_secs(0) {
                    sleep(additional_delay_required);
                }

                self.state.replace(ThrottleState::Initialized {
                    previous_invocation: Instant::now()
                });
            },
            ThrottleState::Uninitialized => {
                self.state.replace(ThrottleState::Initialized {
                    previous_invocation: Instant::now()
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use Throttle;

    #[test]
    fn it_works() {
        // simple throttle configured for 10 TPS
        let throttle = Throttle::new_tps_throttle(10.0);

        let iteration_start = Instant::now();

        for _i in 0..11 {
            throttle.acquire(());
        }

        assert_eq!(iteration_start.elapsed().as_secs() == 1, true);
    }
}
