use studio_protocol::CoreErrorEnvelope;

/// Stable public error categories shared by every Studio operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    InvalidInput,
    Unauthorized,
    Conflict,
    Unavailable,
    Unsupported,
    Internal,
}

impl ErrorCategory {
    pub const ALL: [Self; 6] = [
        Self::InvalidInput,
        Self::Unauthorized,
        Self::Conflict,
        Self::Unavailable,
        Self::Unsupported,
        Self::Internal,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid-input",
            Self::Unauthorized => "unauthorized",
            Self::Conflict => "conflict",
            Self::Unavailable => "unavailable",
            Self::Unsupported => "unsupported",
            Self::Internal => "internal",
        }
    }

    const fn public_message(self) -> &'static str {
        match self {
            Self::InvalidInput => "The request is invalid.",
            Self::Unauthorized => "The operation is not authorized.",
            Self::Conflict => "The operation conflicts with newer state.",
            Self::Unavailable => "The requested service is unavailable.",
            Self::Unsupported => "The operation is not supported.",
            Self::Internal => "The operation failed internally.",
        }
    }

    const fn retryable(self) -> bool {
        matches!(self, Self::Unavailable)
    }
}

/// Internal failure wrapper whose only serializable form is its redacted public envelope.
#[derive(Debug, Clone, PartialEq)]
pub struct CoreError {
    category: ErrorCategory,
    public: Box<CoreErrorEnvelope>,
}

impl CoreError {
    /// Creates a public failure while deliberately dropping private diagnostic text.
    pub fn new(
        category: ErrorCategory,
        request_id: String,
        operation_id: Option<String>,
        code: &'static str,
        private_detail: impl AsRef<str>,
    ) -> Self {
        let _ = private_detail.as_ref();
        Self {
            category,
            public: Box::new(CoreErrorEnvelope {
                category: category.as_str().to_owned(),
                code: code.to_owned(),
                message: category.public_message().to_owned(),
                operation_id,
                request_id,
                retryable: category.retryable(),
            }),
        }
    }

    pub const fn category(&self) -> ErrorCategory {
        self.category
    }

    pub fn public(&self) -> &CoreErrorEnvelope {
        self.public.as_ref()
    }

    pub fn into_public(self) -> CoreErrorEnvelope {
        *self.public
    }
}

#[cfg(test)]
mod tests {
    use super::{CoreError, ErrorCategory};

    #[test]
    fn every_category_has_a_stable_redacted_envelope() {
        for category in ErrorCategory::ALL {
            let error = CoreError::new(
                category,
                "req_categories".to_owned(),
                Some("op_categories".to_owned()),
                "core.test.failure",
                "Bearer super-secret-token",
            );
            let serialized = serde_json::to_string(error.public())
                .expect("public error envelope must serialize");

            assert_eq!(error.public().category, category.as_str());
            assert_eq!(error.public().code, "core.test.failure");
            assert!(!error.public().message.contains("secret"));
            assert!(!serialized.contains("super-secret-token"));
            assert_eq!(
                error.public().retryable,
                category == ErrorCategory::Unavailable
            );
        }
    }
}
