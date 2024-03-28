use gatesim::*;
use gateutil::*;

#[test]
fn test_translate_inputs() {
    assert_eq!(
        Circuit::new(0, [], [],).unwrap(),
        translate_inputs::<u32, u32>(Circuit::new(0, [], []).unwrap(), &[])
    );
    assert_eq!(
        Circuit::new(
            3,
            [
                Gate::new_xor(2, 0),
                Gate::new_xor(1, 3),
                Gate::new_and(1, 3),
                Gate::new_and(2, 0),
                Gate::new_nor(5, 6),
            ],
            [(4, false), (7, true)],
        )
        .unwrap(),
        translate_inputs(
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
            &[2, 0, 1]
        )
    );
    assert_eq!(
        Circuit::new(
            4,
            [
                Gate::new_xor(2, 0),
                Gate::new_xor(3, 1),
                Gate::new_nor(2, 0),
                Gate::new_nor(3, 1),
                Gate::new_and(4, 5),
                Gate::new_and(6, 7),
                Gate::new_nimpl(8, 9),
                Gate::new_nimpl(9, 10),
            ],
            [(11, true)],
        )
        .unwrap(),
        translate_inputs(
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
            &[2, 0, 3, 1]
        )
    );
    assert_eq!(
        Circuit::new(
            4,
            [
                Gate::new_xor(2, 0),
                Gate::new_xor(1, 3),
                Gate::new_nor(2, 0),
                Gate::new_nor(1, 3),
                Gate::new_and(4, 5),
                Gate::new_and(6, 7),
                Gate::new_nimpl(8, 9),
                Gate::new_nimpl(9, 10),
            ],
            [(11, true)],
        )
        .unwrap(),
        translate_inputs(
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
            &[2, 0, 1, 3]
        )
    );
    assert_eq!(
        Circuit::new(
            4,
            [
                Gate::new_xor(2, 0),
                Gate::new_xor(1, 3),
                Gate::new_nor(2, 0),
                Gate::new_nor(1, 3),
                Gate::new_and(4, 5),
                Gate::new_and(6, 7),
                Gate::new_nimpl(8, 9),
                Gate::new_nimpl(9, 10),
            ],
            [(11, true), (2, false), (1, true)],
        )
        .unwrap(),
        translate_inputs(
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
                [(11, true), (0, false), (2, true)],
            )
            .unwrap(),
            &[2, 0, 1, 3]
        )
    );
}

#[test]
fn test_reverse_trans() {
    assert_eq!(vec![1, 2, 0, 3], reverse_trans([2, 0, 1, 3]));
    assert_eq!(vec![2, 4, 1, 0, 3], reverse_trans([3, 2, 0, 4, 1]));
}

#[test]
fn test_translate_inputs_rev() {
    assert_eq!(
        Circuit::new(
            4,
            [
                Gate::new_xor(1, 2),
                Gate::new_xor(0, 3),
                Gate::new_nor(1, 2),
                Gate::new_nor(0, 3),
                Gate::new_and(4, 5),
                Gate::new_and(6, 7),
                Gate::new_nimpl(8, 9),
                Gate::new_nimpl(9, 10),
            ],
            [(11, true)],
        )
        .unwrap(),
        translate_inputs_rev(
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
            [2, 0, 1, 3]
        )
    );
}

#[test]
fn test_negate_inputs() {
    assert_eq!(
        Circuit::new(0, [], [],).unwrap(),
        negate_inputs(Circuit::new(0, [], []).unwrap(), [])
    );
    // combinations of negations
    let circuit = Circuit::new(
        8,
        [
            Gate::new_and(0, 1),
            Gate::new_nor(2, 3),
            Gate::new_nimpl(4, 5),
            Gate::new_xor(6, 7),
        ],
        [(8, false), (9, true), (10, false), (11, true)],
    )
    .unwrap();
    assert_eq!(circuit.clone(), negate_inputs(circuit.clone(), []));
    assert_eq!(
        Circuit::new(
            8,
            [
                Gate::new_nimpl(1, 0),
                Gate::new_nimpl(2, 3),
                Gate::new_nor(4, 5),
                Gate::new_xor(6, 7),
            ],
            [(8, false), (9, true), (10, false), (11, false)]
        )
        .unwrap(),
        negate_inputs(circuit.clone(), [0, 2, 4, 6])
    );
    assert_eq!(
        Circuit::new(
            8,
            [
                Gate::new_nimpl(0, 1),
                Gate::new_nimpl(3, 2),
                Gate::new_and(4, 5),
                Gate::new_xor(6, 7),
            ],
            [(8, false), (9, true), (10, false), (11, false)]
        )
        .unwrap(),
        negate_inputs(circuit.clone(), [1, 3, 5, 7])
    );
    assert_eq!(
        Circuit::new(
            8,
            [
                Gate::new_nor(0, 1),
                Gate::new_and(2, 3),
                Gate::new_nimpl(5, 4),
                Gate::new_xor(6, 7),
            ],
            [(8, false), (9, true), (10, false), (11, true)]
        )
        .unwrap(),
        negate_inputs(circuit.clone(), [0, 1, 2, 3, 4, 5, 6, 7])
    );

    // with output connected to inputs
    let circuit = Circuit::new(
        8,
        [
            Gate::new_and(0, 1),
            Gate::new_nor(2, 3),
            Gate::new_nimpl(4, 5),
            Gate::new_xor(6, 7),
        ],
        [
            (0, false),
            (3, true),
            (8, false),
            (9, true),
            (10, false),
            (11, true),
        ],
    )
    .unwrap();
    assert_eq!(
        Circuit::new(
            8,
            [
                Gate::new_nimpl(0, 1),
                Gate::new_nimpl(3, 2),
                Gate::new_and(4, 5),
                Gate::new_xor(6, 7),
            ],
            [
                (0, false),
                (3, false),
                (8, false),
                (9, true),
                (10, false),
                (11, false)
            ]
        )
        .unwrap(),
        negate_inputs(circuit.clone(), [1, 3, 5, 7])
    );
    // nested xors
    let circuit = Circuit::new(
        4,
        [
            Gate::new_and(0, 1),
            Gate::new_and(2, 3),
            Gate::new_nor(4, 5),
            Gate::new_xor(0, 1),
            Gate::new_xor(2, 3),
            Gate::new_xor(7, 8),
            Gate::new_and(6, 9),
        ],
        [(6, false), (9, false), (10, true)],
    )
    .unwrap();
    assert_eq!(
        Circuit::new(
            4,
            [
                Gate::new_nimpl(1, 0),
                Gate::new_and(2, 3),
                Gate::new_nor(4, 5),
                Gate::new_xor(0, 1),
                Gate::new_xor(2, 3),
                Gate::new_xor(7, 8),
                Gate::new_nimpl(6, 9),
            ],
            [(6, false), (9, true), (10, true)],
        )
        .unwrap(),
        negate_inputs(circuit.clone(), [0])
    );
    assert_eq!(
        Circuit::new(
            4,
            [
                Gate::new_nimpl(0, 1),
                Gate::new_and(2, 3),
                Gate::new_nor(4, 5),
                Gate::new_xor(0, 1),
                Gate::new_xor(2, 3),
                Gate::new_xor(7, 8),
                Gate::new_nimpl(6, 9),
            ],
            [(6, false), (9, true), (10, true)],
        )
        .unwrap(),
        negate_inputs(circuit.clone(), [1])
    );
    assert_eq!(
        Circuit::new(
            4,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(3, 2),
                Gate::new_nor(4, 5),
                Gate::new_xor(0, 1),
                Gate::new_xor(2, 3),
                Gate::new_xor(7, 8),
                Gate::new_nimpl(6, 9),
            ],
            [(6, false), (9, true), (10, true)],
        )
        .unwrap(),
        negate_inputs(circuit.clone(), [2])
    );
    assert_eq!(
        Circuit::new(
            4,
            [
                Gate::new_nimpl(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_nor(4, 5),
                Gate::new_xor(0, 1),
                Gate::new_xor(2, 3),
                Gate::new_xor(7, 8),
                Gate::new_and(6, 9),
            ],
            [(6, false), (9, false), (10, true)],
        )
        .unwrap(),
        negate_inputs(circuit.clone(), [3, 1])
    );
}

