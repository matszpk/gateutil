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
mod utils;

// TODO: add optimization that uses database of circuits (firstly with 1 output).

pub fn translate_inputs<T, U>(circuit: Circuit<T>, trans: &[U]) -> Circuit<T>
where
    T: Clone + Copy + Ord + PartialEq + Eq,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
    U: Clone + Copy,
    T: TryFrom<U>,
    <T as TryFrom<U>>::Error: Debug,
{
    let input_len_t = circuit.input_len();
    let input_len = usize::try_from(input_len_t).unwrap();
    assert_eq!(input_len, trans.len());
    let gates = circuit
        .gates()
        .into_iter()
        .map(|g| {
            let t0 = if g.i0 < input_len_t {
                T::try_from(trans[usize::try_from(g.i0).unwrap()]).unwrap()
            } else {
                g.i0
            };
            let t1 = if g.i1 < input_len_t {
                T::try_from(trans[usize::try_from(g.i1).unwrap()]).unwrap()
            } else {
                g.i1
            };
            Gate {
                i0: t0,
                i1: t1,
                func: g.func,
            }
        })
        .collect::<Vec<_>>();
    let outputs = circuit
        .outputs()
        .into_iter()
        .map(|(i, n)| {
            let t = if *i < input_len_t {
                T::try_from(trans[usize::try_from(*i).unwrap()]).unwrap()
            } else {
                *i
            };
            (t, *n)
        })
        .collect::<Vec<_>>();
    Circuit::new(input_len_t, gates, outputs).unwrap()
}

pub fn reverse_trans<T>(trans: impl IntoIterator<Item = T>) -> Vec<T>
where
    T: Clone + Copy,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let mut out = vec![];
    for (i, idx) in trans.into_iter().enumerate() {
        let idx = usize::try_from(idx).unwrap();
        if idx >= out.len() {
            out.resize(idx + 1, T::default());
        }
        out[idx] = T::try_from(i).unwrap();
    }
    out
}

pub fn translate_inputs_rev<T, U>(
    circuit: Circuit<T>,
    trans: impl IntoIterator<Item = U>,
) -> Circuit<T>
where
    T: Clone + Copy + Ord + PartialEq + Eq,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
    U: Clone + Copy,
    T: TryFrom<U>,
    <T as TryFrom<U>>::Error: Debug,
    U: Default + TryFrom<usize>,
    <U as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<U>,
    <usize as TryFrom<U>>::Error: Debug,
{
    let out = reverse_trans(trans);
    translate_inputs(circuit, &out)
}

pub fn translate_outputs<T, U>(circuit: Circuit<T>, trans: &[U]) -> Circuit<T>
where
    T: Clone + Copy + Ord + PartialEq + Eq + Default,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
    U: Clone + Copy,
    usize: TryFrom<U>,
    <usize as TryFrom<U>>::Error: Debug,
{
    let output_len = circuit.outputs().len();
    assert_eq!(output_len, trans.len());
    let outputs = circuit.outputs();
    let new_outputs = trans
        .into_iter()
        .map(|x| outputs[usize::try_from(*x).unwrap()])
        .collect::<Vec<_>>();
    Circuit::<T>::new(
        circuit.input_len(),
        circuit.gates().into_iter().cloned(),
        new_outputs,
    )
    .unwrap()
}

pub fn translate_outputs_rev<T, U>(
    circuit: Circuit<T>,
    trans: impl IntoIterator<Item = U>,
) -> Circuit<T>
where
    T: Clone + Copy + Ord + PartialEq + Eq + Default,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
    U: Clone + Copy + Default,
    usize: TryFrom<U>,
    <usize as TryFrom<U>>::Error: Debug,
    U: TryFrom<usize>,
    <U as TryFrom<usize>>::Error: Debug,
{
    let out = reverse_trans(trans);
    translate_outputs(circuit, &out)
}

// TODO: add routines to join, split and separate subcircuit

