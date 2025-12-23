use std::time::{Duration, Instant};

pub struct StoveConfig {
    pub idle_threshold: f32,
    pub active_threshold: f32,
    pub active_exit_threshold: f32,
    pub overheat_threshold: f32,

    // Rate thresholds
    pub rising_fast_rate: f32,
    pub falling_rate: f32,
    pub stable_rate: f32,
}

impl Default for StoveConfig {
    fn default() -> Self {
        Self {
            idle_threshold: 150.0,
            active_threshold: 400.0,
            active_exit_threshold: 350.0,
            overheat_threshold: 700.0,
            rising_fast_rate: 5.0 / 60.0,
            falling_rate: -3.0 / 60.0,
            stable_rate: 1.2 / 60.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BurnState {
    Idle,
    Startup,
    ActiveBurn,
    Coaling,
    Overheat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TempChangeType {
    RisingFast,
    Falling,
    Stable,
}

pub struct StoveStateMachine {
    config: StoveConfig,
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
            config: StoveConfig::default(),
            state: BurnState::Idle, // assume idle initially
            state_set_time: now,
            last_update_time: now,
            last_temp: None,
            roc_alpha: roc_alpha.unwrap_or(0.3),
            rate_of_change: None,
        }
    }

    pub fn update(&mut self, current_temp_f: f32) -> bool {
        // Update rate of change
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
                self.last_temp = Some(current_temp_f);
            }
        }

        // determine new state
        let rate_type = self.classify_rate();
        let new_state = self.classify_state(rate_type, current_temp_f);

        if new_state != self.state {
            self.state_set_time = Instant::now();
            self.state = new_state;
            return true;
        }
        return false;
    }

    pub fn current_state(&self) -> BurnState {
        self.state
    }

    pub fn time_in_state(&self) -> Duration {
        Instant::now() - self.state_set_time
    }

    pub fn should_reload(&self) -> bool {
        match self.state {
            BurnState::Coaling => {
                self.last_temp.unwrap_or(0.0) < 300.0
                    || self.time_in_state() > Duration::from_secs(30 * 60) // 30 mins
            }
            _ => false,
        }
    }

    // determines if a given rate of change is rising, stable, falling
    fn classify_rate(&self) -> TempChangeType {
        match self.rate_of_change {
            None => TempChangeType::Stable,
            Some(r) => {
                if r > self.config.rising_fast_rate {
                    TempChangeType::RisingFast
                } else if r < self.config.falling_rate {
                    TempChangeType::Falling
                } else {
                    TempChangeType::Stable
                }
            }
        }
    }

    fn classify_state(&self, roc_class: TempChangeType, temp: f32) -> BurnState {
        if temp > self.config.overheat_threshold {
            return BurnState::Overheat;
        }

        match self.state {
            // Idle can transition only to startup or itself
            BurnState::Idle => {
                if temp > self.config.idle_threshold && roc_class == TempChangeType::RisingFast {
                    BurnState::Startup
                } else {
                    BurnState::Idle
                }
            }

            // Startup can go to active burn (normal),
            // coaling (going out),
            // or remain
            BurnState::Startup => {
                if temp > self.config.active_threshold {
                    BurnState::ActiveBurn
                } else if roc_class == TempChangeType::Falling {
                    BurnState::Coaling
                } else {
                    BurnState::Startup
                }
            }

            // ActiveBurn can go to coaling (dying),
            // or remain activeBurn (overheat handled above)
            BurnState::ActiveBurn => {
                if temp < self.config.active_exit_threshold && roc_class == TempChangeType::Falling
                {
                    BurnState::Coaling
                } else {
                    BurnState::ActiveBurn
                }
            }

            // Coaling can become idle (died),
            // startup (reloaded),
            // or stay the same
            BurnState::Coaling => {
                if temp < self.config.idle_threshold && roc_class == TempChangeType::Stable {
                    BurnState::Idle
                } else if roc_class == TempChangeType::RisingFast {
                    BurnState::Startup
                } else {
                    BurnState::Coaling
                }
            }

            // Overheat can decrease to active,
            // or remain overheat
            BurnState::Overheat => {
                if temp < self.config.active_threshold {
                    // drop to 400f before recovering
                    BurnState::ActiveBurn
                } else {
                    BurnState::Overheat
                }
            }
        }
    }
}

// tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    const FLOAT_TOLERANCE: f32 = 0.1;

    fn sm_idle(
        sm: &mut StoveStateMachine,
        delta: Duration,
        rate: Option<f32>,
        last_temp: Option<f32>,
    ) {
        let now = Instant::now();
        let before = now.checked_sub(delta);

        sm.state = BurnState::Idle;
        sm.last_update_time = before.unwrap();
        sm.last_temp = last_temp;
        sm.rate_of_change = rate;
    }

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
        sm_idle(&mut sm, Duration::from_secs(2), None, Some(0.0)); // simulated first update
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
        sm_idle(&mut sm, Duration::from_secs(2), Some(20.0), Some(0.0));

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

    #[test]
    fn remains_idle_under_active_threshold() {
        let mut sm = StoveStateMachine::new();
        // set idle
        sm.state = BurnState::Idle;
        sm.last_update_time = Instant::now();
        sm.last_temp = Some(80.0);
        sm.rate_of_change = Some(5.0 / 60.0); // 5 deg/min
        sleep(Duration::from_secs(2));

        let threshold = sm.config.idle_threshold;
        let new_temp = threshold * 0.9;

        sm.update(new_temp);

        assert!(
            sm.state == BurnState::Idle,
            "expected {:?}, received {:?}",
            BurnState::Idle,
            sm.state
        );
    }

    #[test]
    fn transitions_to_startup() {
        let mut sm = StoveStateMachine::new();
        sm_idle(
            &mut sm,
            Duration::from_secs(2),
            Some(5.0 / 60.0),
            Some(64.5),
        );

        sm.update(215.0);

        assert_eq!(sm.state, BurnState::Startup);
    }

    #[test]
    fn transitions_to_burn() {
        let mut sm = StoveStateMachine::new();
        sm_idle(
            &mut sm,
            Duration::from_secs(20),
            Some(6.0 / 60.0),
            Some(300.0),
        );
        sm.state = BurnState::Startup;

        sm.update(401.0);

        assert_eq!(sm.state, BurnState::ActiveBurn);
    }

    #[test]
    fn transitions_to_overheat() {
        let mut sm = StoveStateMachine::new();
        sm_idle(
            &mut sm,
            Duration::from_secs(20),
            Some(6.0 / 60.0),
            Some(600.0),
        );
        sm.state = BurnState::ActiveBurn;

        sm.update(813.45);

        assert_eq!(sm.state, BurnState::Overheat);
    }

    #[test]
    fn transitions_back_to_active_burn_from_overheat() {
        let mut sm = StoveStateMachine::new();
        sm_idle(
            &mut sm,
            Duration::from_secs(20),
            Some(6.0 / 60.0),
            Some(813.45),
        );
        sm.state = BurnState::Overheat;

        sm.update(399.0);

        assert_eq!(sm.state, BurnState::ActiveBurn);
    }

    #[test]
    fn transitions_to_coaling_from_active() {
        let mut sm = StoveStateMachine::new();
        sm_idle(
            &mut sm,
            Duration::from_secs(20),
            Some(6.0 / 60.0),
            Some(401.0),
        );
        sm.state = BurnState::ActiveBurn;

        sm.update(320.0);

        assert_eq!(sm.state, BurnState::Coaling);
    }
}
