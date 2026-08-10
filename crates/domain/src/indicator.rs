//! Indicators — display rule #1 ("a raw count is not information") as a type.

/// A non-negative count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Count(u32);

impl Count {
    pub fn new(n: u32) -> Self {
        Self(n)
    }
    pub fn get(self) -> u32 {
        self.0
    }
}

/// The median of comparable objects. Required, never optional.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Baseline {
    pub median: f64,
    pub sample_size: u32,
}

impl Baseline {
    /// Median of a set of comparable values. `None` for an empty sample — a
    /// baseline without comparables is not information (display rule #1).
    pub fn from_samples(samples: &[f64]) -> Option<Baseline> {
        if samples.is_empty() {
            return None;
        }
        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = sorted.len() / 2;
        let median = if sorted.len() % 2 == 1 {
            sorted[mid]
        } else {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        };
        Some(Baseline {
            median,
            sample_size: sorted.len() as u32,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Baseline;

    #[test]
    fn median_of_even_sample_averages_the_two_middle_values() {
        let b = Baseline::from_samples(&[537.0, 558.0]).expect("non-empty");
        assert_eq!(b.median, 547.5);
        assert_eq!(b.sample_size, 2);
    }

    #[test]
    fn median_of_odd_sample_is_the_middle_value() {
        let b = Baseline::from_samples(&[7.0, 90.0, 173.0]).expect("non-empty");
        assert_eq!(b.median, 90.0);
        assert_eq!(b.sample_size, 3);
    }

    #[test]
    fn median_of_single_sample_is_that_value() {
        let b = Baseline::from_samples(&[7.0]).expect("non-empty");
        assert_eq!(b.median, 7.0);
        assert_eq!(b.sample_size, 1);
    }

    #[test]
    fn empty_sample_has_no_baseline() {
        assert!(Baseline::from_samples(&[]).is_none());
    }
}

/// Link to the versioned methodology entry that defines a figure.
#[derive(Debug, Clone)]
pub struct MethodRef {
    pub id: String,
    pub version: u32,
}

/// A figure that, by construction, carries its baseline and its method.
/// You cannot model "a raw count with no baseline" — the compiler forbids it.
#[derive(Debug, Clone)]
pub struct Indicator {
    pub value: Count,
    pub baseline: Baseline,
    pub method: MethodRef,
}
