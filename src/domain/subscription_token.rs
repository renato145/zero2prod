use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::str::FromStr;

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

impl Default for SubscriptionToken {
    fn default() -> Self {
        Self::new()
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
        if s.len() == 25 {
            Ok(Self(s.into()))
        } else {
            Err(format!("Token {:?} is not in a valid format.", s))
        }
    }
}

impl<'de> serde::Deserialize<'de> for SubscriptionToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let token = String::deserialize(deserializer)?;
        token
            .parse::<SubscriptionToken>()
            .map_err(serde::de::Error::custom)
    }
}
