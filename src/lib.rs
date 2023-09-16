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
    for (i, (o, n)) in circuit.outputs().iter().enumerate() {
        let o_u = usize::try_from(*o).unwrap();
        match gate_map[o_u] {
            OutputEntry::NewIndex(no) => {
                new_outputs.push((no, *n));
                output_entries.push(OutputEntry::NewIndex(T::try_from(i).unwrap()));
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
// pub fn optimize_clause_circuit<T>(clause_circuit: ClauseCircuit<T>) -> ClauseCircuit<T>
// where
//     T: Default + TryFrom<usize>,
//     <T as TryFrom<usize>>::Error: Debug,
//     usize: TryFrom<T>,
//     <usize as TryFrom<T>>::Error: Debug,
// {
//     ClauseCircuit {
//         input_len: T::default(),
//         clauses: vec![],
//         outputs: vec![],
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        assert_eq!(
            Circuit::new(
                3,
                [
                    Gate::new_xor(0, 1),
                    Gate::new_xor(2, 3),
                    Gate::new_and(2, 3),
                    Gate::new_and(0, 1),
                    Gate::new_nor(5, 6),
                ],
                [(4, false), (7, true)],
            )
            .unwrap(),
            deduplicate(
                Circuit::new(
                    3,
                    [
                        Gate::new_xor(0, 1),
                        Gate::new_xor(2, 3),
                        Gate::new_and(2, 3),
                        Gate::new_and(0, 1),
                        Gate::new_nor(5, 6),
                    ],
                    [(4, false), (7, true)],
                )
                .unwrap()
            )
        );
        assert_eq!(
            Circuit::new(
                3,
                [
                    Gate::new_xor(0, 1),
                    Gate::new_xor(2, 3),
                    Gate::new_and(2, 3),
                    Gate::new_and(0, 1),
                    Gate::new_nor(5, 6),
                ],
                [(4, false), (7, true)],
            )
            .unwrap(),
            deduplicate(
                Circuit::new(
                    3,
                    [
                        Gate::new_xor(0, 1),
                        Gate::new_xor(2, 3),
                        Gate::new_xor(1, 0),
                        Gate::new_and(2, 5),
                        Gate::new_and(0, 1),
                        Gate::new_nor(6, 7),
                    ],
                    [(4, false), (8, true)],
                )
                .unwrap()
            )
        );
        assert_eq!(
            Circuit::new(
                4,
                [
                    Gate::new_xor(0, 1),
                    Gate::new_xor(2, 3),
                    Gate::new_nor(0, 1),
                    Gate::new_nor(2, 3),
                    Gate::new_and(4, 5),
                    Gate::new_and(6, 7),
                    Gate::new_nimpl(8, 9),
                    Gate::new_nimpl(9, 10),
                ],
                [(11, true)],
            )
            .unwrap(),
            deduplicate(
                Circuit::new(
                    4,
                    [
                        Gate::new_xor(0, 1),
                        Gate::new_xor(2, 3),
                        Gate::new_nor(0, 1),
                        Gate::new_nor(2, 3),
                        Gate::new_and(4, 5),
                        Gate::new_and(6, 7),
                        Gate::new_nor(0, 1),
                        Gate::new_nor(2, 3),
                        Gate::new_and(10, 11),
                        Gate::new_nimpl(8, 9),
                        Gate::new_nimpl(12, 13),
                    ],
                    [(14, true)],
                )
                .unwrap()
            )
        );
        assert_eq!(
            Circuit::new(
                4,
                [
                    Gate::new_xor(0, 1),
                    Gate::new_xor(2, 3),
                    Gate::new_nor(0, 1),
                    Gate::new_nor(2, 3),
                    Gate::new_and(4, 5),
                    Gate::new_and(6, 7),
                    Gate::new_nimpl(9, 8),
                    Gate::new_nimpl(9, 10),
                ],
                [(11, true)],
            )
            .unwrap(),
            deduplicate(
                Circuit::new(
                    4,
                    [
                        Gate::new_xor(1, 0), // arguments can be swapped
                        Gate::new_xor(2, 3),
                        Gate::new_nor(0, 1),
                        Gate::new_nor(2, 3),
                        Gate::new_and(4, 5),
                        Gate::new_and(6, 7),
                        Gate::new_nor(0, 1),
                        Gate::new_nor(2, 3),
                        Gate::new_and(10, 11),
                        Gate::new_nimpl(9, 8), // arguments can not be swapped
                        Gate::new_nimpl(12, 13),
                    ],
                    [(14, true)],
                )
                .unwrap()
            )
        );
    }

    #[test]
    fn test_assign_to_circuit() {
        assert_eq!(
            (
                Circuit::new(1, [Gate::new_and(0, 0)], [(1, false)]).unwrap(),
                vec![OutputEntry::NewIndex(0), OutputEntry::Value(true)],
                vec![OutputEntry::NewIndex(0)],
            ),
            assign_to_circuit(
                &Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
                [(1, true)]
            )
        );
        assert_eq!(
            (
                Circuit::new(1, [Gate::new_nimpl(0, 0)], [(1, false)]).unwrap(),
                vec![OutputEntry::NewIndex(0), OutputEntry::Value(false)],
                vec![OutputEntry::NewIndex(0)],
            ),
            assign_to_circuit(
                &Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
                [(1, false)]
            )
        );
        assert_eq!(
            (
                Circuit::new(1, [Gate::new_and(0, 0)], [(1, true)]).unwrap(),
                vec![OutputEntry::NewIndex(0), OutputEntry::Value(true)],
                vec![OutputEntry::NewIndex(0)],
            ),
            assign_to_circuit(
                &Circuit::new(2, [Gate::new_and(0, 1)], [(2, true)]).unwrap(),
                [(1, true)]
            )
        );
        
        assert_eq!(
            (
                Circuit::new(1, [Gate::new_and(0, 0)], [(1, false)]).unwrap(),
                vec![OutputEntry::Value(true), OutputEntry::NewIndex(0)],
                vec![OutputEntry::NewIndex(0)],
            ),
            assign_to_circuit(
                &Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
                [(0, true)]
            )
        );
        assert_eq!(
            (
                Circuit::new(1, [Gate::new_nimpl(0, 0)], [(1, false)]).unwrap(),
                vec![OutputEntry::Value(false), OutputEntry::NewIndex(0)],
                vec![OutputEntry::NewIndex(0)],
            ),
            assign_to_circuit(
                &Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
                [(0, false)]
            )
        );
        
        // evaluation of gate
        assert_eq!(
            (
                Circuit::new(0, [], []).unwrap(),
                vec![OutputEntry::Value(false), OutputEntry::Value(true)],
                vec![OutputEntry::Value(true)],
            ),
            assign_to_circuit(
                &Circuit::new(2, [Gate::new_and(0, 1)], [(2, true)]).unwrap(),
                [(0, false), (1, true)]
            )
        );
        assert_eq!(
            (
                Circuit::new(0, [], []).unwrap(),
                vec![OutputEntry::Value(false), OutputEntry::Value(true)],
                vec![OutputEntry::Value(false)],
            ),
            assign_to_circuit(
                &Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
                [(0, false), (1, true)]
            )
        );
        assert_eq!(
            (
                Circuit::new(0, [], []).unwrap(),
                vec![OutputEntry::Value(true), OutputEntry::Value(true)],
                vec![OutputEntry::Value(true)],
            ),
            assign_to_circuit(
                &Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
                [(0, true), (1, true)]
            )
        );
    }
}