pub fn negate_inputs<T>(circuit: Circuit<T>, to_neg: impl IntoIterator<Item = T>) -> Circuit<T>
where
    T: Clone + Copy + Ord + PartialEq + Eq,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let input_len_t = circuit.input_len();
    let input_len = usize::try_from(input_len_t).unwrap();
    let len = circuit.len();
    let mut negs = vec![false; input_len + len];
    for t in to_neg {
        assert!(t < input_len_t);
        negs[usize::try_from(t).unwrap()] = true;
    }
    let gates = circuit
        .gates()
        .into_iter()
        .enumerate()
        .map(|(i, g)| {
            let gi0 = usize::try_from(g.i0).unwrap();
            let gi1 = usize::try_from(g.i1).unwrap();
            let (f_neg0, f_neg1) = match g.func {
                GateFunc::And => (false, false),
                GateFunc::Nor => (true, true),
                GateFunc::Nimpl => (false, true),
                GateFunc::Xor => (false, false),
            };
            let neg0 = negs[gi0] ^ f_neg0;
            let neg1 = negs[gi1] ^ f_neg1;
            match g.func {
                GateFunc::And | GateFunc::Nor | GateFunc::Nimpl => {
                    if neg0 {
                        if neg1 {
                            Gate {
                                i0: g.i0,
                                i1: g.i1,
                                func: GateFunc::Nor,
                            }
                        } else {
                            Gate {
                                i0: g.i1,
                                i1: g.i0,
                                func: GateFunc::Nimpl,
                            }
                        }
                    } else {
                        if neg1 {
                            Gate {
                                i0: g.i0,
                                i1: g.i1,
                                func: GateFunc::Nimpl,
                            }
                        } else {
                            Gate {
                                i0: g.i0,
                                i1: g.i1,
                                func: GateFunc::And,
                            }
                        }
                    }
                }
                GateFunc::Xor => {
                    negs[input_len + i] ^= neg0 ^ neg1;
                    Gate {
                        i0: g.i0,
                        i1: g.i1,
                        func: GateFunc::Xor,
                    }
                }
            }
        })
        .collect::<Vec<_>>();
    let outputs = circuit
        .outputs()
        .into_iter()
        .map(|(o, n)| (*o, n ^ negs[usize::try_from(*o).unwrap()]))
        .collect::<Vec<_>>();
    Circuit::new(input_len_t, gates, outputs).unwrap()
}

