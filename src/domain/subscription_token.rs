use std::str::FromStr;

use rand::{distributions::Alphanumeric, thread_rng, Rng};

pub struct SubscriptionToken(String);

impl SubscriptionToken {
    /// Generate a random 25-characters-long case-sensitive subscription token.
    pub fn new() -> Self {
        let mut rng = thread_rng();
        let token = std::iter::repeat_with(|| rng.sample(Alphanumeric))
            .map(char::from)
            .take(25)
            .collect();
        Self(token)
    }
}

impl AsRef<str> for SubscriptionToken {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl FromStr for SubscriptionToken {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!();
    }
}

impl<'de> serde::Deserialize<'de> for SubscriptionToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let token = String::deserialize(deserializer)?;
        Self::parse
    }
}
