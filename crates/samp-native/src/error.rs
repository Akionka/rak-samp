/// Failures specific to direct, profile-gated SA-MP native operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectClientError {
    InvalidArgument,
    NotReady,
    Busy,
    UnsupportedVersion,
    QueueFull,
}
