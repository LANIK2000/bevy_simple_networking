use std::{ops::RangeInclusive, time::Duration};

/// Default number of ticks executed per minute.
const DEFAULT_TICKS_PER_MIN: u32 = 100;

/// Resource to track the state of the network separately from the ECS frame timings
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NetworkTime {
    /// The current frame
    frame_number: u32,
    /// Accumulated duration since last frame
    elapsed_duration: Duration,
    /// Duration per frame
    per_frame_duration: Duration,
    /// Determines how often we send messages. i.e. "Every N frames" where N is message_send_rate
    message_send_rate: u8,
    /// Number of frames behind the ecs. This will usually be 0 or 1 if the ECS system
    /// is keeping up
    frame_lag: u32,
}

impl NetworkTime {
    /// Returns the frame numbers needed to be run this game frame.
    #[must_use]
    pub fn network_frames_to_run(&self) -> RangeInclusive<u32> {
        (self.frame_number + 1 - self.frame_lag)..=self.frame_number
    }

    /// Determines whether or not to send a message in the current frame based on the
    /// `message_send_rate`
    #[must_use]
    pub fn should_send_message_now(&self) -> bool {
        self.should_send_message(self.frame_number)
    }

    /// Determines whether or not to send a message based on the `message_send_rate`
    #[must_use]
    pub fn should_send_message(&self, frame: u32) -> bool {
        frame % u32::from(self.message_send_rate) == 0
    }

    /// Bumps the frame number
    pub fn increment_frame_number(&mut self) {
        self.frame_number += 1;
        self.elapsed_duration -= self.per_frame_duration;
        self.frame_lag += 1;
    }

    /// Resets the frame lag
    pub fn reset_frame_lag(&mut self) {
        self.frame_lag = 0;
    }

    /// Increases the `elapsed_duration` by the given duration
    pub fn update_elapsed(&mut self, duration: Duration) {
        self.elapsed_duration += duration;
    }

    /// Returns the current frame number
    #[must_use]
    pub fn frame_number(&self) -> u32 {
        self.frame_number
    }

    /// Sets the frame number to the given frame number. This is useful when synchronizing frames
    /// with a server for example.
    pub fn set_frame_number(&mut self, new_frame: u32) {
        self.frame_number = new_frame;
    }

    /// Returns the total duration since the last frame
    #[must_use]
    pub fn elapsed_duration(&self) -> Duration {
        self.elapsed_duration
    }

    /// Returns the duration between each frame. This number is calculated when a frame rate
    /// is set
    #[must_use]
    pub fn per_frame_duration(&self) -> Duration {
        self.per_frame_duration
    }

    /// Returns the rate at which messages should be sent over the network.
    /// i.e. 'every N frames' where N is `message_send_rate`.
    #[must_use]
    pub fn message_send_rate(&self) -> u8 {
        self.message_send_rate
    }

    /// Returns the number of network frames which the current frame is behind. This will usually be 0 or 1 if the ECS system
    /// is keeping up.
    #[must_use]
    pub fn frame_lag(&self) -> u32 {
        self.frame_lag
    }

    /// Sets the rate at which the network progresses. Specified in hertz (frames/second).
    pub fn set_network_frame_rate(&mut self, new_rate: u32) {
        self.per_frame_duration = Duration::from_secs(1) / new_rate;
    }

    /// Set the rate which messages are sent. Specified as 'every N frames' where N is `new_rate`.
    pub fn set_message_send_rate(&mut self, new_rate: u8) {
        self.message_send_rate = new_rate;
    }
}

impl Default for NetworkTime {
    fn default() -> Self {
        Self {
            frame_number: 0,
            elapsed_duration: Duration::from_secs(0),
            // Default to 100 frames / minute
            per_frame_duration: Duration::from_secs(60) / DEFAULT_TICKS_PER_MIN,
            // Default to sending a message with every frame
            message_send_rate: 1,
            // Default the lag to run so systems have a chance to run on frame 0
            frame_lag: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_calculated_properties_and_getters() {
        let mut time = NetworkTime::default();
        time.set_network_frame_rate(20);
        assert_eq!(time.frame_number(), 0);
        assert_eq!(time.frame_lag(), 1);
        assert_eq!(time.message_send_rate(), 1);
        assert_eq!(time.per_frame_duration(), Duration::from_millis(50));
        assert_eq!(time.elapsed_duration(), Duration::from_millis(0));
    }

    #[test]
    fn test_message_send_rate_should_send_every_2_frames() {
        let mut time = NetworkTime::default();
        time.set_message_send_rate(2);

        for i in 1..100 {
            // every second frame (even) should return true
            if i % 2 == 0 {
                assert_eq!(time.should_send_message(i), true);
            } else {
                assert_eq!(time.should_send_message(i), false);
            }
        }
    }

    #[test]
    fn test_elapsed_duration_gets_updated() {
        let mut time = NetworkTime::default();

        let elapsed_time = Duration::from_millis(500);
        time.update_elapsed(elapsed_time);

        assert_eq!(time.elapsed_duration(), elapsed_time);
    }
}
