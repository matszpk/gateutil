use gatesim::*;

use std::cmp::Ord;
use std::collections::HashMap;
use std::fmt::Debug;

use crate::smart_bitmap::*;

enum DedupRef<T> {
    Clause(T),
    ClausePart { clause_index: T, subclause_index: T },
}

#[derive(Clone)]
enum Dedup2ClauseBody<T> {
    Original {
        // if clause empty - then has been replaced
        clause: Clause<T>,
        // list of option: index - index of literal in clause
        // value - Some(v) - new or old clause index
        used_literals: Vec<Option<T>>,
    },
    Replaced {
        new_index: T,
    },
}

#[derive(Clone)]
struct Dedup2Clause<T> {
    orig_index: T,
    extra_index: Option<T>,
    body: Dedup2ClauseBody<T>,
}

impl<T> Dedup2Clause<T>
where
    T: Default + Clone + Copy + Debug + Ord + PartialEq + Eq,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    fn new(orig_index: T, extra_index: Option<T>, clause: Clause<T>) -> Self {
        let clause_len = clause.len();
        Self {
            orig_index,
            extra_index,
            body: Dedup2ClauseBody::Original {
                clause,
                used_literals: vec![None; clause_len],
            },
        }
    }

    // idea:
    // if literals choosen to deduplicate in other clause is already used
    // then they can be used if deduplicated literals contains all already used literals
    // in other clause with same reduction and other literal in other clause.
    // example: (l1 (used:1), l2 (used:1), l3)
    fn compare_and_dedup(&mut self, val_map: &mut HashMap<SmartAllValues<T>, T>) {}
}