// structure seq: iterator over:
// tuple.0 - circuit to join
// tuple.1 - from_first: key - input index for next circuit
//               (value, neg) - option of index of output from some previous circuit
//               and negation for this input
// index of output from some previous circuit - just index of output from list of
// all outputs from all circuits ordered from first to last circuit.
pub fn join_circuits_seq<T>(
    seq: impl IntoIterator<Item = (Circuit<T>, impl IntoIterator<Item = Option<(T, bool)>>)>,
    last: Circuit<T>,
) -> Circuit<T>
where
    T: Clone + Copy + Ord + PartialEq + Eq + Debug,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let seq = seq
        .into_iter()
        .map(|(c, t)| (c, t.into_iter().collect::<Vec<_>>()))
        .collect::<Vec<_>>();
    if seq.is_empty() {
        return last;
    }
    let input1_len_t = seq.first().unwrap().0.input_len();
    let input1_len = usize::try_from(input1_len_t).unwrap();
    // check whether length of from_firsts are equal to input length of next circuit
    assert!(seq
        .iter()
        .enumerate()
        .all(|(i, (_, t))| if i + 1 < seq.len() {
            t.len() == usize::try_from(seq[i + 1].0.input_len()).unwrap()
        } else {
            t.len() == usize::try_from(last.input_len()).unwrap()
        }));
    let input_len = input1_len
        + seq
            .iter()
            .map(|(_, t)| t.iter().filter(|x| x.is_none()).count())
            .sum::<usize>();
    let total_input_len = seq
        .iter()
        .map(|(c, _)| usize::try_from(c.input_len()).unwrap())
        .sum::<usize>();
    let total_gate_num = seq.iter().map(|(c, _)| c.len()).sum::<usize>();
    // generate used_outputs for all circuits
    let total_output_num =
        seq.iter().map(|(c, _)| c.outputs().len()).sum::<usize>() + last.outputs().len();
    let used_outputs = {
        let mut used_outputs = vec![false; total_output_num];
        let mut output_count = 0;
        for (c, t) in &seq {
            output_count += c.outputs().len();
            for (o, _) in t.iter().filter_map(|x| *x) {
                let o = usize::try_from(o).unwrap();
                assert!(o < output_count);
                used_outputs[o] = true;
            }
        }
        used_outputs
    };
    // input_trans - translation map for all inputs
    let mut input_trans = (0..input1_len)
        .map(|x| T::try_from(x).unwrap())
        .collect::<Vec<_>>();
    input_trans.reserve(total_input_len - input1_len);
    // total outputs from first circuit to last
    let mut outputs = Vec::<(T, bool)>::with_capacity(total_output_num);
    let mut gates = Vec::with_capacity(total_gate_num);
    let mut input_index = 0;
    let mut gate_index = 0;
    let mut out_input_index = 0;
    for i in 0..seq.len() {
        let circuit1 = &seq[i].0;
        let circuit2 = if i + 1 < seq.len() {
            &seq[i + 1].0
        } else {
            &last
        };
        let from_first = &seq[i].1;

        let input2_len_t = circuit2.input_len();
        let input2_len = usize::try_from(input2_len_t).unwrap();
        if i == 0 {
            outputs.extend(circuit1.outputs().iter().map(|(xt, n)| {
                let x = usize::try_from(*xt).unwrap();
                let x = if x >= input1_len {
                    T::try_from(x - input1_len + input_len).unwrap()
                } else {
                    *xt
                };
                (x, *n)
            }));

            gates.extend(circuit1.gates().iter().map(|g| {
                let gi0 = if g.i0 >= input1_len_t {
                    T::try_from(usize::try_from(g.i0).unwrap() - input1_len + input_len).unwrap()
                } else {
                    g.i0
                };
                let gi1 = if g.i1 >= input1_len_t {
                    T::try_from(usize::try_from(g.i1).unwrap() - input1_len + input_len).unwrap()
                } else {
                    g.i1
                };
                Gate {
                    i0: gi0,
                    i1: gi1,
                    func: g.func,
                }
            }));
            gate_index = gates.len();
            input_index += input1_len;
            out_input_index += input1_len;
        }
        let mut unused_input2_count = 0;
        let joined_input2_map = from_first
            .into_iter()
            .map(|idx| {
                if let Some((idx, n)) = idx {
                    let idxu = usize::try_from(*idx).unwrap();
                    // assign output index from circuit1 to used input from circuit2
                    (idxu, true, *n)
                } else {
                    let idx = unused_input2_count;
                    unused_input2_count += 1;
                    // assign unused index to unused input from circuit2
                    (idx, false, false)
                }
            })
            .collect::<Vec<_>>();
        // negations to inputs from circuit2
        let negs = joined_input2_map
            .iter()
            .enumerate()
            .filter_map(|(i, (idx, joined, neg))| {
                if *joined && (outputs[*idx].1 ^ neg) {
                    Some(T::try_from(i).unwrap())
                } else {
                    None
                }
            });
        let circuit2 = negate_inputs(circuit2.clone(), negs);
        // make input2_map - to translate inputs from circuit2
        input_trans.extend(joined_input2_map.into_iter().map(|(idx, joined, _)| {
            if joined {
                outputs[idx].0
            } else {
                T::try_from(out_input_index + idx).unwrap()
            }
        }));
        outputs.extend(circuit2.outputs().iter().map(|(x, n)| {
            let x = usize::try_from(*x).unwrap();
            let x = if x >= input2_len {
                T::try_from(x - input2_len + gate_index + input_len).unwrap()
            } else {
                input_trans[x + input_index]
            };
            (x, *n)
        }));
        gates.extend(circuit2.gates().iter().map(|g| {
            let gi0 = if g.i0 >= input2_len_t {
                T::try_from(usize::try_from(g.i0).unwrap() - input2_len + input_len + gate_index)
                    .unwrap()
            } else {
                let iin = usize::try_from(g.i0).unwrap();
                input_trans[iin + input_index]
            };
            let gi1 = if g.i1 >= input2_len_t {
                T::try_from(usize::try_from(g.i1).unwrap() - input2_len + input_len + gate_index)
                    .unwrap()
            } else {
                let iin = usize::try_from(g.i1).unwrap();
                input_trans[iin + input_index]
            };
            Gate {
                i0: gi0,
                i1: gi1,
                func: g.func,
            }
        }));
        gate_index = gates.len();
        input_index += input2_len;
        out_input_index += unused_input2_count;
    }
    outputs = outputs
        .into_iter()
        .enumerate()
        .filter_map(|(i, x)| if !used_outputs[i] { Some(x) } else { None })
        .collect::<Vec<_>>();

    Circuit::new(T::try_from(input_len).unwrap(), gates, outputs).unwrap()
}

