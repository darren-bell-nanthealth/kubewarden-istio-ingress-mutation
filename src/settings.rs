use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// Describe the settings your policy expects when
// loaded by the policy server.
#[derive(Serialize, Deserialize, Default, Debug)]
pub(crate) struct Settings {
    #[serde(default)]
    pub secret: String,
}

impl kubewarden::settings::Validatable for Settings {
    fn validate(&self) -> Result<(), String> {
        // TODO: perform settings validation if applies
        if self.secret.is_empty() {
            return Err(format!("The secret setting must have a value"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use kubewarden_policy_sdk::settings::Validatable;

    #[test]
    fn validate_settings() -> Result<(), ()> {
        let settings = Settings {
            secret : String::from("ExternalSecret")
        };

        assert!(settings.validate().is_ok());
        Ok(())
    }
}
