use chrono::{DateTime, TimeZone, Utc};

use crate::model::Tr;

/// Five hoards, each one a list of Tr.
/// We keep all Trs to use those dates as
/// a filter for the next search.
pub struct Hoard {
    a: [Vec<Tr>; 5],
}

impl Hoard {
    pub fn new() -> Hoard {
        Hoard {
            a: Default::default(),
        }
    }

    /// Gets the available hoard for an index.
    /// Index is 0..4 inclusive.
    pub fn checkout(&self, i: usize) -> (DateTime<Utc>, Vec<Tr>) {
        (
            match self.a[i].first() {
                Some(t) => t.realizada_em,
                None => Utc.timestamp_micros(0).unwrap(),
            },
            self.a[i].clone(),
        )
    }

    /// Adds new items to a hoard.
    /// The items are expected to be sorted by date,
    /// decreasing. They're also expected to not
    /// contain any items that are already in the hoard.
    /// The items in the argument list will be copied.
    pub fn stash(&mut self, i: usize, new: &Vec<Tr>) {
        let list = &mut self.a[i];
        list.splice(0..0, new.clone());
    }
}