// INFO: from_first - index - input index for circuit2,
//                    value - option of output index for circuit1
pub fn join_two_circuits<T>(
    circuit1: Circuit<T>,
    from_first: impl IntoIterator<Item = Option<(T, bool)>>,
    circuit2: Circuit<T>,
) -> Circuit<T>
where
    T: Clone + Copy + Ord + PartialEq + Eq + Debug,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    join_circuits_seq([(circuit1, from_first)], circuit2)
}

/// Deduplicates gates in circuit. It finds duplicates by comparing gate and its inputs.
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

/// Output entry to store assignment of value (for output and input).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputEntry<T> {
    NewIndex(T),
    Value(bool),
}

/// Returns circuit with assignment and mapping from older input to new input
/// and output mapping from older output index to new output index or value.
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

// DEBUG
// fn dump_clauses<T>(input_len: usize, clauses: &[Clause<T>], outputs: &[(T, bool)])
// where
//     T: Clone + Copy + Ord + PartialEq + Eq,
//     T: Default + TryFrom<usize>,
//     <T as TryFrom<usize>>::Error: Debug,
//     usize: TryFrom<T>,
//     <usize as TryFrom<T>>::Error: Debug,
// {
//     println!("Dump ClauseCircuit data:");
//     println!("  InputLen: {}", input_len);
//     println!("  Clauses:");
//     for (i, c) in clauses.iter().enumerate() {
//         println!(
//             "    {}: {} {:?}",
//             input_len + i,
//             c.kind,
//             c.literals
//                 .iter()
//                 .map(|(l, n)| (usize::try_from(*l).unwrap(), *n))
//                 .collect::<Vec<_>>()
//         );
//     }
//     println!(
//         "  Outputs: {:?}",
//         outputs
//             .iter()
//             .map(|(l, n)| (usize::try_from(*l).unwrap(), *n))
//             .collect::<Vec<_>>()
//     );
// }
//
// fn dump_join_and_remove_clauses_output<T>(
//     input_len: &usize,
//     clauses: &Vec<(Clause<T>, bool)>,
//     output_map: &[OutputEntryN<T>],
//     oim_opt: &Option<Vec<usize>>,
// ) where
//     T: Clone + Copy + Ord + PartialEq + Eq,
//     T: Default + TryFrom<usize>,
//     <T as TryFrom<usize>>::Error: Debug,
//     usize: TryFrom<T>,
//     <usize as TryFrom<T>>::Error: Debug,
// {
//     println!("Dump JNR data:");
//     println!("  InputLen: {}", input_len);
//     println!("  Clauses:");
//     for (i, (c, n)) in clauses.iter().enumerate() {
//         println!(
//             "    {}: {} {:?} {}",
//             input_len + i,
//             c.kind,
//             c.literals
//                 .iter()
//                 .map(|(l, n)| (usize::try_from(*l).unwrap(), *n))
//                 .collect::<Vec<_>>(),
//             *n
//         );
//     }
//     println!("  OutputMap:");
//     for (i, oe) in output_map
//         .iter()
//         .map(|oe| match oe {
//             OutputEntryN::NewIndex(v, n) => {
//                 OutputEntryN::NewIndex(usize::try_from(*v).unwrap(), *n)
//             }
//             OutputEntryN::Value(v, on) => OutputEntryN::Value(*v, *on),
//         })
//         .enumerate()
//     {
//         println!("    {}: {:?}", i, oe);
//     }
//     println!("  OIMOpt:");
//     if let Some(oim_opt) = oim_opt.as_ref() {
//         for (i, idx) in oim_opt.iter().enumerate() {
//             println!("    {}: {}", i, idx);
//         }
//     }
// }
//
// fn dump_nclauses<T>(input_len: &usize, clauses: &Vec<(Clause<T>, bool)>)
// where
//     T: Clone + Copy + Ord + PartialEq + Eq,
//     T: Default + TryFrom<usize>,
//     <T as TryFrom<usize>>::Error: Debug,
//     usize: TryFrom<T>,
//     <usize as TryFrom<T>>::Error: Debug,
// {
//     println!("Dump NClauses:");
//     println!("  InputLen: {}", input_len);
//     println!("  Clauses:");
//     for (i, (c, n)) in clauses.iter().enumerate() {
//         println!(
//             "    {}: {} {:?} {}",
//             input_len + i,
//             c.kind,
//             c.literals
//                 .iter()
//                 .map(|(l, n)| (usize::try_from(*l).unwrap(), *n))
//                 .collect::<Vec<_>>(),
//             *n
//         );
//     }
// }
// DEBUG

