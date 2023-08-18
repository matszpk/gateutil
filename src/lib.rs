use gatesim::*;

use std::cmp::Ord;
use std::fmt::Debug;
use std::iter;

/// Assign values to inputs. Return new circuit and mapping to output.
pub fn assign<T>(
    circuit: Circuit<T>,
    assign: impl IntoIterator<Item = (T, bool)>,
) -> (Circuit<T>, Vec<T>, Vec<(T, bool)>)
where
    T: Clone + Copy + PartialEq + Eq + Ord + Default + Debug,
    T: TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    #[derive(Clone, Copy, Debug)]
    enum Value<T> {
        Bool(bool),
        // index, negation
        Output(T, bool),
    }

    let input_len = usize::try_from(circuit.input_len()).unwrap();
    let mut assign_map = iter::repeat(Value::Output(T::default(), false))
        .take(input_len + circuit.len())
        .collect::<Vec<Value<T>>>();
    for (input_idx, value) in assign {
        assign_map[usize::try_from(input_idx).unwrap()] = Value::Bool(value);
    }
    // count rest of input
    let new_input_len = assign_map[0..input_len]
        .iter()
        .filter(|x| matches!(x, Value::Output(_, _)))
        .count();
    // recounting rest of input
    let mut from_new_input = vec![0; new_input_len];
    for (i, (oi, o)) in assign_map[0..input_len]
        .iter_mut()
        .enumerate()
        .filter(|(_, x)| matches!(x, Value::Output(_, _)))
        .enumerate()
    {
        *o = Value::Output(T::try_from(i).unwrap(), false);
        from_new_input[i] = oi;
    }

    //println!("Test: {}", circuit);
    let mut used_inputs = vec![false; input_len];
    let mut new_gates: Vec<Gate<T>> = vec![];
    let mut new_output_count = new_input_len;
    for (i, g) in circuit.gates().iter().enumerate() {
        let oi = input_len + i;
        let gi1 = usize::try_from(g.i0).unwrap();
        let gi2 = usize::try_from(g.i1).unwrap();

        //println!("  Gate: {} {} {} {}", i, gi1, gi2, g.func);
        match assign_map[gi1] {
            Value::Bool(v1) => {
                match assign_map[gi2] {
                    Value::Bool(v2) => {
                        assign_map[oi] = Value::Bool(g.eval_args(v1, v2));
                    }
                    Value::Output(idx, n) => {
                        // no assign of v2
                        let o1_0 = g.eval_args(v1, false);
                        let o1_1 = g.eval_args(v1, true);
                        if o1_0 == o1_1 {
                            assign_map[oi] = Value::Bool(o1_0);
                        } else {
                            assign_map[oi] = Value::Output(idx, o1_0 ^ n);
                        }
                    }
                }
            }
            Value::Output(idx, n) => {
                match assign_map[gi2] {
                    Value::Bool(v2) => {
                        // no assign of v1
                        let o2_0 = g.eval_args(false, v2);
                        let o2_1 = g.eval_args(true, v2);
                        if o2_0 == o2_1 {
                            assign_map[oi] = Value::Bool(o2_0);
                        } else {
                            assign_map[oi] = Value::Output(idx, o2_0 ^ n);
                        }
                    }
                    Value::Output(idx2, n2) => {
                        if idx == idx2 {
                            // if same outputs
                            let (o2_0, o2_1) = if n == n2 {
                                if n {
                                    (g.eval_args(true, true), g.eval_args(false, false))
                                } else {
                                    (g.eval_args(false, false), g.eval_args(true, true))
                                }
                            } else {
                                if n {
                                    (g.eval_args(true, false), g.eval_args(false, true))
                                } else {
                                    (g.eval_args(false, true), g.eval_args(true, false))
                                }
                            };
                            if o2_0 == o2_1 {
                                assign_map[oi] = Value::Bool(o2_0);
                            } else {
                                assign_map[oi] = Value::Output(idx, o2_0);
                            }
                        } else {
                            let (new_func, swap, out_n) = match g.func {
                                GateFunc::And => {
                                    if n {
                                        if n2 {
                                            (GateFunc::Nor, false, false)
                                        } else {
                                            (GateFunc::Nimpl, true, false)
                                        }
                                    } else {
                                        if n2 {
                                            (GateFunc::Nimpl, false, false)
                                        } else {
                                            (GateFunc::And, false, false)
                                        }
                                    }
                                }
                                GateFunc::Nor => {
                                    if n {
                                        if n2 {
                                            (GateFunc::And, false, false)
                                        } else {
                                            (GateFunc::Nimpl, false, false)
                                        }
                                    } else {
                                        if n2 {
                                            (GateFunc::Nimpl, true, false)
                                        } else {
                                            (GateFunc::Nor, false, false)
                                        }
                                    }
                                }
                                GateFunc::Nimpl => {
                                    if n {
                                        if n2 {
                                            (GateFunc::Nimpl, true, false)
                                        } else {
                                            (GateFunc::Nor, false, false)
                                        }
                                    } else {
                                        if n2 {
                                            (GateFunc::And, true, false)
                                        } else {
                                            (GateFunc::Nimpl, false, false)
                                        }
                                    }
                                }
                                GateFunc::Xor => {
                                    if n {
                                        if n2 {
                                            (GateFunc::Xor, false, false)
                                        } else {
                                            (GateFunc::Xor, true, true)
                                        }
                                    } else {
                                        if n2 {
                                            (GateFunc::Xor, false, true)
                                        } else {
                                            (GateFunc::Xor, false, false)
                                        }
                                    }
                                }
                            };
                            let (idx, idx2) = if swap { (idx2, idx) } else { (idx, idx2) };
                            new_gates.push(Gate {
                                i0: idx,
                                i1: idx2,
                                func: new_func,
                            });
                            assign_map[oi] =
                                Value::Output(T::try_from(new_output_count).unwrap(), out_n);
                            new_output_count += 1;
                            if gi1 < input_len {
                                used_inputs[gi1] = true;
                            }
                            if gi2 < input_len {
                                used_inputs[gi2] = true;
                            }
                        }
                    }
                }
            }
        }
    }

    for (orig_idx, _) in circuit.outputs().iter() {
        let orig_idx = usize::try_from(*orig_idx).unwrap();
        //println!("IOM0xx: {} {:?} {:?}", orig_idx, assign_map, used_inputs);
        //if orig_idx < input_len && matches!(assign_map[orig_idx], Value::Output(_, _)) {
        if let Value::Output(idx, _) = assign_map[orig_idx] {
            let old_i = from_new_input[usize::try_from(idx).unwrap()];
            if old_i < input_len {
                used_inputs[old_i] = true;
            }
        }
    }

    //println!("IOM0: {:?} {:?}", assign_map, used_inputs);
    let old_new_input_len = new_input_len;

    let new_input_len = assign_map[0..input_len]
        .iter()
        .enumerate()
        .filter(|(i, x)| used_inputs[*i] && matches!(x, Value::Output(_, _)))
        .count();
    //println!("NewNewInputLen: {}", new_input_len);
    let out_inputs = assign_map[0..input_len]
        .iter()
        .enumerate()
        .filter(|(i, x)| used_inputs[*i] && matches!(x, Value::Output(_, _)))
        .map(|(i, _)| T::try_from(i).unwrap())
        .collect::<Vec<_>>();

    // fix inputs of new gates
    for g in new_gates.iter_mut() {
        let gi1 = usize::try_from(g.i0).unwrap();
        let gi2 = usize::try_from(g.i1).unwrap();
        if gi1 >= old_new_input_len {
            g.i0 = T::try_from(gi1 - (old_new_input_len - new_input_len)).unwrap();
        }
        if gi2 >= old_new_input_len {
            g.i1 = T::try_from(gi2 - (old_new_input_len - new_input_len)).unwrap();
        }
    }

    //println!("IOM: {:?} {:?}", circuit.outputs(), assign_map);
    let mut output_value_mapping = vec![];
    let mut new_outputs = vec![];
    for (i, (orig_idx, orig_n)) in circuit.outputs().iter().enumerate() {
        let orig_idx = usize::try_from(*orig_idx).unwrap();
        match assign_map[orig_idx] {
            Value::Bool(v) => {
                output_value_mapping.push((T::try_from(i).unwrap(), v ^ orig_n));
            }
            Value::Output(idx, n) => {
                new_outputs.push((idx, n ^ orig_n));
            }
        }
    }

    //println!("CVF: {:?} {:?} {:?}", new_input_len, new_gates, new_outputs);
    (
        Circuit::new(T::try_from(new_input_len).unwrap(), new_gates, new_outputs).unwrap(),
        out_inputs,
        output_value_mapping,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assign_empty() {
        assert_eq!(
            (Circuit::new(0, [], []).unwrap(), vec![], vec![(0, false)]),
            assign(Circuit::new(1, [], [(0, false)]).unwrap(), [(0, false)])
        );
        assert_eq!(
            (Circuit::new(0, [], []).unwrap(), vec![], vec![(0, true)]),
            assign(Circuit::new(1, [], [(0, false)]).unwrap(), [(0, true)])
        );
        assert_eq!(
            (Circuit::new(0, [], []).unwrap(), vec![], vec![(0, true)]),
            assign(Circuit::new(1, [], [(0, true)]).unwrap(), [(0, false)])
        );
        assert_eq!(
            (Circuit::new(0, [], []).unwrap(), vec![], vec![(0, false)]),
            assign(Circuit::new(1, [], [(0, true)]).unwrap(), [(0, true)])
        );

        assert_eq!(
            (
                Circuit::new(1, [], [(0, true)]).unwrap(),
                vec![1],
                vec![(0, true), (2, true), (3, false)]
            ),
            assign(
                Circuit::new(4, [], [(0, false), (1, true), (2, false), (3, false)]).unwrap(),
                [(0, true), (2, true), (3, false)]
            )
        );

        assert_eq!(
            (
                Circuit::new(2, [], [(0, true), (1, false)]).unwrap(),
                vec![1, 3],
                vec![(0, true), (2, false)]
            ),
            assign(
                Circuit::new(4, [], [(0, false), (1, true), (2, true), (3, false)]).unwrap(),
                [(0, true), (2, true)]
            )
        );
    }

    #[test]
    fn test_assign_1() {
        for (gate, input, value, out_neg, exp) in [
            (
                Gate::new_and(0, 1),
                0,
                false,
                false,
                (Circuit::new(0, [], []).unwrap(), vec![], vec![(0, false)]),
            ),
            (
                Gate::new_and(0, 1),
                0,
                false,
                true,
                (Circuit::new(0, [], []).unwrap(), vec![], vec![(0, true)]),
            ),
            (
                Gate::new_and(0, 1),
                0,
                true,
                false,
                (Circuit::new(1, [], [(0, false)]).unwrap(), vec![1], vec![]),
            ),
            (
                Gate::new_and(0, 1),
                1,
                true,
                false,
                (Circuit::new(1, [], [(0, false)]).unwrap(), vec![0], vec![]),
            ),
            (
                Gate::new_and(0, 1),
                0,
                true,
                true,
                (Circuit::new(1, [], [(0, true)]).unwrap(), vec![1], vec![]),
            ),
            (
                Gate::new_and(1, 0),
                0,
                false,
                false,
                (Circuit::new(0, [], []).unwrap(), vec![], vec![(0, false)]),
            ),
            (
                Gate::new_and(1, 0),
                0,
                true,
                false,
                (Circuit::new(1, [], [(0, false)]).unwrap(), vec![1], vec![]),
            ),
            (
                Gate::new_nor(0, 1),
                0,
                false,
                false,
                (Circuit::new(1, [], [(0, true)]).unwrap(), vec![1], vec![]),
            ),
            (
                Gate::new_nor(0, 1),
                0,
                false,
                true,
                (Circuit::new(1, [], [(0, false)]).unwrap(), vec![1], vec![]),
            ),
            (
                Gate::new_nor(0, 1),
                0,
                true,
                false,
                (Circuit::new(0, [], []).unwrap(), vec![], vec![(0, false)]),
            ),
            (
                Gate::new_nor(1, 0),
                0,
                false,
                false,
                (Circuit::new(1, [], [(0, true)]).unwrap(), vec![1], vec![]),
            ),
            (
                Gate::new_nor(1, 0),
                0,
                true,
                false,
                (Circuit::new(0, [], []).unwrap(), vec![], vec![(0, false)]),
            ),
            (
                Gate::new_nimpl(0, 1),
                0,
                false,
                false,
                (Circuit::new(0, [], []).unwrap(), vec![], vec![(0, false)]),
            ),
            (
                Gate::new_nimpl(0, 1),
                1,
                false,
                false,
                (Circuit::new(1, [], [(0, false)]).unwrap(), vec![0], vec![]),
            ),
            (
                Gate::new_nimpl(0, 1),
                0,
                true,
                false,
                (Circuit::new(1, [], [(0, true)]).unwrap(), vec![1], vec![]),
            ),
            (
                Gate::new_nimpl(1, 0),
                0,
                false,
                false,
                (Circuit::new(1, [], [(0, false)]).unwrap(), vec![1], vec![]),
            ),
            (
                Gate::new_nimpl(1, 0),
                0,
                true,
                false,
                (Circuit::new(0, [], []).unwrap(), vec![], vec![(0, false)]),
            ),
            (
                Gate::new_xor(0, 1),
                0,
                false,
                false,
                (Circuit::new(1, [], [(0, false)]).unwrap(), vec![1], vec![]),
            ),
            (
                Gate::new_xor(0, 1),
                0,
                true,
                false,
                (Circuit::new(1, [], [(0, true)]).unwrap(), vec![1], vec![]),
            ),
            (
                Gate::new_xor(1, 0),
                0,
                false,
                false,
                (Circuit::new(1, [], [(0, false)]).unwrap(), vec![1], vec![]),
            ),
            (
                Gate::new_xor(1, 0),
                0,
                true,
                false,
                (Circuit::new(1, [], [(0, true)]).unwrap(), vec![1], vec![]),
            ),
        ] {
            assert_eq!(
                exp,
                assign(
                    Circuit::new(2, [gate], [(2, out_neg)]).unwrap(),
                    [(input, value)]
                ),
                "{} {} {} {}",
                gate,
                input,
                value,
                out_neg
            );
        }
    }
    #[test]
    fn test_assign_2() {
        for (gate1, gate2, input, value, out_neg, exp) in [
            (
                Gate::new_and(0, 1),
                Gate::new_and(1, 2),
                0,
                false,
                false,
                (Circuit::new(0, [], []).unwrap(), vec![], vec![(0, false)]),
            ),
            (
                Gate::new_and(0, 1),
                Gate::new_and(1, 2),
                0,
                true,
                false,
                (Circuit::new(1, [], [(0, false)]).unwrap(), vec![1], vec![]),
            ),
            (
                Gate::new_nor(0, 1),
                Gate::new_and(1, 2),
                0,
                false,
                false,
                (Circuit::new(0, [], []).unwrap(), vec![], vec![(0, false)]),
            ),
        ] {
            assert_eq!(
                exp,
                assign(
                    Circuit::new(2, [gate1, gate2], [(3, out_neg)]).unwrap(),
                    [(input, value)]
                ),
                "{} {} {} {} {}",
                gate1,
                gate2,
                input,
                value,
                out_neg
            );
        }
    }
}
