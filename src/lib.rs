//! A simple throttle, used for slowing down repeated code. Use this to avoid drowning out
//! downstream systems. For example, if I were reading the contents of a file repeatedly (polling
//! for data, perhaps), or calling an external network resource, I could use a `Throttle` to slow
//! that down to avoid resource contention or browning out a downstream service. Another potential
//! use of a `Throttle` is in video game code to lock a framerate lower to promote predictable
//! gameplay or to avoid burning up user's graphics hardware unnecessarily.
//!
//! This ranges in utility from a simple TPS throttle, "never go faster than *x* transactions per
//! second,"
//!
//! ```rust
//! # extern crate mysteriouspants_throttle;
//! # use std::time::Instant;
//! # use mysteriouspants_throttle::Throttle;
//! # fn main() {
//! // create a new Throttle that rate limits to 10 TPS
//! let mut throttle = Throttle::new_tps_throttle(10.0);
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
//! To more complicated variable-rate throttles, which may be as advanced as to slow in response to
//! backpressure.
//!
//! ```rust
//! # extern crate mysteriouspants_throttle;
//! # use std::time::{Duration, Instant};
//! # use mysteriouspants_throttle::Throttle;
//! # fn main() {
//! let mut throttle = Throttle::new_variable_throttle(
//!     |arg: u64, _| Duration::from_millis(arg));
//!
//! let iteration_start = Instant::now();
//!
//! for i in 0..5 {
//!   throttle.acquire(i * 100);
//! }
//!
//! assert_eq!(iteration_start.elapsed().as_secs() == 1, true);
//! # }
//! ```
//!
//! When using your throttle, keep in mind that you are responsible for sharing it between
//! threads safely and responsibly.

use std::time::{Duration, Instant};
use std::thread::sleep;

#[derive(Copy, Clone)]
enum ThrottleState {
    Uninitialized,
    Initialized {
        previous_invocation: Instant
    }
}

/// A simple configurable throttle for slowing down code, a little struct holding some state.
pub struct Throttle<TArg> {
    delay_calculator: Box<Fn(TArg, Duration) -> Duration>,
    state: ThrottleState
}

