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
#[derive(Debug, Clone, Copy)]
pub struct Baseline {
    pub median: f64,
    pub sample_size: u32,
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
