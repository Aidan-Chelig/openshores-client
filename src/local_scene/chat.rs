use super::*;

impl Session {
    pub fn send_story_control(
        &mut self,
        control: crate::chat::StoryControl,
        timeout: Duration,
    ) -> io::Result<()> {
        if timeout.is_zero() || timeout > Duration::from_secs(10) {
            return Err(invalid("invalid story control timeout"));
        }
        let channel = self
            ._chat
            .as_mut()
            .ok_or_else(|| invalid("authenticated chat channel unavailable"))?;
        channel.deadline = Instant::now() + timeout;
        channel.continuation_deadline = None;
        write_frame(channel, &control.encode())
    }
    pub fn send_hail(&mut self, name: &str, timeout: Duration) -> io::Result<()> {
        let payload = crate::chat::Hail::encode(self.bootstrap.avatar_id, name)?;
        if timeout.is_zero() || timeout > Duration::from_secs(10) {
            return Err(invalid("invalid hail write timeout"));
        }
        let channel = self
            ._chat
            .as_mut()
            .ok_or_else(|| invalid("authenticated chat channel unavailable"))?;
        channel.deadline = Instant::now() + timeout;
        channel.continuation_deadline = None;
        write_frame(channel, &payload)
    }
    pub fn send_choice(&mut self, reply: crate::chat::Reply, timeout: Duration) -> io::Result<()> {
        let payload = reply.encode()?;
        if timeout.is_zero() || timeout > Duration::from_secs(10) {
            return Err(invalid("invalid choice write timeout"));
        }
        let channel = self
            ._chat
            .as_mut()
            .ok_or_else(|| invalid("authenticated chat channel unavailable"))?;
        channel.deadline = Instant::now() + timeout;
        channel.continuation_deadline = None;
        write_frame(channel, &payload)
    }
    pub fn send_message(
        &mut self,
        message: &crate::chat::Outgoing,
        timeout: Duration,
    ) -> io::Result<()> {
        let payload = message.encode(self.bootstrap.avatar_id)?;
        if timeout.is_zero() || timeout > Duration::from_secs(10) {
            return Err(invalid("invalid chat write timeout"));
        }
        let channel = self
            ._chat
            .as_mut()
            .ok_or_else(|| invalid("authenticated chat channel unavailable"))?;
        channel.deadline = Instant::now() + timeout;
        channel.continuation_deadline = None;
        write_frame(channel, &payload)
    }
    /// Poll only the authenticated chat socket. Partial frames are errors,
    /// never idle; payloads never enter the scene replay/capture.
    pub fn receive_chat(&mut self, timeout: Duration) -> io::Result<SceneRead> {
        if timeout.is_zero() || timeout > Duration::from_secs(1) {
            return Err(invalid(
                "chat receive timeout must be positive and at most one second",
            ));
        }
        let Some(channel) = &mut self._chat else {
            return Ok(SceneRead::Idle);
        };
        let deadline = Instant::now() + timeout;
        channel.deadline = deadline;
        channel.continuation_deadline = deadline.checked_add(Duration::from_millis(100));
        let before = channel.received;
        let result = read_frame(channel, 128 * 1024);
        if channel.received > 16 * 1024 * 1024 {
            return Err(invalid("chat session exceeds receive budget"));
        }
        match result {
            Ok(packet) => Ok(SceneRead::Packet(packet)),
            Err(error) if channel.received == before => match error.kind() {
                io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock => Ok(SceneRead::Idle),
                io::ErrorKind::UnexpectedEof => Ok(SceneRead::Closed),
                _ => Err(error),
            },
            Err(error) => Err(error),
        }
    }
}
