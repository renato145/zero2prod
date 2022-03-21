mod dashboard;
mod delivery_process;
mod logout;
mod newsletter;
mod not_found;
mod password;

pub use dashboard::admin_dashboard;
pub use delivery_process::delivery_process;
pub use logout::log_out;
pub use newsletter::*;
pub use not_found::not_found;
pub use password::*;
