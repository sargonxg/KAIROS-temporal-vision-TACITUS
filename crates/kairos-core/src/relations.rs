use crate::temporal::TemporalInterval;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum AllenRelation {
    Before,
    After,
    Meets,
    MetBy,
    Overlaps,
    OverlappedBy,
    Starts,
    StartedBy,
    During,
    Contains,
    Finishes,
    FinishedBy,
    Equals,
}

impl AllenRelation {
    pub fn inverse(self) -> Self {
        use AllenRelation::*;
        match self {
            Before => After,
            After => Before,
            Meets => MetBy,
            MetBy => Meets,
            Overlaps => OverlappedBy,
            OverlappedBy => Overlaps,
            Starts => StartedBy,
            StartedBy => Starts,
            During => Contains,
            Contains => During,
            Finishes => FinishedBy,
            FinishedBy => Finishes,
            Equals => Equals,
        }
    }

    pub fn natural(self) -> &'static str {
        use AllenRelation::*;
        match self {
            Before => "before",
            After => "after",
            Meets => "meets",
            MetBy => "met by",
            Overlaps => "overlaps",
            OverlappedBy => "overlapped by",
            Starts => "starts",
            StartedBy => "started by",
            During => "during",
            Contains => "contains",
            Finishes => "finishes",
            FinishedBy => "finished by",
            Equals => "coincides with",
        }
    }
}

const TOL_SECS: i64 = 60 * 60 * 24;

pub fn compute(x: &TemporalInterval, y: &TemporalInterval) -> AllenRelation {
    use AllenRelation::*;
    let now = Utc::now();
    let xs = x.from.timestamp();
    let xe = x.to.unwrap_or(now).timestamp();
    let ys = y.from.timestamp();
    let ye = y.to.unwrap_or(now).timestamp();

    let approx = |a: i64, b: i64| (a - b).abs() <= TOL_SECS;

    if approx(xs, ys) && approx(xe, ye) {
        return Equals;
    }
    if approx(xs, ys) && xe < ye - TOL_SECS {
        return Starts;
    }
    if approx(xs, ys) && xe > ye + TOL_SECS {
        return StartedBy;
    }
    if approx(xe, ye) && xs > ys + TOL_SECS {
        return Finishes;
    }
    if approx(xe, ye) && xs < ys - TOL_SECS {
        return FinishedBy;
    }
    if approx(xe, ys) {
        return Meets;
    }
    if approx(ye, xs) {
        return MetBy;
    }
    if xe < ys {
        return Before;
    }
    if ye < xs {
        return After;
    }
    if xs > ys && xe < ye {
        return During;
    }
    if ys > xs && ye < xe {
        return Contains;
    }
    if xs < ys && xe < ye && xe > ys {
        return Overlaps;
    }
    if xs > ys && xe > ye && xs < ye {
        return OverlappedBy;
    }
    Overlaps
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpisodeRelation {
    pub from_episode: String,
    pub to_episode: String,
    pub relation: AllenRelation,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone};

    fn iv(from: &str, to: Option<&str>) -> TemporalInterval {
        let f = Utc.from_utc_datetime(
            &NaiveDate::parse_from_str(from, "%Y-%m-%d")
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        );
        let t = to.map(|s| {
            Utc.from_utc_datetime(
                &NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
            )
        });
        TemporalInterval {
            from: f,
            to: t,
            fuzziness_secs: 0,
        }
    }

    #[test]
    fn computes_core_relations() {
        assert_eq!(
            compute(
                &iv("2024-01-01", Some("2024-03-01")),
                &iv("2024-06-01", Some("2024-08-01"))
            ),
            AllenRelation::Before
        );
        assert_eq!(
            compute(
                &iv("2024-01-01", Some("2024-03-01")),
                &iv("2024-03-01", Some("2024-06-01"))
            ),
            AllenRelation::Meets
        );
        assert_eq!(
            compute(
                &iv("2024-02-01", Some("2024-04-01")),
                &iv("2024-01-01", Some("2024-12-31"))
            ),
            AllenRelation::During
        );
        assert_eq!(
            compute(
                &iv("2024-01-01", Some("2024-06-01")),
                &iv("2024-04-01", Some("2024-09-01"))
            ),
            AllenRelation::Overlaps
        );
        assert_eq!(
            compute(
                &iv("2024-01-01", Some("2024-06-01")),
                &iv("2024-01-01", Some("2024-06-01"))
            ),
            AllenRelation::Equals
        );
    }

    #[test]
    fn inverse_round_trip() {
        for rel in [
            AllenRelation::Before,
            AllenRelation::Meets,
            AllenRelation::During,
            AllenRelation::Starts,
            AllenRelation::Finishes,
            AllenRelation::Overlaps,
            AllenRelation::Equals,
        ] {
            assert_eq!(rel.inverse().inverse(), rel);
        }
    }
}
