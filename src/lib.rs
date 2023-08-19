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
}
