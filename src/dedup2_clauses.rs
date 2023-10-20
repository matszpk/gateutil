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
struct Dedup2Clause<T> {
    orig_index: T,
    extra_index: Option<T>,
    clause: Clause<T>,
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
            clause,
        }
    }

    // idea:
    // if literals choosen to deduplicate in other clause is already used
    // then they can be used if deduplicated literals contains all already used literals
    // in other clause with same reduction and other literal in other clause.
    // example: (l1 (used:1), l2 (used:1), l3)
    // idea: while replacing clause by subclause other clause, then
    // update both target clause and other clause hash entries.
    fn compare_and_dedup(
        &mut self,
        extra_clause_start: usize,
        clauses: &mut Vec<Dedup2Clause<T>>,
        val_map: &mut HashMap<SmartAllValues<T>, (T, usize)>,
    ) {
    }
}
