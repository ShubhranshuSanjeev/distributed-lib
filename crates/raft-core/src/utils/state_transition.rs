/// State transition result
/// Represents the result of a state transition, either successful or failed.
pub enum STResult<NewState, OriginalState> {
    Ok(NewState),
    Aborted(OriginalState),
}