impl <TArg> Throttle<TArg> {
    /// Creates a new `Throttle` with a variable delay controlled by a closure. `delay_calculator`
    /// itself is an interesting type, any closure which satisfies `Fn(TArg, Duration) -> Duration`.
    ///
    /// This lambda is called to determine the duration between iterations of your code.
    ///
    /// ```text
    /// |TArg, Duration| -> Duration
    ///   |      |             |
    ///   |      |             |
    ///   |      |             v
    ///   |      |    Duration that ought to have elapsed between calls
    ///   |      |    to acquire. If the Duration you return is less
    ///   |      |    than the Duration passed to you, or is zero, that
    ///   |      |    means that no additional time will to be waited.
    ///   |      |
    ///   |      +--> The time since the previous call to acquire and now.
    ///   |
    ///   |           An argument passed through from the call to acquire.
    ///   +---------> You can use this to change the behavior of your
    ///               Throttle based on conditions in your calling code.
    /// ```
    ///
    /// Expressed differently, on the axis of time,
    ///
    /// ```text
    ///   /------------lambda return--------------------\
    ///   /----duration arg-----\                       |
    ///  +-----------------------------------------------+
    ///  ^                       ^\------additional-----/
    ///  |                       |       time waited
    ///  |                       |
    ///  previous call           acquire called
    ///  to acquire
    /// ```
    ///
    /// An example use of a variable-rate throttle might be to wait different periods of time
    /// depending on whether your program is in backpressure, so "ease up" on your downstream call
    /// rate, so to speak.
    ///
    /// ```
    /// # extern crate mysteriouspants_throttle;
    /// # use std::time::{Duration, Instant};
    /// # use mysteriouspants_throttle::Throttle;
    /// let mut throttle = Throttle::new_variable_throttle(
    ///     |in_backpressure: bool, time_since_previous_acquire: Duration|
    ///         match in_backpressure {
    ///             true => Duration::from_millis(210),
    ///             false => Duration::from_millis(110)
    ///         });
    ///
    /// // the first one is free!
    /// throttle.acquire(false);
    ///
    /// let start_nopressure = Instant::now();
    /// throttle.acquire(false);
    /// assert_eq!(start_nopressure.elapsed().as_secs() == 0, true);
    /// assert_eq!(start_nopressure.elapsed().subsec_nanos() >= 100_000_000, true);
    ///
    /// let start_yespressure = Instant::now();
    /// throttle.acquire(true);
    /// assert_eq!(start_yespressure.elapsed().as_secs() == 0, true);
    /// assert_eq!(start_yespressure.elapsed().subsec_nanos() >= 200_000_000, true);
    /// ```
    pub fn new_variable_throttle<TDelayCalculator: Fn(TArg, Duration) -> Duration + 'static>(
        delay_calculator: TDelayCalculator) -> Throttle<TArg> {
        return Throttle {
            delay_calculator: Box::new(delay_calculator),
            state: ThrottleState::Uninitialized
        };
    }

    /// Creates a new `Throttle` with a constant delay of `tps`<sup>-1</sup> &middot; 1000 ms, or
    /// `tps`-transactions per second.
    ///
    /// ```rust
    /// # extern crate mysteriouspants_throttle;
    /// # use std::time::{Duration, Instant};
    /// # use mysteriouspants_throttle::Throttle;
    /// let mut throttle = Throttle::new_tps_throttle(0.9);
    ///
    /// // the first one is free!
    /// throttle.acquire(());
    ///
    /// let start = Instant::now();
    /// throttle.acquire(());
    /// assert_eq!(start.elapsed().as_secs() == 1, true);
    /// ```
    pub fn new_tps_throttle(tps: f32) -> Throttle<TArg> {
        let wait_for_millis = ((1.0 / tps) * 1000.0) as u64;
        return Throttle {
            delay_calculator: Box::new(move |_, _|
                Duration::from_millis(wait_for_millis)),
            state: ThrottleState::Uninitialized
        };
    }

    /// Acquires the throttle, waiting (sleeping the current thread) until enough time has passed
    /// for the running code to be at or slower than the throttle allows. The first call to
    /// `acquire` will never wait because there has been an undefined or arguably infinite amount
    /// of time from the previous time acquire was called. The argument `arg` is passed to the
    /// closure governing the wait time.
    pub fn acquire(&mut self, arg: TArg) {
        match self.state {
            ThrottleState::Initialized { previous_invocation } => {
                let time_since_previous_acquire =
                    Instant::now().duration_since(previous_invocation);
                let delay_time = (self.delay_calculator)(arg, time_since_previous_acquire);

                if delay_time > Duration::from_secs(0)
                        && delay_time > time_since_previous_acquire {
                    let additional_delay_required = delay_time - time_since_previous_acquire;

                    if additional_delay_required > Duration::from_secs(0) {
                        sleep(additional_delay_required);
                    }
                }

                self.state = ThrottleState::Initialized { previous_invocation: Instant::now() };
            },
            ThrottleState::Uninitialized => {
                self.state = ThrottleState::Initialized { previous_invocation: Instant::now() };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};
    use std::thread::sleep;
    use Throttle;

    #[test]
    fn it_works() {
        // simple throttle configured for 10 TPS
        let mut throttle = Throttle::new_tps_throttle(10.0);

        // the first one is free
        throttle.acquire(());

        let iteration_start = Instant::now();

        for _i in 0..10 {
            throttle.acquire(());
        }

        assert_eq!(iteration_start.elapsed().as_secs() == 1, true);
    }

    #[test]
    fn it_works_more_complicated() {
        let mut throttle = Throttle::new_variable_throttle(
            |arg: u64, _| Duration::from_millis(arg));

        let iteration_start = Instant::now();

        for i in 0..5 {
            throttle.acquire(i * 100);
        }

        assert_eq!(iteration_start.elapsed().as_secs() == 1, true);
    }

    // from a user-perspective, a delay of zero ought to mean "no delay," and I don't want to
    // worry about pesky panics trying to subtract durations!

    #[test]
    fn it_works_with_no_delay_at_all_tps() {
        let mut throttle = Throttle::new_tps_throttle(0.0);

        throttle.acquire(());
        throttle.acquire(());

        // no panic, no problem!
    }

    #[test]
    fn it_works_with_no_delay_at_all_variable() {
        let mut throttle = Throttle::new_variable_throttle(
            |_, _| Duration::from_millis(0));

        throttle.acquire(());
        throttle.acquire(());

        // no panic, no problem!
    }

    #[test]
    fn it_works_with_duration_smaller_than_already_elapsed_time() {
        // iterate every 10 ms
        let mut throttle = Throttle::new_tps_throttle(100.0);

        // the first one is free!
        throttle.acquire(());

        sleep(Duration::from_millis(20));

        throttle.acquire(());

        // no panic, no problem!
    }
}