#[test]
fn test_join_two_circuits() {
    assert_eq!(
        Circuit::new(0, [], [],).unwrap(),
        join_two_circuits(
            Circuit::new(0, [], []).unwrap(),
            [],
            Circuit::new(0, [], []).unwrap()
        )
    );
    assert_eq!(
        Circuit::new(
            4,
            [Gate::new_and(0, 1), Gate::new_nor(2, 3),],
            [(4, false), (5, false)]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
            [None, None],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap()
        )
    );
    assert_eq!(
        Circuit::new(3, [Gate::new_and(0, 1), Gate::new_nor(3, 2),], [(4, false)]).unwrap(),
        join_two_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
            [Some((0, false)), None],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap()
        )
    );
    assert_eq!(
        Circuit::new(3, [Gate::new_and(0, 1), Gate::new_nor(2, 3),], [(4, false)]).unwrap(),
        join_two_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
            [None, Some((0, false))],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap()
        )
    );
    assert_eq!(
        Circuit::new(2, [Gate::new_and(0, 1), Gate::new_nor(2, 2),], [(3, false)]).unwrap(),
        join_two_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
            [Some((0, false)), Some((0, false))],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap()
        )
    );
    // with input2 negation
    assert_eq!(
        Circuit::new(
            4,
            [Gate::new_and(0, 1), Gate::new_nor(2, 3),],
            [(4, true), (5, false)]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(2, true)]).unwrap(),
            [None, None],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap()
        )
    );
    assert_eq!(
        Circuit::new(
            3,
            [Gate::new_and(0, 1), Gate::new_nimpl(3, 2),],
            [(4, false)]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(2, true)]).unwrap(),
            [Some((0, false)), None],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap()
        )
    );
    assert_eq!(
        Circuit::new(
            3,
            [Gate::new_and(0, 1), Gate::new_nimpl(3, 2),],
            [(4, false)]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
            [Some((0, true)), None],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap()
        )
    );
    assert_eq!(
        Circuit::new(3, [Gate::new_and(0, 1), Gate::new_nor(3, 2),], [(4, false)]).unwrap(),
        join_two_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(2, true)]).unwrap(),
            [Some((0, true)), None],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap()
        )
    );
    assert_eq!(
        Circuit::new(
            3,
            [Gate::new_and(0, 1), Gate::new_nimpl(3, 2),],
            [(4, false)]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(2, true)]).unwrap(),
            [None, Some((0, false))],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap()
        )
    );
    assert_eq!(
        Circuit::new(2, [Gate::new_and(0, 1), Gate::new_and(2, 2),], [(3, false)]).unwrap(),
        join_two_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(2, true)]).unwrap(),
            [Some((0, false)), Some((0, false))],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap()
        )
    );
    assert_eq!(
        Circuit::new(2, [Gate::new_and(0, 1), Gate::new_and(2, 2),], [(3, false)]).unwrap(),
        join_two_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
            [Some((0, true)), Some((0, true))],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap()
        )
    );
    assert_eq!(
        Circuit::new(
            2,
            [Gate::new_and(0, 1), Gate::new_nimpl(2, 2),],
            [(3, false)]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
            [Some((0, true)), Some((0, false))],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap()
        )
    );
    assert_eq!(
        Circuit::new(
            2,
            [Gate::new_and(0, 1), Gate::new_nimpl(2, 2),],
            [(3, false)]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(2, true)]).unwrap(),
            [Some((0, true)), Some((0, false))],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap()
        )
    );
    // with connected inputs to outputs
    assert_eq!(
        Circuit::new(
            3,
            [Gate::new_and(0, 1), Gate::new_nor(1, 2),],
            [(3, false), (1, true), (4, false)]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(1, false), (2, false)]).unwrap(),
            [Some((0, false)), None],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(0, true), (2, false)]).unwrap()
        )
    );
    assert_eq!(
        Circuit::new(
            3,
            [Gate::new_and(0, 1), Gate::new_nor(3, 2),],
            [(1, false), (3, true), (4, false)]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(1, false), (2, false)]).unwrap(),
            [Some((1, false)), None],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(0, true), (2, false)]).unwrap()
        )
    );
    // with connected inputs to outputs (negs)
    assert_eq!(
        Circuit::new(
            3,
            [Gate::new_and(0, 1), Gate::new_nimpl(1, 2),],
            [(3, false), (1, false), (4, false)]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(1, true), (2, false)]).unwrap(),
            [Some((0, false)), None],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(0, true), (2, false)]).unwrap()
        )
    );
    assert_eq!(
        Circuit::new(
            3,
            [Gate::new_and(0, 1), Gate::new_nimpl(1, 2),],
            [(3, false), (1, false), (4, false)]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(1, false), (2, false)]).unwrap(),
            [Some((0, true)), None],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(0, true), (2, false)]).unwrap()
        )
    );
    assert_eq!(
        Circuit::new(
            3,
            [Gate::new_and(0, 1), Gate::new_nor(1, 2),],
            [(3, false), (1, true), (4, false)]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(1, true), (2, false)]).unwrap(),
            [Some((0, true)), None],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(0, true), (2, false)]).unwrap()
        )
    );
    assert_eq!(
        Circuit::new(
            3,
            [Gate::new_and(0, 1), Gate::new_nimpl(3, 2),],
            [(1, false), (3, false), (4, false)]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(1, false), (2, true)]).unwrap(),
            [Some((1, false)), None],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(0, true), (2, false)]).unwrap()
        )
    );
    // no fill
    assert_eq!(
        Circuit::new(
            9,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_nor(4, 5),
                Gate::new_xor(6, 8),
                Gate::new_nimpl(12, 7),
            ],
            [
                (1, false),
                (2, false),
                (9, false),
                (10, false),
                (4, false),
                (7, false),
                (11, false),
                (13, false)
            ]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(
                4,
                [Gate::new_and(0, 1), Gate::new_nimpl(2, 3),],
                [(1, false), (2, false), (4, false), (5, false)]
            )
            .unwrap(),
            [None, None, None, None, None],
            Circuit::new(
                5,
                [
                    Gate::new_nor(0, 1),
                    Gate::new_xor(2, 4),
                    Gate::new_nimpl(6, 3),
                ],
                [(0, false), (3, false), (5, false), (7, false)]
            )
            .unwrap()
        )
    );
    assert_eq!(
        Circuit::new(
            9,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_nor(4, 5),
                Gate::new_xor(6, 8),
                Gate::new_nimpl(12, 7),
            ],
            [
                (1, false),
                (2, true),
                (9, false),
                (10, true),
                (4, true),
                (7, false),
                (11, true),
                (13, false)
            ]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(
                4,
                [Gate::new_and(0, 1), Gate::new_nimpl(2, 3),],
                [(1, false), (2, true), (4, false), (5, true)]
            )
            .unwrap(),
            [None, None, None, None, None],
            Circuit::new(
                5,
                [
                    Gate::new_nor(0, 1),
                    Gate::new_xor(2, 4),
                    Gate::new_nimpl(6, 3),
                ],
                [(0, true), (3, false), (5, true), (7, false)]
            )
            .unwrap()
        )
    );
    // full fill
    assert_eq!(
        Circuit::new(
            4,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_nor(5, 2),
                Gate::new_xor(1, 2),
                Gate::new_nimpl(7, 4),
            ],
            [(5, false), (4, false), (6, false), (8, false)]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(
                4,
                [Gate::new_and(0, 1), Gate::new_nimpl(2, 3),],
                [(1, false), (2, false), (4, false), (5, false)]
            )
            .unwrap(),
            [
                Some((3, false)),
                Some((1, false)),
                Some((0, false)),
                Some((2, false)),
                Some((1, false))
            ],
            Circuit::new(
                5,
                [
                    Gate::new_nor(0, 1),
                    Gate::new_xor(2, 4),
                    Gate::new_nimpl(6, 3),
                ],
                [(0, false), (3, false), (5, false), (7, false)]
            )
            .unwrap()
        )
    );
    assert_eq!(
        Circuit::new(
            4,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_nimpl(5, 2),
                Gate::new_xor(1, 2),
                Gate::new_nor(7, 4),
            ],
            [(5, true), (4, true), (6, true), (8, false)]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(
                4,
                [Gate::new_and(0, 1), Gate::new_nimpl(2, 3),],
                [(1, true), (2, false), (4, false), (5, true)]
            )
            .unwrap(),
            [
                Some((3, false)),
                Some((1, false)),
                Some((0, false)),
                Some((2, false)),
                Some((1, false))
            ],
            Circuit::new(
                5,
                [
                    Gate::new_nor(0, 1),
                    Gate::new_xor(2, 4),
                    Gate::new_nimpl(6, 3),
                ],
                [(0, false), (3, true), (5, true), (7, false)]
            )
            .unwrap()
        )
    );
    assert_eq!(
        Circuit::new(
            4,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_nimpl(5, 2),
                Gate::new_xor(1, 2),
                Gate::new_nor(7, 4),
            ],
            [(5, true), (4, true), (6, true), (8, false)]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(
                4,
                [Gate::new_and(0, 1), Gate::new_nimpl(2, 3),],
                [(1, true), (2, false), (4, false), (5, false)]
            )
            .unwrap(),
            [
                Some((3, true)),
                Some((1, false)),
                Some((0, false)),
                Some((2, false)),
                Some((1, false))
            ],
            Circuit::new(
                5,
                [
                    Gate::new_nor(0, 1),
                    Gate::new_xor(2, 4),
                    Gate::new_nimpl(6, 3),
                ],
                [(0, false), (3, true), (5, true), (7, false)]
            )
            .unwrap()
        )
    );
    // full fill only in input2
    assert_eq!(
        Circuit::new(
            4,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_nor(4, 2),
                Gate::new_xor(4, 2),
                Gate::new_nimpl(7, 2),
            ],
            [
                (1, false),
                (5, false),
                (4, false),
                (2, false),
                (6, false),
                (8, false)
            ]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(
                4,
                [Gate::new_and(0, 1), Gate::new_nimpl(2, 3),],
                [(1, false), (2, false), (4, false), (5, false)]
            )
            .unwrap(),
            [
                Some((2, false)),
                Some((1, false)),
                Some((2, false)),
                Some((1, false)),
                Some((1, false))
            ],
            Circuit::new(
                5,
                [
                    Gate::new_nor(0, 1),
                    Gate::new_xor(2, 4),
                    Gate::new_nimpl(6, 3),
                ],
                [(0, false), (3, false), (5, false), (7, false)]
            )
            .unwrap()
        )
    );
    assert_eq!(
        Circuit::new(
            4,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_and(4, 2),
                Gate::new_xor(4, 2),
                Gate::new_and(7, 2),
            ],
            [
                (1, false),
                (5, false),
                (4, true),
                (2, false),
                (6, false),
                (8, true)
            ]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(
                4,
                [Gate::new_and(0, 1), Gate::new_nimpl(2, 3),],
                [(1, false), (2, true), (4, true), (5, false)]
            )
            .unwrap(),
            [
                Some((2, false)),
                Some((1, false)),
                Some((2, false)),
                Some((1, false)),
                Some((1, false))
            ],
            Circuit::new(
                5,
                [
                    Gate::new_nor(0, 1),
                    Gate::new_xor(2, 4),
                    Gate::new_nimpl(6, 3),
                ],
                [(0, false), (3, true), (5, false), (7, true)]
            )
            .unwrap()
        )
    );
    // full fill only in output1
    assert_eq!(
        Circuit::new(
            5,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_nor(6, 4),
                Gate::new_xor(1, 2),
                Gate::new_nimpl(8, 5),
            ],
            [(6, false), (5, false), (7, false), (9, false)]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(
                4,
                [Gate::new_and(0, 1), Gate::new_nimpl(2, 3),],
                [(1, false), (2, false), (4, false), (5, false)]
            )
            .unwrap(),
            [
                Some((3, false)),
                None,
                Some((0, false)),
                Some((2, false)),
                Some((1, false))
            ],
            Circuit::new(
                5,
                [
                    Gate::new_nor(0, 1),
                    Gate::new_xor(2, 4),
                    Gate::new_nimpl(6, 3),
                ],
                [(0, false), (3, false), (5, false), (7, false)]
            )
            .unwrap()
        )
    );
    assert_eq!(
        Circuit::new(
            5,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_nimpl(6, 4),
                Gate::new_xor(1, 2),
                Gate::new_and(8, 5),
            ],
            [(6, true), (5, false), (7, true), (9, false)]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(
                4,
                [Gate::new_and(0, 1), Gate::new_nimpl(2, 3),],
                [(1, false), (2, false), (4, true), (5, true)]
            )
            .unwrap(),
            [
                Some((3, false)),
                None,
                Some((0, false)),
                Some((2, false)),
                Some((1, false))
            ],
            Circuit::new(
                5,
                [
                    Gate::new_nor(0, 1),
                    Gate::new_xor(2, 4),
                    Gate::new_nimpl(6, 3),
                ],
                [(0, false), (3, true), (5, true), (7, false)]
            )
            .unwrap()
        )
    );
    // no full fill
    assert_eq!(
        Circuit::new(
            5,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_nor(5, 2),
                Gate::new_xor(5, 2),
                Gate::new_nimpl(8, 4),
            ],
            [
                (1, false),
                (6, false),
                (5, false),
                (4, false),
                (7, false),
                (9, false)
            ]
        )
        .unwrap(),
        join_two_circuits(
            Circuit::new(
                4,
                [Gate::new_and(0, 1), Gate::new_nimpl(2, 3),],
                [(1, false), (2, false), (4, false), (5, false)]
            )
            .unwrap(),
            [
                Some((2, false)),
                Some((1, false)),
                Some((2, false)),
                None,
                Some((1, false))
            ],
            Circuit::new(
                5,
                [
                    Gate::new_nor(0, 1),
                    Gate::new_xor(2, 4),
                    Gate::new_nimpl(6, 3),
                ],
                [(0, false), (3, false), (5, false), (7, false)]
            )
            .unwrap()
        )
    );
}

