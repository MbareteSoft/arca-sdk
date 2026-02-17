use std::time::Duration;

/// Margin before token expiration to trigger refresh (in minutes)
pub const TOKEN_EXPIRY_MARGIN_MINUTES: i64 = 5;

/// HTTP request timeout
pub const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

/// HTTP connection timeout
pub const HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Login ticket generation time offset (minutes before now)
pub const LOGIN_TICKET_GEN_OFFSET_MINUTES: i64 = 5;

/// Login ticket expiration time offset (minutes after now)
pub const LOGIN_TICKET_EXP_OFFSET_MINUTES: i64 = 10;