/// Return optimized circuit, mapping to new inputs, mapping to new outputs.
/// It optimize circuit by joining clauses with same type and resolving duplicates
/// of literals in clauses and resolving values from that clauses.
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
    // println!("OptStart");
    // DEBUG
    // dump_clauses(
    //     usize::try_from(circuit.input_len()).unwrap(),
    //     circuit.clauses(),
    //     circuit.outputs(),
    // );
    // const JOIN_REDUCE_ITER_NUM: u32 = 100;
    // let mut jnr_iter = 0;
    // !DEBUG
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
    let mut repeat = 0;
    loop {
        let mut do_next = reduce_clauses(&mut clauses);
        //println!("OptXPhase0: {:?}", clauses);
        // DEBUG
        // println!("join_iter: {}", jnr_iter);
        // dump_nclauses(&new_input_len, &clauses);
        // DEBUG
        // join clauses and remove unnecessary clauses
        let old_clause_len = clauses.len();
        do_next |= join_and_remove_clauses(
            &mut new_input_len,
            &mut clauses,
            circuit.outputs(),
            &mut output_map,
            &mut oim_opt,
        );
        // DEBUG
        // dump_join_and_remove_clauses_output(&new_input_len, &clauses, &output_map, &oim_opt);
        // DEBUG
        if old_clause_len == clauses.len() {
            if repeat == 10 {
                do_next = false;
            }
            repeat += 1;
        } else {
            repeat = 0;
        }
        // DEBUG
        // jnr_iter += 1;
        // if jnr_iter == JOIN_REDUCE_ITER_NUM {
        //     do_next = false;
        // }
        // !DEBUG
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
            OutputEntryN::Value(v, _) => {
                new_outputs_map[i] = OutputEntry::Value(v ^ on);
            }
        }
    }

    // let do_fix = new_clauses.iter().any(|x| x.len() == 1);
    let do_fix = new_clauses.iter().any(|x| x.len() <= 1);
    if do_fix {
        println!("OptimizeFIX");
        // for c in new_clauses.iter_mut() {
        //     if c.len() == 1 {
        //         c.kind = ClauseKind::And;
        //         c.literals.push(c.literals[0]);
        //     }
        // }
        // DEBUG
        // dump_clauses(new_input_len, &new_clauses, &new_outputs);
        // !DEBUG
        for c in new_clauses.iter_mut() {
            if c.len() == 1 {
                c.kind = ClauseKind::And;
                c.literals.push(c.literals[0]);
            } else if c.len() == 0 {
                c.literals = vec![(T::default(), false), (T::default(), true)];
                if c.kind == ClauseKind::And {
                    c.kind = ClauseKind::Xor;
                } else {
                    c.kind = ClauseKind::And;
                }
            }
        }
        let (c, ni, no) = (
            ClauseCircuit::new(
                T::try_from(new_input_len).unwrap(),
                new_clauses,
                new_outputs,
            )
            .unwrap(),
            new_inputs,
            new_outputs_map,
        );
        // println!("OptEnd");
        let (newc, newni, newno) = optimize_clause_circuit(c);
        let out_input_map = join_input_map(&ni, &newni);
        let out_output_map = join_output_entry_map(&no, &newno);
        (newc, out_input_map, out_output_map)
        // (c, ni, no)
    } else {
        // println!("OptEnd");
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
}

/// Assigns and optimize clause circuit. See to optimize_clause_circuit and assign_to_circuit.
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

/// Joins input/output maps from previous and next operation and returns joined in/out map.
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

/// Joins input/output maps from previous and next operation and returns joined in/out map.
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

/// Joins input/output maps from previous and next operation and returns joined in/out map.
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

