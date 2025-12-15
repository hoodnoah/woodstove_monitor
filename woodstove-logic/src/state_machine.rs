use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BurnState {
    Idle,
    Startup,
    ActiveBurn,
    Coaling,
    Overheat,
}

pub struct StoveStateMachine {
    state: BurnState,
    state_set_time: Instant,
    last_update_time: Instant,
    last_temp: Option<f32>,
    roc_alpha: f32,
    rate_of_change: Option<f32>,
    // TODO: What fields do you need?
    // - current state
    // - temperature history for rate-of-change
    // - when did we enter current state
    // - configuration thresholds?
}

impl StoveStateMachine {
    pub fn new() -> Self {
        StoveStateMachine::new_roc(None)
    }

    pub fn new_roc(roc_alpha: Option<f32>) -> Self {
        let now = Instant::now();
        StoveStateMachine {
            state: BurnState::Idle, // assume idle initially
            state_set_time: now,
            last_update_time: now,
            last_temp: None,
            roc_alpha: roc_alpha.unwrap_or(0.3),
            rate_of_change: None,
        }
    }

    pub fn update(&mut self, current_temp_f: f32) {
        let now = Instant::now();
        let last = self.last_update_time;
        self.last_update_time = now;
        match self.last_temp {
            None => {
                // first update from uninitialized
                self.last_temp = Some(current_temp_f);
            }
            Some(last_temp) => {
                let instantaneous = (current_temp_f - last_temp) / (now - last).as_secs_f32();
                match self.rate_of_change {
                    None => {
                        // second update
                        self.rate_of_change = Some(instantaneous);
                    }
                    Some(last_roc) => {
                        // calculate exponential moving average
                        let new =
                            self.roc_alpha * instantaneous + (1.0 - self.roc_alpha) * last_roc;
                        self.rate_of_change = Some(new);
                    }
                }
            }
        }

        // let elapsed = now - self.last_update_time;
        // TODO:
        // - Calculate rate of change
        // - Determine state transitions
        // - Update internal state
        // todo!()
    }

    pub fn current_state(&self) -> BurnState {
        self.state
    }

    pub fn time_in_state(&self) -> Duration {
        Instant::now() - self.state_set_time
    }

    pub fn should_reload(&self) -> bool {
        // TODO: Logic for when reload is needed
        // Based on state + temperature + time?
        todo!()
    }
}

// tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    const FLOAT_TOLERANCE: f32 = 0.1;

    #[test]
    fn new_state_machine_starts_idle() {
        let sm = StoveStateMachine::new();
        assert_eq!(sm.current_state(), BurnState::Idle);
    }

    #[test]
    fn new_state_machine_has_no_temp() {
        let sm = StoveStateMachine::new();
        assert_eq!(sm.last_temp, None)
    }

    #[test]
    fn new_state_machine_has_no_rate_of_change() {
        let sm = StoveStateMachine::new();
        assert_eq!(sm.rate_of_change, None)
    }

    #[test]
    fn first_update_sets_updated_time() {
        let mut sm = StoveStateMachine::new();
        let original = sm.last_update_time;
        sleep(Duration::from_millis(500));
        sm.update(20.0);

        assert_ne!(sm.last_update_time, original);
    }

    #[test]
    fn first_update_sets_temp() {
        let mut sm = StoveStateMachine::new();
        let original = sm.last_temp;
        sm.update(20.0);

        assert_ne!(original, sm.last_temp);
        assert_eq!(sm.last_temp, Some(20.0));
    }

    #[test]
    fn first_update_does_not_set_rate() {
        let mut sm = StoveStateMachine::new();
        let sleep_time = Duration::from_secs(2);
        sleep(sleep_time);

        sm.update(20.0);

        assert!(sm.rate_of_change.is_none());
    }

    #[test]
    fn second_update_sets_roc() {
        let mut sm = StoveStateMachine::new();
        sm.update(0.0);

        sleep(Duration::from_secs(2));
        sm.update(20.0);

        assert!(sm.rate_of_change.is_some());
    }

    #[test]
    fn second_update_sets_correct_roc() {
        let mut sm = StoveStateMachine::new();
        sm.update(0.0);

        sleep(Duration::from_secs(2));
        sm.update(20.0);

        let expected = 20.0 / 2.0;
        let actual = sm.rate_of_change.unwrap();
        let variance = (expected - actual).abs();
        assert!(
            variance < FLOAT_TOLERANCE,
            "Rate of change mismatch: expected {}, got {} (var {})",
            expected,
            actual,
            variance,
        )
    }

    #[test]
    fn third_update_updates_roc() {
        let mut sm = StoveStateMachine::new();
        sm.update(0.0);
        sleep(Duration::from_secs(2));
        sm.update(20.0);
        sleep(Duration::from_secs(2));
        sm.update(10.0);

        let expected_rate = 0.3 * (10.0 / 2.0) + 0.7 * (20.0 / 2.0);
        let actual = sm.rate_of_change.unwrap();
        let variance = expected_rate - actual;

        assert!(
            variance < FLOAT_TOLERANCE,
            "expected variance between expected_rate ({}) and actual rate ({}) to be less than {}, received {}.",
            expected_rate,
            actual,
            FLOAT_TOLERANCE,
            variance
        );
    }
}
