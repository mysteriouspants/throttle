/// A simple throttle, used for slowing down repeated code. Use this to avoid
/// drowning out downstream systems. For example, if I were reading the contents
/// of a file repeatedly (polling for data, perhaps), or calling an external
/// network resource, I could use a `Throttle` to slow that down to avoid
/// resource contention or browning out a downstream service.
///
/// ```rust
/// # #[macro_use]
/// # extern crate mysteriouspants_throttle;
/// # use std::time::{Duration, Instant};
/// # use mysteriouspants_throttle::Throttle;
///
/// # fn main() {
/// // create a new Throttle that rate limits to 10 TPS
/// let throttle = tps_throttle!(10.0);
///
/// let iteration_start = Instant::now();
///
/// // iterate eleven times, which at 10 TPS should take 1 second
/// for _i in 0..11 {
///   throttle.acquire(());
///   // do the needful
/// }
///
/// // prove that it did, in fact, take 1 second
/// assert_eq!(iteration_start.elapsed().as_secs() == 1, true);
/// # }
/// ```

use std::cell::Cell;
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::marker::PhantomData;

#[derive(Copy, Clone)]
enum ThrottleState {
    Uninitialized,
    Initialized {
        previous_invocation: Instant
    }
}

// A simple configurable throttle for slowing down code.
pub struct Throttle<TArg, TDelay: Fn(TArg, Duration) -> Duration> {
    delay_calculator: TDelay,
    state: Cell<ThrottleState>,
    delay_arg_type: PhantomData<TArg>
}

impl <TArg, TDelay: Fn(TArg, Duration) -> Duration> Throttle<TArg, TDelay> {
    /// Creates a new `Throttle` with a variable delay controlled by a closure.
    pub fn new(delay_calculator: TDelay) -> Throttle<TArg, TDelay> {
        return Throttle {
            delay_calculator,
            state: Cell::new(ThrottleState::Uninitialized),
            delay_arg_type: PhantomData
        };
    }

    /// Acquires the throttle, waiting (sleeping the current thread) until
    /// enough time has passed for the running code to be at or slower than
    /// the throttle allows.
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

// Creates a new `Throttle` tuned to a supplied constant TPS (Transactions Per Second).
#[macro_export]
macro_rules! tps_throttle {
    ( $tps:expr ) => {
        {
            Throttle::new(|_, _| Duration::from_millis(((1.0 / ($tps as f32)) * 1000.0) as u64))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};
    use Throttle;

    #[test]
    fn it_works() {
        // simple throttle configured for 10 TPS
        let throttle = tps_throttle!(10.0);

        let iteration_start = Instant::now();

        for _i in 0..11 {
            throttle.acquire(());
        }

        assert_eq!(iteration_start.elapsed().as_secs() == 1, true);
    }
}