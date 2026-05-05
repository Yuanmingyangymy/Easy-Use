use rand::{distributions::Alphanumeric, Rng};

pub const DEFAULT_TOKEN_LEN: usize = 32;

pub fn generate_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(DEFAULT_TOKEN_LEN)
        .map(char::from)
        .collect()
}

pub fn constant_time_eq(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }

    left.bytes()
        .zip(right.bytes())
        .fold(0u8, |accumulator, (a, b)| accumulator | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_token_has_expected_length() {
        assert_eq!(generate_token().len(), DEFAULT_TOKEN_LEN);
    }

    #[test]
    fn generated_tokens_are_not_reused_in_basic_sample() {
        let first = generate_token();
        let second = generate_token();
        assert_ne!(first, second);
    }

    #[test]
    fn token_compare_validates_equal_values() {
        let token = generate_token();
        assert!(constant_time_eq(&token, &token));
        assert!(!constant_time_eq(&token, "wrong-token"));
    }
}