// DEBUG
// pub(crate) fn dump_dedup_clauses<T>(clauses: &[DedupClause<T>])
// where
//     T: Clone + Copy + Ord + PartialEq + Eq + Hash,
//     T: Default + TryFrom<usize>,
//     <T as TryFrom<usize>>::Error: Debug,
//     usize: TryFrom<T>,
//     <usize as TryFrom<T>>::Error: Debug,
// {
//     println!("DedupClauses:");
//     for (i, dc) in clauses.iter().enumerate() {
//         println!(
//             "  {}: {} {:?} {} {:?}",
//             i,
//             usize::try_from(dc.orig_index).unwrap(),
//             dc.extra_index.map(|x| usize::try_from(x).unwrap()),
//             dc.clause.kind,
//             dc.clause
//                 .literals
//                 .iter()
//                 .map(|(l, n)| (usize::try_from(*l).unwrap(), *n))
//                 .collect::<Vec<_>>()
//         );
//     }
// }
// DEBUG

/// Deduplicate clauses and clause literals. It deduplciates same clauses and literals and
/// subclauses (part of clauses). Returns new circuit and boolean value.
/// If some possible literal duplicates then returns true, otherwise return false.
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
    // DEBUG
    // println!("AndClauses");
    // dump_dedup_clauses(&and_clauses);
    // DEBUG

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
    // DEBUG
    // println!("XorClauses");
    // dump_dedup_clauses(&xor_clauses);
    // DEBUG

    if !and_clauses_need_optim {
        // because deduplicate_literal_clauses and deduplicate_literal_clauses_0
        // move some literals from old clauses into new clauses then
        // checking 1-literal clauses and duplicated literals is not needed.
        deduplicate_literal_clauses_0(
            &mut extra_clause_index,
            &mut and_clauses,
            &mut and_trans_tbl,
        );
        // DEBUG
        // println!("AndClauses2");
        // dump_dedup_clauses(&and_clauses);
        // DEBUG
        deduplicate_literal_clauses(
            &mut extra_clause_index,
            &mut and_clauses,
            &mut and_trans_tbl,
        );
    }
    // DEBUG
    // println!("AndClauses3");
    // dump_dedup_clauses(&and_clauses);
    // DEBUG

    if !xor_clauses_need_optim {
        // because deduplicate_literal_clauses and deduplicate_literal_clauses_0
        // move some literals from old clauses into new clauses then
        // checking 1-literal clauses and duplicated literals is not needed.
        deduplicate_literal_clauses_0(
            &mut extra_clause_index,
            &mut xor_clauses,
            &mut xor_trans_tbl,
        );
        // DEBUG
        // println!("XorClauses2");
        // dump_dedup_clauses(&xor_clauses);
        // DEBUG
        deduplicate_literal_clauses(
            &mut extra_clause_index,
            &mut xor_clauses,
            &mut xor_trans_tbl,
        );
    }
    // DEBUG
    // println!("XorClauses3");
    // dump_dedup_clauses(&xor_clauses);
    // DEBUG

    // println!("AndTransTbl: {:?}", and_trans_tbl);
    // println!("XorTransTbl: {:?}", xor_trans_tbl);
    translate_clauses(&mut and_clauses, &xor_trans_tbl);
    translate_clauses(&mut xor_clauses, &and_trans_tbl);

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

/// Returns optimized circuit, mapping to new inputs, mapping to new outputs.
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
        // DEBUG
        // println!("After optimize");
        // dump_clauses(
        //     usize::try_from(new_circuit.input_len()).unwrap(),
        //     new_circuit.clauses(),
        //     new_circuit.outputs(),
        // );
        // DEBUG
        (new_circuit, continue_dedup) = deduplicate_clause_circuit(new_circuit);
        // DEBUG
        // println!("After dedup");
        // dump_clauses(
        //     usize::try_from(new_circuit.input_len()).unwrap(),
        //     new_circuit.clauses(),
        //     new_circuit.outputs(),
        // );
        // DEBUG
        let (next_circuit, next_input_map, next_output_map) = optimize_clause_circuit(new_circuit);
        input_map = join_input_map(&input_map, &next_input_map);
        output_map = join_output_entry_map(&output_map, &next_output_map);
        new_circuit = next_circuit;
    }
    (new_circuit, input_map, output_map)
}

/// Assigns and optimize and deduplicate clause circuit.
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

// min and max depth of circuit

