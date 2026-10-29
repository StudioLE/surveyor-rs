//! Types shared across domains.

mod author;
mod change;
mod channel_name;
mod commit;
mod commit_hash;
mod footer;
mod repo_name;
mod response;
mod short_commit_hash;
mod stage;

pub use author::*;
pub use change::*;
pub use channel_name::*;
pub use commit::*;
pub use commit_hash::*;
pub use footer::*;
pub use repo_name::*;
pub use response::*;
pub use short_commit_hash::*;
pub use stage::*;
