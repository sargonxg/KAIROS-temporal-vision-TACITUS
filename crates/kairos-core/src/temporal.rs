use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidityInterval {
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
}

impl ValidityInterval {
    pub fn closed(from: DateTime<Utc>, to: DateTime<Utc>) -> Self {
        Self {
            valid_from: Some(from),
            valid_to: Some(to),
        }
    }

    pub fn from(from: DateTime<Utc>) -> Self {
        Self {
            valid_from: Some(from),
            valid_to: None,
        }
    }

    pub fn instant(at: DateTime<Utc>) -> Self {
        Self::closed(at, at + Duration::seconds(1))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransactionInterval {
    pub recorded_from: DateTime<Utc>,
    pub recorded_to: Option<DateTime<Utc>>,
}

impl TransactionInterval {
    pub fn now() -> Self {
        Self {
            recorded_from: Utc::now(),
            recorded_to: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bitemporal<T> {
    pub value: T,
    pub valid: ValidityInterval,
    pub txn: TransactionInterval,
    pub provenance: Option<String>,
    pub confidence: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TemporalInterval {
    pub from: DateTime<Utc>,
    pub to: Option<DateTime<Utc>>,
    pub fuzziness_secs: u32,
}

impl TemporalInterval {
    pub fn point(at: DateTime<Utc>) -> Self {
        Self {
            from: at,
            to: Some(at + Duration::seconds(1)),
            fuzziness_secs: 0,
        }
    }

    pub fn span(&self) -> Duration {
        self.to.unwrap_or_else(Utc::now) - self.from
    }
}