pub fn min_and_max_depth_list<T>(circuit: &Circuit<T>) -> (Vec<(T, T)>, T, T)
where
    T: Clone + Copy + PartialEq + PartialOrd + Ord + Eq + Debug,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let input_len_t = circuit.input_len();
    let input_len = usize::try_from(input_len_t).unwrap();
    let outputs = circuit.outputs();
    let gates = circuit.gates();
    let gate_num = gates.len();
    let mut global_min_depth = T::try_from(gate_num + 1).unwrap();
    let mut global_max_depth = T::default();
    struct StackEntry {
        node: usize,
        way: usize,
    }
    let mut visited = vec![false; input_len + gate_num];
    let mut depths = vec![(T::default(), T::default()); input_len + gate_num];
    let mut stack = vec![];
    for (o, _) in outputs {
        let oi = usize::try_from(*o).unwrap();
        stack.push(StackEntry { node: oi, way: 0 });
        while !stack.is_empty() {
            let top = stack.last_mut().unwrap();
            let way = top.way;
            if way == 0 {
                if !visited[top.node] {
                    visited[top.node] = true;
                } else {
                    stack.pop();
                    continue;
                }
                top.way += 1;
                let gate = gates[top.node - input_len];
                if gate.i0 >= input_len_t {
                    stack.push(StackEntry {
                        node: usize::try_from(gate.i0).unwrap(),
                        way: 0,
                    });
                }
            } else if way == 1 {
                top.way += 1;
                let gate = gates[top.node - input_len];
                if gate.i1 >= input_len_t {
                    stack.push(StackEntry {
                        node: usize::try_from(gate.i1).unwrap(),
                        way: 0,
                    });
                }
            } else {
                let gate = gates[top.node - input_len];
                let (min_depth0, max_depth0) = depths[usize::try_from(gate.i0).unwrap()];
                let (min_depth1, max_depth1) = depths[usize::try_from(gate.i1).unwrap()];
                depths[top.node] = (
                    T::try_from(
                        1 + usize::try_from(std::cmp::min(min_depth0, min_depth1)).unwrap(),
                    )
                    .unwrap(),
                    T::try_from(
                        1 + usize::try_from(std::cmp::max(max_depth0, max_depth1)).unwrap(),
                    )
                    .unwrap(),
                );
                stack.pop();
            }
        }
        let (min_depth, max_depth) = depths[oi];
        global_min_depth = std::cmp::min(global_min_depth, min_depth);
        global_max_depth = std::cmp::max(global_max_depth, max_depth);
    }
    (depths, global_min_depth, global_max_depth)
}

pub fn min_and_max_depth<T>(circuit: &Circuit<T>) -> (T, T)
where
    T: Clone + Copy + PartialEq + PartialOrd + Ord + Eq + Debug,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let (_, min_depth, max_depth) = min_and_max_depth_list(circuit);
    (min_depth, max_depth)
}

pub fn simple_pipeliner<T>(circuit: Circuit<T>, depth_in_stage: usize) -> Circuit<T>
where
    T: Clone + Copy + PartialEq + PartialOrd + Ord + Eq + Debug,
    T: Default + TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let (min_max_list, _, max_depth) = min_and_max_depth_list(&circuit);
    let input_len_t = circuit.input_len();
    let input_len = usize::try_from(input_len_t).unwrap();
    let outputs = circuit.outputs();
    let gates = circuit.gates();
    let gate_num = gates.len();
    // depths where wire must be hold
    let mut depths_to_hold = vec![max_depth; input_len + gate_num];
    for ((_, maxd), g) in min_max_list[input_len..].iter().zip(gates.iter()) {
        depths_to_hold[usize::try_from(g.i0).unwrap()] = *maxd;
        depths_to_hold[usize::try_from(g.i1).unwrap()] = *maxd;
    }
    // generate gate entries - holds original gate indices
    // entry: (stage in pipeline, depth in stage in pipeline, original gate wire index)
    let mut gate_entries = (input_len..input_len + gate_num)
        .map(|i| {
            let gate_depth = usize::try_from(depths_to_hold[i]).unwrap();
            (
                gate_depth / depth_in_stage,
                gate_depth % depth_in_stage,
                i,
            )
        })
        .collect::<Vec<_>>();
    // sort gate entries
    gate_entries.sort();
    // calculate state length
    // for (i, (stage, stage_depth, gi)) in gate_entries.iter().enumerate() {
    //     let cur_depth = stage * depth_in_stage + stage_depth;
    //     let next_stage_depth = stage * (depth_in_stage + 1);
    //     let 
    // }
    Circuit::new(T::default(), [], []).unwrap()
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
