#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthState {
    Unknown,
    Healthy,
    Unhealthy,
}

#[derive(Debug, Clone)]
pub struct HealthMachine {
    healthy_threshold: u32,
    unhealthy_threshold: u32,
    state: HealthState,
    success_streak: u32,
    failure_streak: u32,
}

impl HealthMachine {
    pub fn new(healthy_threshold: u32, unhealthy_threshold: u32) -> Self {
        Self {
            healthy_threshold: healthy_threshold.max(1),
            unhealthy_threshold: unhealthy_threshold.max(1),
            state: HealthState::Unknown,
            success_streak: 0,
            failure_streak: 0,
        }
    }

    pub fn state(&self) -> HealthState {
        self.state
    }

    pub fn observe(&mut self, ok: bool) -> HealthState {
        if ok {
            self.success_streak = self.success_streak.saturating_add(1);
            self.failure_streak = 0;
            if self.success_streak >= self.healthy_threshold {
                self.state = HealthState::Healthy;
            }
        } else {
            self.failure_streak = self.failure_streak.saturating_add(1);
            self.success_streak = 0;
            if self.failure_streak >= self.unhealthy_threshold {
                self.state = HealthState::Unhealthy;
            }
        }
        self.state
    }

    pub fn reset(&mut self) {
        self.state = HealthState::Unknown;
        self.success_streak = 0;
        self.failure_streak = 0;
    }
}
