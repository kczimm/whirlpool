#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum State {
    Active,
    Idle,
    Finished,
}
