flat_mod! { mutex }

/// `!Send` and `!Sync` channels designed to send information between JavaScript contexts
pub mod channel;
pub mod abort;