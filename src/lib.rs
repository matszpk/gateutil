use gatesim::*;

use std::cmp::Ord;
use std::fmt::Debug;

#[derive(Clone, Copy, Debug)]
pub enum Value<T> {
    Bool(bool),
    // index, negation
    Output(T, bool),
}

pub fn assign<T>(
    circuit: Circuit<T>,
    assign: impl IntoIterator<Item = (T, bool)>,
) -> (Circuit<T>, Vec<Value<T>>)
where
    T: Clone + Copy + PartialEq + Eq + Ord + Default,
    T: TryFrom<usize>,
    <T as TryFrom<usize>>::Error: Debug,
    usize: TryFrom<T>,
    <usize as TryFrom<T>>::Error: Debug,
{
    let input_len = usize::try_from(circuit.input_len()).unwrap();
    let mut assign_map = (0..input_len + circuit.len())
        .map(|x| Value::Output(T::try_from(x).unwrap(), false))
        .collect::<Vec<Value<T>>>();
    for (input_idx, value) in assign {
        assign_map[usize::try_from(input_idx).unwrap()] = Value::Bool(value);
    }
    let new_input_len = assign_map[0..input_len]
        .iter()
        .filter(|x| matches!(x, Value::Output(_, _)))
        .count();

    let mut new_gates: Vec<Gate<T>> = vec![];
    let mut new_output_count = input_len;
    for (i, g) in circuit.gates().iter().enumerate() {
        let oi = input_len + i;
        let gi1 = usize::try_from(g.i0).unwrap();
        let gi2 = usize::try_from(g.i1).unwrap();

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
                            let o2_0 = g.eval_args(false, false);
                            let o2_1 = g.eval_args(true, true);
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
                        }
                    }
                }
            }
        }
    }

    let mut output_value_mapping = vec![];
    let mut new_outputs = vec![];
    for (orig_idx, orig_n) in circuit.outputs() {
        let orig_idx = usize::try_from(*orig_idx).unwrap();
        match assign_map[orig_idx] {
            Value::Bool(v) => {
                output_value_mapping.push(Value::Bool(v ^ orig_n));
            }
            Value::Output(idx, n) => {
                output_value_mapping
                    .push(Value::Output(T::try_from(new_outputs.len()).unwrap(), n));
                new_outputs.push((idx, n ^ orig_n));
            }
        }
    }

    (
        Circuit::new(T::try_from(new_input_len).unwrap(), new_gates, new_outputs).unwrap(),
        output_value_mapping,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
}