#[test]
fn test_join_circuits_seq() {
    // three
    assert_eq!(
        Circuit::new(
            6,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_nor(7, 4),
                Gate::new_xor(1, 2),
                Gate::new_nimpl(9, 6),
                Gate::new_and(10, 7),
                Gate::new_nor(5, 6),
                Gate::new_nor(12, 8),
                Gate::new_nimpl(11, 5),
            ],
            [(10, false), (8, false), (13, false), (14, false)]
        )
        .unwrap(),
        join_circuits_seq(
            [
                (
                    Circuit::new(
                        4,
                        [Gate::new_and(0, 1), Gate::new_nimpl(2, 3),],
                        [(1, false), (2, false), (4, false), (5, false)]
                    )
                    .unwrap(),
                    [
                        Some((3, false)),
                        None,
                        Some((0, false)),
                        Some((2, false)),
                        Some((1, false))
                    ]
                ),
                (
                    Circuit::new(
                        5,
                        [
                            Gate::new_nor(0, 1),
                            Gate::new_xor(2, 4),
                            Gate::new_nimpl(6, 3),
                        ],
                        [(0, false), (3, false), (5, false), (7, false)]
                    )
                    .unwrap(),
                    [
                        Some((7, false)),
                        Some((4, false)),
                        None,
                        Some((6, false)),
                        Some((5, false))
                    ]
                ),
            ],
            Circuit::new(
                5,
                [
                    Gate::new_and(0, 1),
                    Gate::new_nor(2, 4),
                    Gate::new_nor(6, 3),
                    Gate::new_nimpl(5, 2),
                ],
                [(0, false), (3, false), (7, false), (8, false)]
            )
            .unwrap()
        )
    );
    // three 2
    assert_eq!(
        Circuit::new(
            6,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_nor(7, 4),
                Gate::new_xor(6, 2),
                Gate::new_nimpl(8, 6),
                Gate::new_and(10, 7),
                Gate::new_nor(5, 1),
                Gate::new_nor(12, 9),
                Gate::new_nimpl(11, 5),
            ],
            [(10, false), (9, false), (13, false), (14, false)]
        )
        .unwrap(),
        join_circuits_seq(
            [
                (
                    Circuit::new(
                        4,
                        [Gate::new_and(0, 1), Gate::new_nimpl(2, 3),],
                        [(1, false), (2, false), (4, false), (5, false)]
                    )
                    .unwrap(),
                    [
                        Some((3, false)),
                        None,
                        Some((0, false)),
                        Some((2, false)),
                        Some((1, false))
                    ]
                ),
                (
                    Circuit::new(
                        5,
                        [
                            Gate::new_nor(0, 1),
                            Gate::new_xor(3, 4),
                            Gate::new_nimpl(5, 3),
                        ],
                        [(0, false), (2, false), (6, false), (7, false)]
                    )
                    .unwrap(),
                    [
                        Some((7, false)),
                        Some((4, false)),
                        None,
                        Some((6, false)),
                        Some((5, false))
                    ]
                ),
            ],
            Circuit::new(
                5,
                [
                    Gate::new_and(0, 1),
                    Gate::new_nor(2, 4),
                    Gate::new_nor(6, 3),
                    Gate::new_nimpl(5, 2),
                ],
                [(0, false), (3, false), (7, false), (8, false)]
            )
            .unwrap()
        )
    );
    // three 2 with negations
    assert_eq!(
        Circuit::new(
            6,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_nor(7, 4),
                Gate::new_xor(6, 2),
                Gate::new_and(8, 6),
                Gate::new_and(10, 7),
                Gate::new_nor(5, 1),
                Gate::new_nimpl(9, 12),
                Gate::new_nimpl(11, 5),
            ],
            [(10, false), (9, true), (13, false), (14, false)]
        )
        .unwrap(),
        join_circuits_seq(
            [
                (
                    Circuit::new(
                        4,
                        [Gate::new_and(0, 1), Gate::new_nimpl(2, 3),],
                        [(1, false), (2, false), (4, true), (5, false)]
                    )
                    .unwrap(),
                    [
                        Some((3, false)),
                        None,
                        Some((0, false)),
                        Some((2, false)),
                        Some((1, false))
                    ]
                ),
                (
                    Circuit::new(
                        5,
                        [
                            Gate::new_nor(0, 1),
                            Gate::new_xor(3, 4),
                            Gate::new_nimpl(5, 3),
                        ],
                        [(0, false), (2, false), (6, false), (7, false)]
                    )
                    .unwrap(),
                    [
                        Some((7, false)),
                        Some((4, false)),
                        None,
                        Some((6, false)),
                        Some((5, false))
                    ]
                ),
            ],
            Circuit::new(
                5,
                [
                    Gate::new_and(0, 1),
                    Gate::new_nor(2, 4),
                    Gate::new_nor(6, 3),
                    Gate::new_nimpl(5, 2),
                ],
                [(0, false), (3, false), (7, false), (8, false)]
            )
            .unwrap()
        )
    );
    assert_eq!(
        Circuit::new(
            6,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_nor(7, 4),
                Gate::new_xor(6, 2),
                Gate::new_and(8, 6),
                Gate::new_and(10, 7),
                Gate::new_nor(5, 1),
                Gate::new_nimpl(9, 12),
                Gate::new_nimpl(11, 5),
            ],
            [(10, false), (9, true), (13, false), (14, false)]
        )
        .unwrap(),
        join_circuits_seq(
            [
                (
                    Circuit::new(
                        4,
                        [Gate::new_and(0, 1), Gate::new_nimpl(2, 3),],
                        [(1, false), (2, false), (4, false), (5, false)]
                    )
                    .unwrap(),
                    [
                        Some((3, false)),
                        None,
                        Some((0, false)),
                        Some((2, true)),
                        Some((1, false))
                    ]
                ),
                (
                    Circuit::new(
                        5,
                        [
                            Gate::new_nor(0, 1),
                            Gate::new_xor(3, 4),
                            Gate::new_nimpl(5, 3),
                        ],
                        [(0, false), (2, false), (6, false), (7, false)]
                    )
                    .unwrap(),
                    [
                        Some((7, false)),
                        Some((4, false)),
                        None,
                        Some((6, false)),
                        Some((5, false))
                    ]
                ),
            ],
            Circuit::new(
                5,
                [
                    Gate::new_and(0, 1),
                    Gate::new_nor(2, 4),
                    Gate::new_nor(6, 3),
                    Gate::new_nimpl(5, 2),
                ],
                [(0, false), (3, false), (7, false), (8, false)]
            )
            .unwrap()
        )
    );
    // three with negations
    assert_eq!(
        Circuit::new(
            6,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_xor(7, 4),
                Gate::new_nor(1, 2),
                Gate::new_nimpl(9, 6),
                Gate::new_nimpl(10, 7),
                Gate::new_nor(5, 6),
                Gate::new_nimpl(8, 12),
                Gate::new_nimpl(11, 5),
            ],
            [(10, false), (8, true), (13, false), (14, false)]
        )
        .unwrap(),
        join_circuits_seq(
            [
                (
                    Circuit::new(
                        4,
                        [Gate::new_and(0, 1), Gate::new_nimpl(2, 3),],
                        [(1, false), (2, false), (4, false), (5, true)]
                    )
                    .unwrap(),
                    [
                        Some((3, false)),
                        None,
                        Some((0, false)),
                        Some((2, false)),
                        Some((1, false))
                    ]
                ),
                (
                    Circuit::new(
                        5,
                        [
                            Gate::new_xor(0, 1),
                            Gate::new_nor(2, 4),
                            Gate::new_nimpl(6, 3),
                        ],
                        [(0, false), (3, false), (5, false), (7, false)]
                    )
                    .unwrap(),
                    [
                        Some((7, false)),
                        Some((4, false)),
                        None,
                        Some((6, false)),
                        Some((5, false))
                    ]
                ),
            ],
            Circuit::new(
                5,
                [
                    Gate::new_and(0, 1),
                    Gate::new_nor(2, 4),
                    Gate::new_nor(6, 3),
                    Gate::new_nimpl(5, 2),
                ],
                [(0, false), (3, false), (7, false), (8, false)]
            )
            .unwrap()
        )
    );
    assert_eq!(
        Circuit::new(
            6,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_xor(7, 4),
                Gate::new_nor(1, 2),
                Gate::new_nimpl(9, 6),
                Gate::new_nimpl(10, 7),
                Gate::new_nimpl(6, 5),
                Gate::new_nimpl(8, 12),
                Gate::new_nimpl(11, 5),
            ],
            [(10, false), (8, true), (13, false), (14, false)]
        )
        .unwrap(),
        join_circuits_seq(
            [
                (
                    Circuit::new(
                        4,
                        [Gate::new_and(0, 1), Gate::new_nimpl(2, 3),],
                        [(1, false), (2, false), (4, false), (5, true)]
                    )
                    .unwrap(),
                    [
                        Some((3, false)),
                        None,
                        Some((0, false)),
                        Some((2, false)),
                        Some((1, false))
                    ]
                ),
                (
                    Circuit::new(
                        5,
                        [
                            Gate::new_xor(0, 1),
                            Gate::new_nor(2, 4),
                            Gate::new_nimpl(6, 3),
                        ],
                        [(0, false), (3, true), (5, false), (7, false)]
                    )
                    .unwrap(),
                    [
                        Some((7, false)),
                        Some((4, false)),
                        None,
                        Some((6, false)),
                        Some((5, false))
                    ]
                ),
            ],
            Circuit::new(
                5,
                [
                    Gate::new_and(0, 1),
                    Gate::new_nor(2, 4),
                    Gate::new_nor(6, 3),
                    Gate::new_nimpl(5, 2),
                ],
                [(0, false), (3, false), (7, false), (8, false)]
            )
            .unwrap()
        )
    );
    // three - use older outputs
    assert_eq!(
        Circuit::new(
            5,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_nor(6, 4),
                Gate::new_xor(1, 2),
                Gate::new_nimpl(8, 5),
                Gate::new_and(9, 6),
                Gate::new_nor(5, 5),
                Gate::new_nor(11, 7),
                Gate::new_nimpl(10, 5),
            ],
            [(9, false), (7, false), (12, false), (13, false)]
        )
        .unwrap(),
        join_circuits_seq(
            [
                (
                    Circuit::new(
                        4,
                        [Gate::new_and(0, 1), Gate::new_nimpl(2, 3),],
                        [(1, false), (2, false), (4, false), (5, false)]
                    )
                    .unwrap(),
                    [
                        Some((3, false)),
                        None,
                        Some((0, false)),
                        Some((2, false)),
                        Some((1, false))
                    ]
                ),
                (
                    Circuit::new(
                        5,
                        [
                            Gate::new_nor(0, 1),
                            Gate::new_xor(2, 4),
                            Gate::new_nimpl(6, 3),
                        ],
                        [(0, false), (3, false), (5, false), (7, false)]
                    )
                    .unwrap(),
                    [
                        Some((7, false)),
                        Some((4, false)),
                        Some((2, false)),
                        Some((6, false)),
                        Some((5, false))
                    ]
                ),
            ],
            Circuit::new(
                5,
                [
                    Gate::new_and(0, 1),
                    Gate::new_nor(2, 4),
                    Gate::new_nor(6, 3),
                    Gate::new_nimpl(5, 2),
                ],
                [(0, false), (3, false), (7, false), (8, false)]
            )
            .unwrap()
        )
    );
    // five
    assert_eq!(
        Circuit::new(
            8,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_xor(4, 5),
                Gate::new_and(5, 3),
                Gate::new_nimpl(11, 8),
                Gate::new_nor(3, 12),
                Gate::new_and(9, 10),
                Gate::new_nimpl(8, 14),
                Gate::new_xor(12, 15),
                Gate::new_xor(11, 6),
                Gate::new_nor(13, 10),
                Gate::new_and(18, 10),
                Gate::new_nor(14, 11),
                Gate::new_nimpl(15, 10),
                Gate::new_nimpl(9, 7),
                Gate::new_nimpl(4, 19),
            ],
            [
                (1, false),
                (5, false),
                (8, false),
                (16, true),
                (13, false),
                (10, false),
                (17, false),
                (20, false),
                (21, true),
                (22, false),
                (23, true)
            ]
        )
        .unwrap(),
        join_circuits_seq(
            [
                (
                    Circuit::new(
                        6,
                        [
                            Gate::new_and(0, 1),
                            Gate::new_nimpl(2, 3),
                            Gate::new_xor(4, 5),
                        ],
                        [
                            (1, false),
                            (3, true),
                            (4, false),
                            (5, false),
                            (6, true),
                            (7, false),
                            (8, false)
                        ]
                    )
                    .unwrap(),
                    vec![Some((3, false)), Some((1, true)), Some((4, true)),]
                ),
                (
                    Circuit::new(
                        3,
                        [
                            Gate::new_and(0, 1),
                            Gate::new_nimpl(3, 2),
                            Gate::new_nor(1, 4),
                        ],
                        [(0, false), (2, true), (3, true), (4, false), (5, false),]
                    )
                    .unwrap(),
                    vec![
                        Some((5, true)),
                        Some((6, true)),
                        Some((8, true)),
                        Some((10, false)),
                    ]
                ),
                (
                    Circuit::new(
                        4,
                        [
                            Gate::new_nor(0, 1),
                            Gate::new_nimpl(2, 4),
                            Gate::new_xor(3, 5),
                        ],
                        [
                            (0, false),
                            (1, true),
                            (2, false),
                            (4, true),
                            (5, false),
                            (6, true),
                        ]
                    )
                    .unwrap(),
                    vec![
                        Some((9, false)),
                        None,
                        Some((11, false)),
                        Some((6, false)),
                        Some((13, false))
                    ]
                ),
                (
                    Circuit::new(
                        5,
                        [
                            Gate::new_xor(0, 1),
                            Gate::new_nor(2, 3),
                            Gate::new_and(6, 4),
                        ],
                        [
                            (0, false),
                            (2, false),
                            (3, true),
                            (4, false),
                            (5, true),
                            (7, true),
                        ]
                    )
                    .unwrap(),
                    vec![
                        Some((15, false)),
                        Some((16, true)),
                        None,
                        Some((2, false)),
                        Some((18, false)),
                        Some((20, true)),
                        Some((12, false)),
                        Some((23, true))
                    ]
                ),
            ],
            Circuit::new(
                8,
                [
                    Gate::new_and(0, 4),
                    Gate::new_nor(1, 5),
                    Gate::new_nor(2, 6),
                    Gate::new_nimpl(3, 7),
                ],
                [(8, false), (9, true), (10, false), (11, true)]
            )
            .unwrap()
        )
    );
    // with empty input join
    assert_eq!(
        Circuit::new(
            6,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_nor(7, 4),
                Gate::new_xor(6, 2),
                Gate::new_and(8, 6),
                Gate::new_and(10, 7),
                Gate::new_nor(5, 1),
                Gate::new_nimpl(9, 12),
                Gate::new_nimpl(11, 5),
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_and(16, 4),
                Gate::new_nor(15, 2),
                Gate::new_and(17, 15),
                Gate::new_xor(19, 16),
                Gate::new_nor(5, 1),
                Gate::new_nimpl(18, 21),
                Gate::new_nimpl(20, 5),
                Gate::new_xor(10, 19),
                Gate::new_xor(9, 18),
                Gate::new_xor(13, 22),
                Gate::new_xor(14, 23),
                Gate::new_and(24, 25),
                Gate::new_and(26, 28),
                Gate::new_and(27, 29),
            ],
            [(30, false)]
        )
        .unwrap(),
        join_circuits_seq(
            [
                (
                    Circuit::new(6, [], (0..6).map(|i| (i, false))).unwrap(),
                    (0..6).map(|i| Some((i, false))).collect::<Vec<_>>()
                ),
                (
                    Circuit::new(
                        6,
                        [
                            Gate::new_and(0, 1),
                            Gate::new_nimpl(2, 3),
                            Gate::new_nor(7, 4),
                            Gate::new_xor(6, 2),
                            Gate::new_and(8, 6),
                            Gate::new_and(10, 7),
                            Gate::new_nor(5, 1),
                            Gate::new_nimpl(9, 12),
                            Gate::new_nimpl(11, 5),
                        ],
                        [(10, false), (9, true), (13, false), (14, false)]
                    )
                    .unwrap(),
                    (0..6).map(|i| Some((i, false))).collect::<Vec<_>>()
                ),
                (
                    Circuit::new(
                        6,
                        [
                            Gate::new_and(0, 1),
                            Gate::new_nimpl(2, 3),
                            Gate::new_and(7, 4),
                            Gate::new_nor(6, 2),
                            Gate::new_and(8, 6),
                            Gate::new_xor(10, 7),
                            Gate::new_nor(5, 1),
                            Gate::new_nimpl(9, 12),
                            Gate::new_nimpl(11, 5),
                        ],
                        [(10, false), (9, true), (13, false), (14, false)]
                    )
                    .unwrap(),
                    (6..6 + 4 * 2).map(|i| Some((i, false))).collect::<Vec<_>>()
                ),
            ],
            Circuit::new(
                8,
                [
                    Gate::new_xor(0, 4),
                    Gate::new_xor(1, 5),
                    Gate::new_xor(2, 6),
                    Gate::new_xor(3, 7),
                    Gate::new_and(8, 9),
                    Gate::new_and(10, 12),
                    Gate::new_and(11, 13),
                ],
                [(14, false)]
            )
            .unwrap()
        )
    );
    // with empty input join
    assert_eq!(
        Circuit::new(
            6,
            [
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_nor(7, 4),
                Gate::new_xor(6, 2),
                Gate::new_and(8, 6),
                Gate::new_and(10, 7),
                Gate::new_nor(5, 1),
                Gate::new_nimpl(9, 12),
                Gate::new_nimpl(11, 5),
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_and(16, 4),
                Gate::new_nor(15, 2),
                Gate::new_and(17, 15),
                Gate::new_xor(19, 16),
                Gate::new_nor(5, 1),
                Gate::new_nimpl(18, 21),
                Gate::new_nimpl(20, 5),
                Gate::new_xor(10, 18),
                Gate::new_xor(9, 19),
                Gate::new_xor(13, 23),
                Gate::new_xor(14, 22),
                Gate::new_nor(24, 25),
                Gate::new_and(26, 28),
                Gate::new_and(27, 29),
            ],
            [(30, false)]
        )
        .unwrap(),
        join_circuits_seq(
            [
                (
                    Circuit::new(6, [], (0..6).map(|i| (i, false))).unwrap(),
                    (0..6).map(|i| Some((i, false))).collect::<Vec<_>>()
                ),
                (
                    Circuit::new(
                        6,
                        [
                            Gate::new_and(0, 1),
                            Gate::new_nimpl(2, 3),
                            Gate::new_nor(7, 4),
                            Gate::new_xor(6, 2),
                            Gate::new_and(8, 6),
                            Gate::new_and(10, 7),
                            Gate::new_nor(5, 1),
                            Gate::new_nimpl(9, 12),
                            Gate::new_nimpl(11, 5),
                        ],
                        [(10, false), (9, true), (13, false), (14, false)]
                    )
                    .unwrap(),
                    (0..6).map(|i| Some((i, false))).collect::<Vec<_>>()
                ),
                (
                    Circuit::new(
                        6,
                        [
                            Gate::new_and(0, 1),
                            Gate::new_nimpl(2, 3),
                            Gate::new_and(7, 4),
                            Gate::new_nor(6, 2),
                            Gate::new_and(8, 6),
                            Gate::new_xor(10, 7),
                            Gate::new_nor(5, 1),
                            Gate::new_nimpl(9, 12),
                            Gate::new_nimpl(11, 5),
                        ],
                        [(9, true), (10, false), (14, false), (13, false)]
                    )
                    .unwrap(),
                    (6..6 + 4 * 2).map(|i| Some((i, false))).collect::<Vec<_>>()
                ),
            ],
            Circuit::new(
                8,
                [
                    Gate::new_xor(0, 4),
                    Gate::new_xor(1, 5),
                    Gate::new_xor(2, 6),
                    Gate::new_xor(3, 7),
                    Gate::new_and(8, 9),
                    Gate::new_and(10, 12),
                    Gate::new_and(11, 13),
                ],
                [(14, false)]
            )
            .unwrap()
        )
    );
    // with empty input join
    assert_eq!(
        Circuit::new(
            6,
            [
                Gate::new_and(0, 1),
                Gate::new_and(2, 3),
                Gate::new_nor(7, 4),
                Gate::new_xor(6, 2),
                Gate::new_and(8, 6),
                Gate::new_and(10, 7),
                Gate::new_nor(5, 1),
                Gate::new_nimpl(9, 12),
                Gate::new_nimpl(11, 5),
                Gate::new_and(0, 1),
                Gate::new_nimpl(2, 3),
                Gate::new_nimpl(16, 4),
                Gate::new_nor(15, 2),
                Gate::new_and(17, 15),
                Gate::new_xor(19, 16),
                Gate::new_nor(5, 1),
                Gate::new_nimpl(18, 21),
                Gate::new_nimpl(20, 5),
                Gate::new_xor(10, 18),
                Gate::new_xor(9, 19),
                Gate::new_xor(13, 23),
                Gate::new_xor(14, 22),
                Gate::new_nor(24, 25),
                Gate::new_and(26, 28),
                Gate::new_and(27, 29),
            ],
            [(30, false)]
        )
        .unwrap(),
        join_circuits_seq(
            [
                (
                    Circuit::new(6, [], (0..6).map(|i| (i, false))).unwrap(),
                    (0..6).map(|i| Some((i, i == 3))).collect::<Vec<_>>()
                ),
                (
                    Circuit::new(
                        6,
                        [
                            Gate::new_and(0, 1),
                            Gate::new_nimpl(2, 3),
                            Gate::new_nor(7, 4),
                            Gate::new_xor(6, 2),
                            Gate::new_and(8, 6),
                            Gate::new_and(10, 7),
                            Gate::new_nor(5, 1),
                            Gate::new_nimpl(9, 12),
                            Gate::new_nimpl(11, 5),
                        ],
                        [(10, false), (9, true), (13, false), (14, false)]
                    )
                    .unwrap(),
                    (0..6).map(|i| Some((i, i == 4))).collect::<Vec<_>>()
                ),
                (
                    Circuit::new(
                        6,
                        [
                            Gate::new_and(0, 1),
                            Gate::new_nimpl(2, 3),
                            Gate::new_and(7, 4),
                            Gate::new_nor(6, 2),
                            Gate::new_and(8, 6),
                            Gate::new_xor(10, 7),
                            Gate::new_nor(5, 1),
                            Gate::new_nimpl(9, 12),
                            Gate::new_nimpl(11, 5),
                        ],
                        [(9, true), (10, false), (14, false), (13, false)]
                    )
                    .unwrap(),
                    (6..6 + 4 * 2).map(|i| Some((i, false))).collect::<Vec<_>>()
                ),
            ],
            Circuit::new(
                8,
                [
                    Gate::new_xor(0, 4),
                    Gate::new_xor(1, 5),
                    Gate::new_xor(2, 6),
                    Gate::new_xor(3, 7),
                    Gate::new_and(8, 9),
                    Gate::new_and(10, 12),
                    Gate::new_and(11, 13),
                ],
                [(14, false)]
            )
            .unwrap()
        )
    );
}

#[test]
fn test_deduplicate() {
    assert_eq!(
        Circuit::new(0, [], [],).unwrap(),
        deduplicate(Circuit::new(0, [], [],).unwrap())
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
