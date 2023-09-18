use gatesim::*;

use std::cmp::Ord;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter;

pub fn deduplicate<T: Clone + Copy + Ord + PartialEq + Eq>(circuit: Circuit<T>) -> Circuit<T>
where
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
    T: Hash,
{
    let mut gate_map = HashMap::<Gate<T>, T>::new();
    let mut new_gates: Vec<Gate<T>> = vec![];
    let input_len = usize::try_from(circuit.input_len()).unwrap();
    let mut gate_count = input_len;
    let mut output_map = Vec::from_iter(
        (0..input_len)
            .map(|x| T::try_from(x).unwrap())
            .chain(iter::repeat(T::default()).take(circuit.len())),
    );

    for (i, g) in circuit.gates().into_iter().enumerate() {
        let oi = input_len + i;
        let gi0 = output_map[usize::try_from(g.i0).unwrap()];
        let gi1 = output_map[usize::try_from(g.i1).unwrap()];
        // convert to new gate - ordered inputs if not nimpl.
        let (gi0, gi1) = if g.func != GateFunc::Nimpl && gi0 > gi1 {
            (gi1, gi0)
        } else {
            (gi0, gi1)
        };
        let newg = Gate {
            i0: gi0,
            i1: gi1,
            func: g.func,
        };
        if let Some(gindex) = gate_map.get(&newg) {
            // if found gate - then store its index into output_map
            output_map[oi] = *gindex;
        } else {
            // otherwise push to new_gates and to gate_map
            new_gates.push(newg);
            let gate_count_t = T::try_from(gate_count).unwrap();
            output_map[oi] = gate_count_t;
            gate_map.insert(newg, gate_count_t);
            gate_count += 1;
        }
    }

    let new_outputs = circuit
        .outputs()
        .into_iter()
        .map(|(x, n)| (output_map[usize::try_from(*x).unwrap()], *n))
        .collect::<Vec<_>>();

    Circuit::new(circuit.input_len(), new_gates, new_outputs).unwrap()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputEntry<T> {
    NewIndex(T),
    Value(bool),
}

// return circuit with assignment and mapping from older input to new input
// and output mapping from older output index to new output index or value
pub fn assign_to_circuit<T>(
    circuit: &Circuit<T>,
    inputs: impl IntoIterator<Item = (T, bool)>,
) -> (Circuit<T>, Vec<OutputEntry<T>>, Vec<OutputEntry<T>>)
where
    T: Default + Clone + Copy + PartialEq + Eq + PartialOrd + Ord,
    T: TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let input_len = usize::try_from(circuit.input_len()).unwrap();
    let len = circuit.len();

    let mut gate_map = vec![OutputEntry::Value(false); input_len + len];
    let mut rest_map = vec![true; input_len];
    // filter inputs
    for (g, v) in inputs.into_iter() {
        let g_u = usize::try_from(g).unwrap();
        rest_map[g_u] = false;
        gate_map[g_u] = OutputEntry::Value(v);
    }
    // generate output inputs
    let out_inputs = rest_map[0..input_len]
        .iter()
        .enumerate()
        .filter_map(|(i, x)| {
            if *x {
                Some(T::try_from(i).unwrap())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    // make to_new_rest_map - conversion to new outputs
    for (i, j) in out_inputs.iter().enumerate() {
        gate_map[usize::try_from(*j).unwrap()] = OutputEntry::NewIndex(T::try_from(i).unwrap());
    }
    let new_input_len = out_inputs.len();
    let mut new_gates: Vec<Gate<T>> = vec![];

    let mut oi = new_input_len;
    for (i, g) in circuit.gates().into_iter().enumerate() {
        let ii = input_len + i;
        let gi0 = usize::try_from(g.i0).unwrap();
        let gi1 = usize::try_from(g.i1).unwrap();
        match gate_map[gi0] {
            OutputEntry::NewIndex(ni0) => {
                match gate_map[gi1] {
                    OutputEntry::NewIndex(ni1) => {
                        gate_map[ii] = OutputEntry::NewIndex(T::try_from(oi).unwrap());
                        new_gates.push(Gate {
                            i0: ni0,
                            i1: ni1,
                            func: g.func,
                        });
                        oi += 1;
                    }
                    OutputEntry::Value(v1) => {
                        gate_map[ii] = OutputEntry::NewIndex(T::try_from(oi).unwrap());
                        let vv0 = g.eval_args(false, v1);
                        let vv1 = g.eval_args(true, v1);
                        new_gates.push(Gate {
                            i0: ni0,
                            i1: ni0,
                            func: if !vv0 && vv1 {
                                // x
                                GateFunc::And
                            } else if vv0 && !vv1 {
                                // !x
                                GateFunc::Nor
                            } else if !vv0 && !vv1 {
                                // 0
                                GateFunc::Nimpl
                            } else {
                                panic!("Unexpected case!");
                            },
                        });
                        oi += 1;
                    }
                }
            }
            OutputEntry::Value(v0) => {
                match gate_map[gi1] {
                    OutputEntry::NewIndex(ni1) => {
                        gate_map[ii] = OutputEntry::NewIndex(T::try_from(oi).unwrap());
                        let vv0 = g.eval_args(v0, false);
                        let vv1 = g.eval_args(v0, true);
                        new_gates.push(Gate {
                            i0: ni1,
                            i1: ni1,
                            func: if !vv0 && vv1 {
                                // x
                                GateFunc::And
                            } else if vv0 && !vv1 {
                                // !x
                                GateFunc::Nor
                            } else if !vv0 && !vv1 {
                                // 0
                                GateFunc::Nimpl
                            } else {
                                panic!("Unexpected case!");
                            },
                        });
                        oi += 1;
                    }
                    OutputEntry::Value(v1) => {
                        let out = g.eval_args(v0, v1);
                        gate_map[ii] = OutputEntry::Value(out);
                    }
                }
            }
        }
    }

    // outputs
    let mut new_outputs = vec![];
    let mut output_entries = vec![];
    for (o, n) in circuit.outputs().iter() {
        let o_u = usize::try_from(*o).unwrap();
        match gate_map[o_u] {
            OutputEntry::NewIndex(no) => {
                output_entries.push(OutputEntry::NewIndex(
                    T::try_from(new_outputs.len()).unwrap(),
                ));
                new_outputs.push((no, *n));
            }
            OutputEntry::Value(v) => {
                output_entries.push(OutputEntry::Value(v ^ n));
            }
        }
    }

    (
        Circuit::<T>::new(T::try_from(new_input_len).unwrap(), new_gates, new_outputs).unwrap(),
        gate_map[0..input_len].to_vec(),
        output_entries,
    )
}

// reduce chain clause - one-literal-clause - clause.
// check whether all usages of clause only in other clause.
// reduce clauses to zero or ones (constants).
// remove duplicated literals in clause.
// reduce literals in clause.
// deduplication based on evaluation (evaluated values for all input values) (optional).
// xor detection in and-or and or-and clause tree.
// find common parts of clauses to reuse more parts.

fn reduce_clauses<T>(clauses: &mut [(Clause<T>, bool)]) -> bool
where
    T: Clone + Copy + Ord + PartialEq + Eq,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let mut to_reduce_tree = false;
    for (clause, cs) in clauses {
        clause.literals.sort();
        let old_len = clause.len();
        match clause.kind {
            ClauseKind::And => {
                clause.literals.dedup();
                let mut pl = None;
                let mut zero = false;
                for (l, _) in &clause.literals {
                    if let Some(pl) = pl {
                        if pl == l {
                            // we have l and not(l) -> clause = 0
                            zero = true;
                            break;
                        }
                    }
                    pl = Some(l);
                }
                if zero {
                    // IMPORTANT: empty clauses treat as false.
                    clause.literals.clear();
                }
            }
            ClauseKind::Xor => {
                let mut pl = None;
                let mut new_literals = vec![];

                for (l, s) in &clause.literals {
                    if *s {
                        *cs = !*cs;
                    }
                    if let Some(pl) = pl {
                        if pl == l {
                            // we have l and l -> reduce 0
                            new_literals.pop();
                        } else {
                            new_literals.push((*l, false));
                        }
                    } else {
                        new_literals.push((*l, false));
                    }
                    pl = Some(l);
                }
                clause.literals = new_literals;
            }
        }
        if old_len >= 2 && clause.len() < 2 {
            to_reduce_tree = true;
        }
    }
    to_reduce_tree
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputEntryN<T> {
    NewIndex(T, bool),
    Value(bool),
}

// return true if further changes is needed.
// output_map includes circuit's inputs.
fn join_and_remove_clauses<T>(
    input_len: &mut usize,
    outputs: &[(T, bool)],
    clauses: &mut [(Clause<T>, bool)],
    output_map: &mut [OutputEntryN<T>],
) -> bool
where
    T: Clone + Copy + Ord + PartialEq + Eq,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let mut output_usages = vec![0; *input_len + clauses.len()];
    for (c, _) in clauses.iter() {
        for (l, _) in &c.literals {
            let l = usize::try_from(*l).unwrap();
            output_usages[l] += 1;
        }
    }
    for (o, _) in outputs.iter() {
        if let OutputEntryN::NewIndex(o, _) = output_map[usize::try_from(*o).unwrap()] {
            let o = usize::try_from(o).unwrap();
            output_usages[o] += 1;
        }
    }

    // generate orig_index_map - convert new indexes to old original indexes
    let orig_index_map_len = clauses.len() + *input_len;
    let mut orig_index_map = vec![0; orig_index_map_len];
    for (i, x) in output_map.iter().enumerate() {
        if let OutputEntryN::NewIndex(x, _) = x {
            orig_index_map[usize::try_from(*x).unwrap()] = i;
        }
    }

    let mut new_output_usages = vec![false; clauses.len()];

    // traversing and join clauses
    #[derive(Clone, Copy, Debug)]
    struct StackEntry {
        node: usize,
        way: usize,
        clause_id: Option<usize>,
    }
    let mut visited = vec![false; clauses.len()];
    
    //
    // traverse 1: resolve one literal clauses and resolve other clauses
    //
    for (o, _) in outputs.iter() {
        let o = usize::try_from(*o).unwrap();
        if o < *input_len {
            new_output_usages[o] = true;
            continue;
        }
        let o = match output_map[o] {
            OutputEntryN::NewIndex(o, _) => {
                let o = usize::try_from(o).unwrap();
                if o < *input_len {
                    continue;
                }
                o
            }
            OutputEntryN::Value(_) => continue,
        };
        let mut stack = Vec::<StackEntry>::new();
        stack.push(StackEntry {
            node: o - *input_len,
            way: 0,
            clause_id: None,
        });
        while !stack.is_empty() {
            let mut top = stack.last_mut().unwrap();
            let node_index = top.node;
            let (clause, clause_neg) = &mut clauses[node_index];

            if top.way == 0 {
                if !visited[node_index] {
                    visited[node_index] = true;
                } else {
                    stack.pop();
                    continue;
                }
            }
            if top.way < clause.literals.len() {
                top.way += 1;
                let l = usize::try_from(clause.literals[top.way].0).unwrap();
                if l >= *input_len {
                    stack.push(StackEntry {
                        node: l - *input_len,
                        way: 0,
                        clause_id: None,
                    });
                } else {
                    new_output_usages[l] = true;
                }
            } else {
                // resolve values and indexes for current clauses
                if clause.literals.len() == 0 {
                    // fill up by zero ^ neg
                    output_map[orig_index_map[*input_len + node_index]] =
                        OutputEntryN::Value(*clause_neg);
                } else if clause.literals.len() == 1 {
                    // propagate to output_map
                    let l = usize::try_from(clause.literals[0].0).unwrap();
                    match output_map[orig_index_map[l]] {
                        OutputEntryN::NewIndex(x, n1) => {
                            output_map[orig_index_map[*input_len + node_index]] =
                                OutputEntryN::NewIndex(x, n1 ^ clause.literals[0].1 ^ *clause_neg);
                        }
                        OutputEntryN::Value(v) => {
                            output_map[orig_index_map[*input_len + node_index]] =
                                OutputEntryN::Value(v ^ clause.literals[0].1 ^ *clause_neg);
                        }
                    }
                } else {
                    // resolve clause
                    let mut new_literals = vec![];
                    for (l, n) in &clause.literals {
                        let l_u = usize::try_from(*l).unwrap();
                        // TODO: handle clause merging!!
                        match output_map[l_u] {
                            OutputEntryN::NewIndex(l1, n1) => {
                                new_literals.push((l1, n ^ n1));
                            }
                            OutputEntryN::Value(v1) => {
                                let v = n ^ v1;
                                match clause.kind {
                                    ClauseKind::And => {
                                        if !v {
                                            new_literals.clear();
                                            break;
                                        }
                                    }
                                    ClauseKind::Xor => {
                                        if v {
                                            *clause_neg = !*clause_neg;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    clause.literals = new_literals;
                    new_output_usages[*input_len + node_index] = true;
                }
                stack.pop();
            }
        }
    }
    
    //
    // traverse 2: collect to parent clauses
    //
    visited.fill(false);
    for (o, _) in outputs.iter() {
        let o = usize::try_from(*o).unwrap();
        if o < *input_len {
            continue;
        }
        let o = match output_map[o] {
            OutputEntryN::NewIndex(o, _) => {
                let o = usize::try_from(o).unwrap();
                if o < *input_len {
                    continue;
                }
                o
            }
            OutputEntryN::Value(_) => continue,
        };
        let mut stack = Vec::<StackEntry>::new();
        stack.push(StackEntry {
            node: o - *input_len,
            way: 0,
            clause_id: None,
        });
        
        while !stack.is_empty() {
            let mut top = stack.last_mut().unwrap();
            let node_index = top.node;
            let (clause, clause_neg) = &mut clauses[node_index];

            if top.way == 0 {
                if !visited[node_index] {
                    visited[node_index] = true;
                } else {
                    stack.pop();
                    continue;
                }
            }
            if top.way < clause.literals.len() {
                top.way += 1;
                let l = usize::try_from(clause.literals[top.way].0).unwrap();
                if l >= *input_len {
                    stack.push(StackEntry {
                        node: l - *input_len,
                        way: 0,
                        clause_id: None,
                    });
                }
            } else {
                stack.pop();
            }
        }
    }
    false
}

// return optimized circuit, mapping to new inputs, mapping to new outputs
pub fn optimize_clause_circuit<T>(
    circuit: ClauseCircuit<T>,
) -> (ClauseCircuit<T>, Vec<Option<T>>, Vec<OutputEntry<T>>)
where
    T: Clone + Copy + Ord + PartialEq + Eq,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let mut clauses = circuit
        .clauses()
        .iter()
        .map(|x| (x.clone(), false))
        .collect::<Vec<_>>();

    let input_len = usize::try_from(circuit.input_len()).unwrap();
    let mut output_map = (0..input_len + clauses.len())
        .map(|x| OutputEntryN::NewIndex(T::try_from(x).unwrap(), false))
        .collect::<Vec<_>>();

    let mut first = true;
    let mut new_input_len = input_len;
    while !reduce_clauses(&mut clauses) || first {
        // join clauses and remove unnecessary clauses
        first = false;
        if !join_and_remove_clauses(
            &mut new_input_len,
            circuit.outputs(),
            &mut clauses,
            &mut output_map,
        ) {
            break;
        }
    }

    (
        ClauseCircuit::new(T::default(), vec![], vec![]).unwrap(),
        vec![],
        vec![],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reduce_clauses() {
        let mut clauses = [
            (
                Clause::new_and([(3, false), (0, false), (1, true), (3, false)]),
                false,
            ),
            (
                Clause::new_and([(3, true), (0, false), (1, true), (3, false)]),
                true,
            ),
            (
                Clause::new_and([(3, true), (3, true), (0, false), (1, true), (3, false)]),
                false,
            ),
            (
                Clause::new_and([(3, true), (0, false), (1, true), (3, true)]),
                false,
            ),
            (
                Clause::new_xor([(4, false), (3, false), (1, true), (2, false)]),
                false,
            ),
            (
                Clause::new_xor([(4, false), (2, false), (1, true), (2, false)]),
                true,
            ),
            (
                Clause::new_xor([(4, false), (2, false), (1, true), (2, true)]),
                true,
            ),
        ];
        assert!(reduce_clauses(&mut clauses));
        assert_eq!(
            [
                (Clause::new_and([(0, false), (1, true), (3, false)]), false),
                (Clause::new_and([]), true),
                (Clause::new_and([]), false),
                (Clause::new_and([(0, false), (1, true), (3, true)]), false),
                (
                    Clause::new_xor([(1, false), (2, false), (3, false), (4, false)]),
                    true
                ),
                (Clause::new_xor([(1, false), (4, false)]), false),
                (Clause::new_xor([(1, false), (4, false)]), true),
            ],
            clauses
        );

        // no changes
        let mut clauses = [
            (
                Clause::new_and([(3, false), (0, false), (1, true), (3, false)]),
                false,
            ),
            (
                Clause::new_xor([(4, false), (2, false), (1, true), (2, true)]),
                true,
            ),
        ];
        assert!(!reduce_clauses(&mut clauses));
        assert_eq!(
            [
                (Clause::new_and([(0, false), (1, true), (3, false)]), false),
                (Clause::new_xor([(1, false), (4, false)]), true),
            ],
            clauses
        );

        let mut clauses = [
            (
                Clause::new_and([(3, false), (0, false), (1, true), (3, false)]),
                false,
            ),
            (Clause::new_xor([(4, false), (2, false), (2, true)]), true),
        ];
        assert!(reduce_clauses(&mut clauses));
        assert_eq!(
            [
                (Clause::new_and([(0, false), (1, true), (3, false)]), false),
                (Clause::new_xor([(4, false)]), false),
            ],
            clauses
        );
    }
}
