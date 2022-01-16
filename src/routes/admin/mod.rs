mod dashboard;
mod logout;
mod middleware;
mod newsletter;
mod password;

pub use dashboard::admin_dashboard;
pub use logout::log_out;
pub use middleware::CheckLogin;
pub use newsletter::*;
pub use password::*;
