use gatesim::*;

use crate::smart_bitmap::*;

enum Dedup2ClauseBody<T> {
    Literals {
        // if clause empty - then has been replaced
        clause: Clause<T>,
        // list of option: index - index of literal in clause
        // value - Some(v) - new or old clause index
        used_literals: Vec<Option<T>>,
    },
    Clause {
        new_index: T,
    }
}

struct Dedup2Clause<T> {
    orig_index: usize,
    body: Dedup2ClauseBody<T>
}
