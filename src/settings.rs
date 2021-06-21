use serde::{Deserialize, Serialize};

// Describe the settings your policy expects when
// loaded by the policy server.
#[derive(Serialize, Deserialize, Default, Debug)]
pub(crate) struct Settings {
    #[serde(default)]
    pub error_message: String,
}

impl kubewarden::settings::Validatable for Settings {
    fn validate(&self) -> Result<(), String> {
        // TODO: perform settings validation if applies
        if self.error_message.is_empty() {
            return Err("The secret setting must have a value".to_string());
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
            error_message: String::from("There is a problem with the ingress that has been applied to the cluster, and it cannot be mutated to support Istio"),
        };

        assert!(settings.validate().is_ok());
        Ok(())
    }
}
