use crate::{errors::AppError, security::token};

use super::now_epoch_secs;

#[derive(Clone, Debug)]
pub struct Session {
    pub token: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub port: u16,
}

impl Session {
    pub fn new(ttl_secs: u64, port: u16) -> Self {
        let created_at = now_epoch_secs();
        Self {
            token: token::generate_token(),
            created_at,
            expires_at: created_at.saturating_add(ttl_secs),
            port,
        }
    }

    pub fn for_test(token: String, created_at: u64, expires_at: u64, port: u16) -> Self {
        Self {
            token,
            created_at,
            expires_at,
            port,
        }
    }

    pub fn validate(&self, candidate: &str) -> Result<(), AppError> {
        if now_epoch_secs() > self.expires_at {
            return Err(AppError::SessionExpired);
        }

        if !token::constant_time_eq(&self.token, candidate) {
            return Err(AppError::TokenInvalid);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_active_token() {
        let now = now_epoch_secs();
        let session = Session::for_test("abc".to_string(), now, now + 60, 0);

        assert!(session.validate("abc").is_ok());
        assert!(matches!(session.validate("wrong"), Err(AppError::TokenInvalid)));
    }

    #[test]
    fn rejects_expired_token() {
        let now = now_epoch_secs();
        let session = Session::for_test("abc".to_string(), now - 20, now - 10, 0);

        assert!(matches!(session.validate("abc"), Err(AppError::SessionExpired)));
    }
}
