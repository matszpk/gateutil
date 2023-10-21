use gatesim::*;

use std::cmp::Ord;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter;

mod join_clauses;
use join_clauses::*;
mod dedup_clauses;
use dedup_clauses::*;
mod dedup2_clauses;
use dedup2_clauses::*;
mod smart_bitmap;
use smart_bitmap::*;
mod utils;

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
                    clause.kind = ClauseKind::Xor; // empty Xor is false
                }
            }
            ClauseKind::Xor => {
                let mut pl = None;
                let mut new_literals = vec![];

                for (l, s) in &clause.literals {
                    *cs ^= s;
                    if let Some(xpl) = pl {
                        if xpl == l {
                            // we have l and l -> remove literal
                            new_literals.pop();
                            pl = None; // reset previous literal
                            continue;
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
        if old_len >= 1 && clause.len() < 2 {
            // return signal to next step if some clause have only 1 literal
            // or reduced to one or zero literals
            to_reduce_tree = true;
        }
    }
    to_reduce_tree
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
    //println!("OptStart");
    let mut clauses = circuit
        .clauses()
        .iter()
        .map(|x| (x.clone(), false))
        .collect::<Vec<_>>();

    let input_len = usize::try_from(circuit.input_len()).unwrap();
    let mut output_map = (0..input_len + clauses.len())
        .map(|x| OutputEntryN::NewIndex(T::try_from(x).unwrap(), false))
        .collect::<Vec<_>>();

    let mut oim_opt = None;
    let mut new_input_len = input_len;
    loop {
        let mut do_next = reduce_clauses(&mut clauses);
        //println!("OptXPhase0: {:?}", clauses);
        // join clauses and remove unnecessary clauses
        do_next |= join_and_remove_clauses(
            &mut new_input_len,
            &mut clauses,
            circuit.outputs(),
            &mut output_map,
            &mut oim_opt,
        );
        //println!("OptXPhase: {:?}", clauses);
        //println!("OptXPhaseMap: {:?}", output_map);
        if !do_next {
            reduce_clauses(&mut clauses);
            //println!("OptXPhaseF: {:?}", clauses);
            break;
        }
    }

    // generate new clauses
    let mut new_clauses = clauses
        .iter()
        .map(|(clause, _)| clause.clone())
        .collect::<Vec<_>>();
    for clause in &mut new_clauses {
        for (l, n) in &mut clause.literals {
            // resolve sign of literal
            let l = usize::try_from(*l).unwrap();
            if l >= new_input_len {
                *n ^= clauses[l - new_input_len].1;
            }
        }
    }

    // new inputs
    let new_inputs = output_map[0..input_len]
        .iter()
        .map(|om| {
            if let OutputEntryN::NewIndex(x, _) = om {
                Some(*x)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // new outputs and new outputs map
    let mut new_outputs = vec![];
    let mut new_outputs_map = vec![OutputEntry::Value(false); circuit.outputs().len()];
    for (i, (o, on)) in circuit.outputs().iter().enumerate() {
        match output_map[usize::try_from(*o).unwrap()] {
            OutputEntryN::NewIndex(x, n) => {
                let no_idx = T::try_from(new_outputs.len()).unwrap();
                new_outputs_map[i] = OutputEntry::NewIndex(no_idx);
                let x_u = usize::try_from(x).unwrap();
                if x_u >= new_input_len {
                    new_outputs.push((x, on ^ n ^ clauses[x_u - new_input_len].1));
                } else {
                    new_outputs.push((x, on ^ n));
                }
            }
            OutputEntryN::Value(v) => {
                new_outputs_map[i] = OutputEntry::Value(v ^ on);
            }
        }
    }

    (
        ClauseCircuit::new(
            T::try_from(new_input_len).unwrap(),
            new_clauses,
            new_outputs,
        )
        .unwrap(),
        new_inputs,
        new_outputs_map,
    )
}

pub fn assign_to_circuit_and_optimize<T>(
    circuit: &Circuit<T>,
    inputs: impl IntoIterator<Item = (T, bool)>,
    seq: bool,
) -> (Circuit<T>, Vec<OutputEntry<T>>, Vec<OutputEntry<T>>)
where
    T: Default + Clone + Copy + PartialEq + Eq + PartialOrd + Ord,
    T: TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let (circuit, input_map, output_map) = assign_to_circuit(circuit, inputs);
    let clause_circuit = ClauseCircuit::from(circuit);
    //println!("ClauseCircuit: {:?}", clause_circuit);
    let (opt_circuit, opt_input_map, opt_output_map) = optimize_clause_circuit(clause_circuit);
    let opt_circuit = if seq {
        Circuit::from_seq(opt_circuit)
    } else {
        Circuit::from(opt_circuit)
    };
    let out_input_map = join_input_entry_and_input_map(&input_map, &opt_input_map);
    let out_output_map = join_output_entry_map(&output_map, &opt_output_map);
    (opt_circuit, out_input_map, out_output_map)
}

// joins input/output maps from previous and next operation and returns joined in/out map.
pub fn join_input_entry_and_input_map<T>(
    input_map: &[OutputEntry<T>],
    opt_input_map: &[Option<T>],
) -> Vec<OutputEntry<T>>
where
    T: Clone + Copy,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let mut out_input_map = vec![OutputEntry::Value(false); input_map.len()];
    for (i, e) in input_map.into_iter().enumerate() {
        out_input_map[i] = match e {
            OutputEntry::NewIndex(x) => {
                let x = usize::try_from(*x).unwrap();
                match opt_input_map[x] {
                    Some(x) => OutputEntry::NewIndex(x),
                    None => OutputEntry::Value(false),
                }
            }
            OutputEntry::Value(v) => OutputEntry::Value(*v),
        };
    }
    out_input_map
}

// joins input/output maps from previous and next operation and returns joined in/out map.
pub fn join_input_map<T>(map: &[Option<T>], next_map: &[Option<T>]) -> Vec<Option<T>>
where
    T: Clone + Copy,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let mut out_map = vec![None; map.len()];
    for (i, e) in map.iter().enumerate() {
        out_map[i] = match e {
            Some(x) => next_map[usize::try_from(*x).unwrap()],
            None => None,
        };
    }
    out_map
}

// joins input/output maps from previous and next operation and returns joined in/out map.
pub fn join_output_entry_map<T>(
    map: &[OutputEntry<T>],
    next_map: &[OutputEntry<T>],
) -> Vec<OutputEntry<T>>
where
    T: Clone + Copy,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let mut out_map = vec![OutputEntry::Value(false); map.len()];
    for (i, e) in map.iter().enumerate() {
        out_map[i] = match e {
            OutputEntry::NewIndex(x) => next_map[usize::try_from(*x).unwrap()],
            OutputEntry::Value(v) => OutputEntry::Value(*v),
        };
    }
    out_map
}

// deduplicate clauses and clause literals
// return new circuit and boolean value.
// if some possible literal duplicates then returns true, otherwise return false
pub fn deduplicate_clause_circuit<T>(circuit: ClauseCircuit<T>) -> (ClauseCircuit<T>, bool)
where
    T: Clone + Copy + Ord + PartialEq + Eq + Hash,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    // assertion for sorted and deduplicated clauses
    assert!(circuit.clauses().iter().all(|c| {
        let mut prev = None;
        for l in &c.literals {
            if let Some(p) = prev {
                if !(p < l) {
                    return false;
                }
            }
            prev = Some(l);
        }
        true
    }));
    let input_len = usize::try_from(circuit.input_len()).unwrap();
    let mut extra_clause_index = input_len + circuit.len();
    // return (clause_index, Option<extra_clause_index>, clause) vector
    let mut and_clauses = circuit
        .clauses()
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            if c.kind == ClauseKind::And {
                Some(DedupClause {
                    orig_index: T::try_from(input_len + i).unwrap(),
                    extra_index: None,
                    clause: c.clone(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let mut and_trans_tbl = deduplicate_clauses(&mut and_clauses);
    and_clauses.sort();
    let and_clauses_need_optim = if !and_trans_tbl.is_empty() {
        // check whether clauses need optimizations
        check_if_clauses_need_optimization_and_fix(&mut and_clauses)
    } else {
        false
    };

    // return (clause_index, Option<extra_clause_index>, clause) vector
    let mut xor_clauses = circuit
        .clauses()
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            if c.kind == ClauseKind::Xor {
                Some(DedupClause {
                    orig_index: T::try_from(input_len + i).unwrap(),
                    extra_index: None,
                    clause: c.clone(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let mut xor_trans_tbl = deduplicate_clauses(&mut xor_clauses);
    xor_clauses.sort();
    let xor_clauses_need_optim = if !xor_trans_tbl.is_empty() {
        check_if_clauses_need_optimization_and_fix(&mut xor_clauses)
    } else {
        false
    };

    if !and_clauses_need_optim {
        // because deduplicate_literal_clauses and deduplicate_literal_clauses_0
        // move some literals from old clauses into new clauses then
        // checking 1-literal clauses and duplicated literals is not needed.
        deduplicate_literal_clauses_0(
            &mut extra_clause_index,
            &mut and_clauses,
            &mut and_trans_tbl,
        );
        deduplicate_literal_clauses(
            &mut extra_clause_index,
            &mut and_clauses,
            &mut and_trans_tbl,
        );
    }

    if !xor_clauses_need_optim {
        // because deduplicate_literal_clauses and deduplicate_literal_clauses_0
        // move some literals from old clauses into new clauses then
        // checking 1-literal clauses and duplicated literals is not needed.
        deduplicate_literal_clauses_0(
            &mut extra_clause_index,
            &mut xor_clauses,
            &mut xor_trans_tbl,
        );
        deduplicate_literal_clauses(
            &mut extra_clause_index,
            &mut xor_clauses,
            &mut xor_trans_tbl,
        );
    }

    // println!("AndTransTbl: {:?}", and_trans_tbl);
    // println!("XorTransTbl: {:?}", xor_trans_tbl);
    translate_clauses(&mut and_clauses, &xor_trans_tbl, false);
    translate_clauses(&mut xor_clauses, &and_trans_tbl, false);

    (
        join_deduplicates_to_clause_circuit(
            input_len,
            extra_clause_index,
            and_clauses,
            and_trans_tbl,
            xor_clauses,
            xor_trans_tbl,
            circuit.outputs(),
        ),
        and_clauses_need_optim | xor_clauses_need_optim,
    )
}

// return optimized circuit, mapping to new inputs, mapping to new outputs
pub fn optimize_and_dedup_clause_circuit<T>(
    circuit: ClauseCircuit<T>,
) -> (ClauseCircuit<T>, Vec<Option<T>>, Vec<OutputEntry<T>>)
where
    T: Clone + Copy + Ord + PartialEq + Eq + Hash,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let (mut new_circuit, mut input_map, mut output_map) = optimize_clause_circuit(circuit);
    let mut continue_dedup = true;
    while continue_dedup {
        (new_circuit, continue_dedup) = deduplicate_clause_circuit(new_circuit);
        let (next_circuit, next_input_map, next_output_map) = optimize_clause_circuit(new_circuit);
        input_map = join_input_map(&input_map, &next_input_map);
        output_map = join_output_entry_map(&output_map, &next_output_map);
        new_circuit = next_circuit;
    }
    (new_circuit, input_map, output_map)
}

pub fn assign_to_circuit_optimize_and_dedup<T>(
    circuit: &Circuit<T>,
    inputs: impl IntoIterator<Item = (T, bool)>,
    seq: bool,
) -> (Circuit<T>, Vec<OutputEntry<T>>, Vec<OutputEntry<T>>)
where
    T: Default + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash,
    T: TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let (circuit, input_map, output_map) = assign_to_circuit(circuit, inputs);
    let clause_circuit = ClauseCircuit::from(circuit);
    //println!("ClauseCircuit: {:?}", clause_circuit);
    let (opt_circuit, opt_input_map, opt_output_map) =
        optimize_and_dedup_clause_circuit(clause_circuit);
    let opt_circuit = if seq {
        Circuit::from_seq(opt_circuit)
    } else {
        Circuit::from(opt_circuit)
    };
    let out_input_map = join_input_entry_and_input_map(&input_map, &opt_input_map);
    let out_output_map = join_output_entry_map(&output_map, &opt_output_map);
    (opt_circuit, out_input_map, out_output_map)
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
                (Clause::new_xor([]), true),
                (Clause::new_xor([]), false),
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

        let mut clauses = [
            (Clause::new_and([(3, false)]), false),
            (Clause::new_xor([(4, false), (2, false), (3, true)]), true),
        ];
        assert!(reduce_clauses(&mut clauses));
        assert_eq!(
            [
                (Clause::new_and([(3, false)]), false),
                (Clause::new_xor([(2, false), (3, false), (4, false)]), false),
            ],
            clauses
        );

        for i in 1..8 {
            let mut clauses = [(
                Clause::new_and(std::iter::repeat((2, false)).take(i)),
                false,
            )];
            assert!(reduce_clauses(&mut clauses));
            assert_eq!([(Clause::new_and([(2, false)]), false),], clauses);
        }

        for i in 1..8 {
            let mut clauses = [(
                Clause::new_xor(std::iter::repeat((2, false)).take(i)),
                false,
            )];
            assert!(reduce_clauses(&mut clauses));
            assert_eq!(
                [(
                    if (i & 1) != 0 {
                        Clause::new_xor([(2, false)])
                    } else {
                        Clause::new_xor([])
                    },
                    false
                )],
                clauses
            );
        }
    }
}
