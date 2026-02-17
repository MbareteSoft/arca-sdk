use crate::constants::defaults::TOKEN_EXPIRY_MARGIN_MINUTES;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Access ticket returned by WSAA authentication
///
/// Implements Drop to ensure sensitive token and sign values
/// are zeroed from memory when the ticket is dropped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketAcceso {
    pub token: String,
    pub sign: String,
    pub expiration_time: String,
}

impl Drop for TicketAcceso {
    fn drop(&mut self) {
        self.token.zeroize();
        self.sign.zeroize();
    }
}

impl TicketAcceso {
    /// Check if the ticket is expired (using default margin)
    pub fn is_expired(&self) -> bool {
        self.is_expired_with_margin(TOKEN_EXPIRY_MARGIN_MINUTES)
    }

    /// Check if the ticket is expired with a custom margin
    pub fn is_expired_with_margin(&self, margin_minutes: i64) -> bool {
        if let Ok(exp) = chrono::DateTime::parse_from_rfc3339(&self.expiration_time) {
            let now = Utc::now();
            exp.with_timezone(&Utc) < (now + Duration::minutes(margin_minutes))
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_is_expired_with_future_time() {
        let future = (Utc::now() + Duration::hours(1)).to_rfc3339();
        let ta = TicketAcceso {
            token: "t".to_string(),
            sign: "s".to_string(),
            expiration_time: future,
        };
        assert!(!ta.is_expired());
    }

    #[test]
    fn test_ticket_is_expired_with_past_time() {
        let past = (Utc::now() - Duration::hours(1)).to_rfc3339();
        let ta = TicketAcceso {
            token: "t".to_string(),
            sign: "s".to_string(),
            expiration_time: past,
        };
        assert!(ta.is_expired());
    }

    #[test]
    fn test_ticket_is_expired_with_custom_margin() {
        // Ticket expires in 3 minutes
        let soon = (Utc::now() + Duration::minutes(3)).to_rfc3339();
        let ta = TicketAcceso {
            token: "t".to_string(),
            sign: "s".to_string(),
            expiration_time: soon,
        };

        // With 5 minute margin, should be considered expired
        assert!(ta.is_expired_with_margin(5));

        // With 1 minute margin, should NOT be expired
        assert!(!ta.is_expired_with_margin(1));
    }
}
