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
fn test_join_circuits() {
    assert_eq!(
        Circuit::new(0, [], [],).unwrap(),
        join_circuits(
            Circuit::new(0, [], []).unwrap(),
            [],
            Circuit::new(0, [], []).unwrap()
        )
    );
    assert_eq!(
        Circuit::new(3, [Gate::new_and(0, 1), Gate::new_nor(3, 2),], [(4, false)]).unwrap(),
        join_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
            [Some(0), None],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap()
        )
    );
    assert_eq!(
        Circuit::new(3, [Gate::new_and(0, 1), Gate::new_nor(2, 3),], [(4, false)]).unwrap(),
        join_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
            [None, Some(0)],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap()
        )
    );
    assert_eq!(
        Circuit::new(2, [Gate::new_and(0, 1), Gate::new_nor(2, 2),], [(3, false)]).unwrap(),
        join_circuits(
            Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
            [Some(0), Some(0)],
            Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap()
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

#[test]
fn test_assign_to_circuit() {
    // assign 0 input
    assert_eq!(
        (
            Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
            vec![OutputEntry::NewIndex(0), OutputEntry::NewIndex(1)],
            vec![OutputEntry::NewIndex(0)],
        ),
        assign_to_circuit(
            &Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
            []
        )
    );
    // assign 0 input to empty circuit
    assert_eq!(
        (Circuit::new(0, [], []).unwrap(), vec![], vec![],),
        assign_to_circuit(&Circuit::new(0, [], []).unwrap(), [])
    );
    // assign 1 input
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
            Circuit::new(1, [Gate::new_nimpl(0, 0)], [(1, false)]).unwrap(),
            vec![OutputEntry::NewIndex(0), OutputEntry::Value(true)],
            vec![OutputEntry::NewIndex(0)],
        ),
        assign_to_circuit(
            &Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap(),
            [(1, true)]
        )
    );
    assert_eq!(
        (
            Circuit::new(1, [Gate::new_nor(0, 0)], [(1, false)]).unwrap(),
            vec![OutputEntry::NewIndex(0), OutputEntry::Value(false)],
            vec![OutputEntry::NewIndex(0)],
        ),
        assign_to_circuit(
            &Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap(),
            [(1, false)]
        )
    );

    assert_eq!(
        (
            Circuit::new(1, [Gate::new_nimpl(0, 0)], [(1, false)]).unwrap(),
            vec![OutputEntry::NewIndex(0), OutputEntry::Value(true)],
            vec![OutputEntry::NewIndex(0)],
        ),
        assign_to_circuit(
            &Circuit::new(2, [Gate::new_nimpl(0, 1)], [(2, false)]).unwrap(),
            [(1, true)]
        )
    );
    assert_eq!(
        (
            Circuit::new(1, [Gate::new_and(0, 0)], [(1, false)]).unwrap(),
            vec![OutputEntry::NewIndex(0), OutputEntry::Value(false)],
            vec![OutputEntry::NewIndex(0)],
        ),
        assign_to_circuit(
            &Circuit::new(2, [Gate::new_nimpl(0, 1)], [(2, false)]).unwrap(),
            [(1, false)]
        )
    );

    // assign 0 input
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

    assert_eq!(
        (
            Circuit::new(1, [Gate::new_nimpl(0, 0)], [(1, false)]).unwrap(),
            vec![OutputEntry::Value(true), OutputEntry::NewIndex(0)],
            vec![OutputEntry::NewIndex(0)],
        ),
        assign_to_circuit(
            &Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap(),
            [(0, true)]
        )
    );
    assert_eq!(
        (
            Circuit::new(1, [Gate::new_nor(0, 0)], [(1, false)]).unwrap(),
            vec![OutputEntry::Value(false), OutputEntry::NewIndex(0)],
            vec![OutputEntry::NewIndex(0)],
        ),
        assign_to_circuit(
            &Circuit::new(2, [Gate::new_nor(0, 1)], [(2, false)]).unwrap(),
            [(0, false)]
        )
    );

    assert_eq!(
        (
            Circuit::new(1, [Gate::new_nor(0, 0)], [(1, false)]).unwrap(),
            vec![OutputEntry::Value(true), OutputEntry::NewIndex(0)],
            vec![OutputEntry::NewIndex(0)],
        ),
        assign_to_circuit(
            &Circuit::new(2, [Gate::new_xor(0, 1)], [(2, false)]).unwrap(),
            [(0, true)]
        )
    );
    assert_eq!(
        (
            Circuit::new(1, [Gate::new_and(0, 0)], [(1, false)]).unwrap(),
            vec![OutputEntry::Value(false), OutputEntry::NewIndex(0)],
            vec![OutputEntry::NewIndex(0)],
        ),
        assign_to_circuit(
            &Circuit::new(2, [Gate::new_xor(0, 1)], [(2, false)]).unwrap(),
            [(0, false)]
        )
    );

    assert_eq!(
        (
            Circuit::new(1, [Gate::new_nor(0, 0)], [(1, false)]).unwrap(),
            vec![OutputEntry::Value(true), OutputEntry::NewIndex(0)],
            vec![OutputEntry::NewIndex(0)],
        ),
        assign_to_circuit(
            &Circuit::new(2, [Gate::new_nimpl(0, 1)], [(2, false)]).unwrap(),
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
            &Circuit::new(2, [Gate::new_nimpl(0, 1)], [(2, false)]).unwrap(),
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
    assert_eq!(
        (
            Circuit::new(0, [], []).unwrap(),
            vec![OutputEntry::Value(true), OutputEntry::Value(true)],
            vec![OutputEntry::Value(false)],
        ),
        assign_to_circuit(
            &Circuit::new(2, [Gate::new_xor(0, 1)], [(2, false)]).unwrap(),
            [(0, true), (1, true)]
        )
    );
}

#[test]
fn test_assign_to_circuit_2() {
    assert_eq!(
        (
            Circuit::new(
                3,
                [
                    Gate::new_and(1, 1),
                    Gate::new_and(0, 1),
                    Gate::new_and(2, 2),
                    Gate::new_and(0, 2),
                    // add a1*b0 + a0*b1
                    Gate::new_xor(4, 5),
                    Gate::new_and(4, 5),
                    // add c(a1*b0 + a0*b1) + a1*b1
                    Gate::new_xor(6, 8),
                    Gate::new_and(6, 8),
                ],
                [(3, false), (7, false), (9, false), (10, false)],
            )
            .unwrap(),
            vec![
                OutputEntry::Value(true),
                OutputEntry::NewIndex(0),
                OutputEntry::NewIndex(1),
                OutputEntry::NewIndex(2)
            ],
            vec![
                OutputEntry::NewIndex(0),
                OutputEntry::NewIndex(1),
                OutputEntry::NewIndex(2),
                OutputEntry::NewIndex(3)
            ],
        ),
        assign_to_circuit(
            &Circuit::new(
                4,
                [
                    Gate::new_and(0, 2),
                    Gate::new_and(1, 2),
                    Gate::new_and(0, 3),
                    Gate::new_and(1, 3),
                    // add a1*b0 + a0*b1
                    Gate::new_xor(5, 6),
                    Gate::new_and(5, 6),
                    // add c(a1*b0 + a0*b1) + a1*b1
                    Gate::new_xor(7, 9),
                    Gate::new_and(7, 9),
                ],
                [(4, false), (8, false), (10, false), (11, false)],
            )
            .unwrap(),
            [(0, true)]
        )
    );

    assert_eq!(
        (
            Circuit::new(
                2,
                [
                    // Gate::new_and(false, true),
                    Gate::new_and(0, 0),
                    Gate::new_nimpl(1, 1),
                    Gate::new_and(0, 1),
                    // add a1*b0 + a0*b1
                    Gate::new_xor(2, 3),
                    Gate::new_and(2, 3),
                    // add c(a1*b0 + a0*b1) + a1*b1
                    Gate::new_xor(4, 6),
                    Gate::new_and(4, 6),
                ],
                [(5, false), (7, false), (8, false)],
            )
            .unwrap(),
            vec![
                OutputEntry::Value(false),
                OutputEntry::NewIndex(0),
                OutputEntry::Value(true),
                OutputEntry::NewIndex(1)
            ],
            vec![
                OutputEntry::Value(false),
                OutputEntry::NewIndex(0),
                OutputEntry::NewIndex(1),
                OutputEntry::NewIndex(2)
            ],
        ),
        assign_to_circuit(
            &Circuit::new(
                4,
                [
                    Gate::new_and(0, 2),
                    Gate::new_and(1, 2),
                    Gate::new_and(0, 3),
                    Gate::new_and(1, 3),
                    // add a1*b0 + a0*b1
                    Gate::new_xor(5, 6),
                    Gate::new_and(5, 6),
                    // add c(a1*b0 + a0*b1) + a1*b1
                    Gate::new_xor(7, 9),
                    Gate::new_and(7, 9),
                ],
                [(4, false), (8, false), (10, false), (11, false)],
            )
            .unwrap(),
            [(0, false), (2, true)]
        )
    );

    assert_eq!(
        (
            Circuit::new(
                2,
                [
                    Gate::new_and(0, 1),
                    Gate::new_and(1, 1),
                    Gate::new_nimpl(0, 0),
                    // Gate::new_and(1, 3), -> false
                    // add a1*b0 + a0*b1
                    Gate::new_xor(3, 4),
                    Gate::new_and(3, 4),
                    // add c(a1*b0 + a0*b1) + a1*b1
                    Gate::new_and(6, 6),
                    Gate::new_nimpl(6, 6),
                ],
                [(2, false), (5, false), (7, false), (8, false)],
            )
            .unwrap(),
            vec![
                OutputEntry::NewIndex(0),
                OutputEntry::Value(true),
                OutputEntry::NewIndex(1),
                OutputEntry::Value(false)
            ],
            vec![
                OutputEntry::NewIndex(0),
                OutputEntry::NewIndex(1),
                OutputEntry::NewIndex(2),
                OutputEntry::NewIndex(3)
            ],
        ),
        assign_to_circuit(
            &Circuit::new(
                4,
                [
                    Gate::new_and(0, 2),
                    Gate::new_and(1, 2),
                    Gate::new_and(0, 3),
                    Gate::new_and(1, 3),
                    // add a1*b0 + a0*b1
                    Gate::new_xor(5, 6),
                    Gate::new_and(5, 6),
                    // add c(a1*b0 + a0*b1) + a1*b1
                    Gate::new_xor(7, 9),
                    Gate::new_and(7, 9),
                ],
                [(4, false), (8, false), (10, false), (11, false)],
            )
            .unwrap(),
            [(1, true), (3, false)]
        )
    );

    assert_eq!(
        (
            Circuit::new(
                2,
                [
                    Gate::new_and(0, 1),
                    Gate::new_and(1, 1),
                    Gate::new_and(0, 0),
                    // Gate::new_and(1, 3), -> true
                    // add a1*b0 + a0*b1
                    Gate::new_xor(3, 4),
                    Gate::new_and(3, 4),
                    // add c(a1*b0 + a0*b1) + a1*b1
                    Gate::new_nor(6, 6),
                    Gate::new_and(6, 6),
                ],
                [(2, false), (5, false), (7, false), (8, false)],
            )
            .unwrap(),
            vec![
                OutputEntry::NewIndex(0),
                OutputEntry::Value(true),
                OutputEntry::NewIndex(1),
                OutputEntry::Value(true)
            ],
            vec![
                OutputEntry::NewIndex(0),
                OutputEntry::NewIndex(1),
                OutputEntry::NewIndex(2),
                OutputEntry::NewIndex(3)
            ],
        ),
        assign_to_circuit(
            &Circuit::new(
                4,
                [
                    Gate::new_and(0, 2),
                    Gate::new_and(1, 2),
                    Gate::new_and(0, 3),
                    Gate::new_and(1, 3),
                    // add a1*b0 + a0*b1
                    Gate::new_xor(5, 6),
                    Gate::new_and(5, 6),
                    // add c(a1*b0 + a0*b1) + a1*b1
                    Gate::new_xor(7, 9),
                    Gate::new_and(7, 9),
                ],
                [(4, false), (8, false), (10, false), (11, false)],
            )
            .unwrap(),
            [(1, true), (3, true)]
        )
    );

    assert_eq!(
        (
            Circuit::new(
                1,
                [
                    Gate::new_nimpl(0, 0),
                    Gate::new_and(0, 0),
                    // Gate::new_and(0, 3), -> false
                    // Gate::new_and(1, 3), -> true
                    // add a1*b0 + a0*b1
                    Gate::new_and(2, 2),
                    Gate::new_nimpl(2, 2),
                    // add c(a1*b0 + a0*b1) + a1*b1
                    Gate::new_nor(4, 4),
                    Gate::new_and(4, 4),
                ],
                [(1, false), (3, false), (5, false), (6, false)],
            )
            .unwrap(),
            vec![
                OutputEntry::Value(false),
                OutputEntry::Value(true),
                OutputEntry::NewIndex(0),
                OutputEntry::Value(true)
            ],
            vec![
                OutputEntry::NewIndex(0),
                OutputEntry::NewIndex(1),
                OutputEntry::NewIndex(2),
                OutputEntry::NewIndex(3)
            ],
        ),
        assign_to_circuit(
            &Circuit::new(
                4,
                [
                    Gate::new_and(0, 2),
                    Gate::new_and(1, 2),
                    Gate::new_and(0, 3),
                    Gate::new_and(1, 3),
                    // add a1*b0 + a0*b1
                    Gate::new_xor(5, 6),
                    Gate::new_and(5, 6),
                    // add c(a1*b0 + a0*b1) + a1*b1
                    Gate::new_xor(7, 9),
                    Gate::new_and(7, 9),
                ],
                [(4, false), (8, false), (10, false), (11, false)],
            )
            .unwrap(),
            [(0, false), (1, true), (3, true)]
        )
    );

    assert_eq!(
        (
            Circuit::new(0, [], []).unwrap(),
            vec![
                OutputEntry::Value(false),
                OutputEntry::Value(true),
                OutputEntry::Value(false),
                OutputEntry::Value(true)
            ],
            vec![
                OutputEntry::Value(false),
                OutputEntry::Value(false),
                OutputEntry::Value(true),
                OutputEntry::Value(false)
            ],
        ),
        assign_to_circuit(
            &Circuit::new(
                4,
                [
                    Gate::new_and(0, 2),
                    Gate::new_and(1, 2),
                    Gate::new_and(0, 3),
                    Gate::new_and(1, 3),
                    // add a1*b0 + a0*b1
                    Gate::new_xor(5, 6),
                    Gate::new_and(5, 6),
                    // add c(a1*b0 + a0*b1) + a1*b1
                    Gate::new_xor(7, 9),
                    Gate::new_and(7, 9),
                ],
                [(4, false), (8, false), (10, false), (11, false)],
            )
            .unwrap(),
            [(0, false), (1, true), (2, false), (3, true)]
        )
    );

    assert_eq!(
        (
            Circuit::new(0, [], []).unwrap(),
            vec![
                OutputEntry::Value(false),
                OutputEntry::Value(true),
                OutputEntry::Value(true),
                OutputEntry::Value(true)
            ],
            vec![
                OutputEntry::Value(false),
                OutputEntry::Value(true),
                OutputEntry::Value(true),
                OutputEntry::Value(false)
            ],
        ),
        assign_to_circuit(
            &Circuit::new(
                4,
                [
                    Gate::new_and(0, 2),
                    Gate::new_and(1, 2),
                    Gate::new_and(0, 3),
                    Gate::new_and(1, 3),
                    // add a1*b0 + a0*b1
                    Gate::new_xor(5, 6),
                    Gate::new_and(5, 6),
                    // add c(a1*b0 + a0*b1) + a1*b1
                    Gate::new_xor(7, 9),
                    Gate::new_and(7, 9),
                ],
                [(4, false), (8, false), (10, false), (11, false)],
            )
            .unwrap(),
            [(0, false), (1, true), (2, true), (3, true)]
        )
    );

    assert_eq!(
        (
            Circuit::new(
                2,
                [
                    // Gate::new_and(0, 2), -> false
                    // Gate::new_xor(0, 2), -> true
                    Gate::new_nimpl(0, 0), // Gate::new_nor(1, 2),
                    Gate::new_nimpl(1, 1), // Gate::new_nor(2, 3),
                    // Gate::new_and(4, 5), -> false
                    // Gate::new_nimpl(4, 5), -> false
                    Gate::new_xor(2, 3),
                    Gate::new_nor(2, 3),
                    // Gate::new_and(8, 9), -> false
                    Gate::new_and(4, 5),
                    Gate::new_nimpl(6, 6),
                ],
                [(7, false)],
            )
            .unwrap(),
            vec![
                OutputEntry::Value(false),
                OutputEntry::NewIndex(0),
                OutputEntry::Value(true),
                OutputEntry::NewIndex(1)
            ],
            vec![
                OutputEntry::Value(true),
                OutputEntry::Value(true),
                OutputEntry::NewIndex(0)
            ],
        ),
        assign_to_circuit(
            &Circuit::new(
                4,
                [
                    Gate::new_and(0, 2),
                    Gate::new_xor(0, 2),
                    Gate::new_nor(1, 2),
                    Gate::new_nor(2, 3),
                    Gate::new_and(4, 5),
                    Gate::new_nimpl(4, 5),
                    Gate::new_xor(6, 7),
                    Gate::new_nor(6, 7),
                    Gate::new_and(8, 9),
                    Gate::new_and(10, 11),
                    Gate::new_and(12, 13),
                ],
                [(5, false), (9, true), (14, false)],
            )
            .unwrap(),
            [(0, false), (2, true)]
        )
    );
}

#[test]
fn test_optimize_clause_circuit() {
    assert_eq!(
        (ClauseCircuit::new(0, [], []).unwrap(), vec![], vec![]),
        optimize_clause_circuit(ClauseCircuit::new(0, [], []).unwrap())
    );

    assert_eq!(
        (
            ClauseCircuit::new(0, [], []).unwrap(),
            vec![None],
            vec![OutputEntry::Value(false)]
        ),
        optimize_clause_circuit(
            ClauseCircuit::new(1, [Clause::new_and([(0, false), (0, true)])], [(1, false)])
                .unwrap()
        )
    );

    assert_eq!(
        (
            ClauseCircuit::new(0, [], []).unwrap(),
            vec![None],
            vec![OutputEntry::Value(true)]
        ),
        optimize_clause_circuit(
            ClauseCircuit::new(1, [Clause::new_and([(0, false), (0, true)])], [(1, true)]).unwrap()
        )
    );

    assert_eq!(
        (
            ClauseCircuit::new(
                3,
                [
                    Clause::new_and([(0, false), (1, true), (2, false)]),
                    Clause::new_and([(2, false), (3, true)]),
                ],
                [(4, false)]
            )
            .unwrap(),
            vec![Some(0), Some(1), Some(2)],
            vec![OutputEntry::NewIndex(0)]
        ),
        optimize_clause_circuit(
            ClauseCircuit::new(
                3,
                [
                    Clause::new_and([(0, false), (1, true), (2, false)]),
                    Clause::new_and([(3, false), (3, false)]),
                    Clause::new_and([(4, true), (2, false)]),
                ],
                [(5, false)]
            )
            .unwrap()
        )
    );

    assert_eq!(
        (
            ClauseCircuit::new(
                3,
                [Clause::new_and([(0, false), (1, true), (2, false)]),],
                [(3, false)]
            )
            .unwrap(),
            vec![Some(0), Some(1), Some(2)],
            vec![OutputEntry::NewIndex(0)]
        ),
        optimize_clause_circuit(
            ClauseCircuit::new(
                3,
                [
                    Clause::new_and([(0, false), (1, true), (2, false)]),
                    Clause::new_and([(3, false), (3, false)]),
                    Clause::new_and([(4, false), (2, false)]),
                ],
                [(5, false)]
            )
            .unwrap()
        )
    );

    assert_eq!(
        (
            ClauseCircuit::new(0, [], []).unwrap(),
            vec![None, None, None],
            vec![OutputEntry::Value(false)]
        ),
        optimize_clause_circuit(
            ClauseCircuit::new(
                3,
                [
                    Clause::new_and([(0, false), (1, true), (2, false)]),
                    Clause::new_and([(3, false), (3, false)]),
                    Clause::new_and([(4, false), (2, true)]),
                ],
                [(5, false)]
            )
            .unwrap()
        )
    );

    assert_eq!(
        (
            ClauseCircuit::new(
                2,
                [Clause::new_xor([(0, false), (1, false)]),],
                [(2, false)]
            )
            .unwrap(),
            vec![None, Some(0), Some(1), None],
            vec![OutputEntry::NewIndex(0), OutputEntry::Value(false)]
        ),
        optimize_clause_circuit(
            ClauseCircuit::new(
                4,
                [
                    Clause::new_xor([(0, false), (1, true), (3, false)]),
                    Clause::new_xor([(0, false), (3, false), (2, true)]),
                    Clause::new_xor([(4, false), (5, false)]),
                    Clause::new_and([(0, false), (1, false)]),
                    Clause::new_and([(1, true), (2, false)]),
                    Clause::new_and([(7, false), (8, false)]),
                ],
                [(6, false), (9, false)]
            )
            .unwrap()
        )
    );

    assert_eq!(
        (
            ClauseCircuit::new(
                6,
                [
                    Clause::new_xor([(1, false), (2, false)]),
                    Clause::new_and([(3, false), (4, false)]),
                    Clause::new_and([(0, false), (1, false), (6, false), (7, true)]),
                    Clause::new_xor([(1, false), (3, false), (5, false), (6, false)]),
                    Clause::new_and([(8, true), (9, false)]),
                ],
                [(10, false)]
            )
            .unwrap(),
            vec![Some(0), Some(1), Some(2), Some(3), Some(4), Some(5)],
            vec![OutputEntry::NewIndex(0)]
        ),
        optimize_clause_circuit(
            ClauseCircuit::new(
                6,
                [
                    Clause::new_and([(0, false), (1, false)]),
                    Clause::new_xor([(1, false), (2, false)]),
                    Clause::new_and([(3, false), (4, false)]),
                    Clause::new_xor([(3, false), (5, true)]),
                    Clause::new_and([(1, false), (1, false)]),
                    Clause::new_and([(6, false), (7, false), (8, true), (10, false)]),
                    Clause::new_xor([(7, true), (9, false), (10, false)]),
                    Clause::new_and([(11, true), (12, false)]),
                ],
                [(13, false)]
            )
            .unwrap()
        )
    );

    assert_eq!(
        (
            ClauseCircuit::new(
                3,
                [Clause::new_xor([(0, false), (1, false), (2, false)]),],
                [(3, true)]
            )
            .unwrap(),
            vec![None, None, Some(0), Some(1), None, Some(2)],
            vec![OutputEntry::NewIndex(0)]
        ),
        optimize_clause_circuit(
            ClauseCircuit::new(
                6,
                [
                    Clause::new_and([(0, false), (1, false)]),
                    Clause::new_xor([(1, false), (2, false)]),
                    Clause::new_and([(3, false), (4, false)]),
                    Clause::new_xor([(3, false), (5, true)]),
                    Clause::new_xor([(1, true), (2, false), (2, false)]),
                    Clause::new_and([(6, false), (7, false), (8, true), (10, false)]),
                    Clause::new_xor([(7, true), (9, false), (10, false)]),
                    Clause::new_and([(11, true), (12, false)]),
                ],
                [(13, false)]
            )
            .unwrap()
        )
    );

    for tv in 0..4 {
        let t = (tv & 1) != 0;
        let t1 = (tv & 2) != 0;
        assert_eq!(
            (
                ClauseCircuit::new(
                    4,
                    [Clause::new_xor([
                        (0, false),
                        (1, false),
                        (2, false),
                        (3, false)
                    ]),],
                    [(4, t ^ t1)]
                )
                .unwrap(),
                vec![None, Some(0), Some(1), Some(2), None, Some(3)],
                vec![OutputEntry::NewIndex(0)]
            ),
            optimize_clause_circuit(
                ClauseCircuit::new(
                    6,
                    [
                        Clause::new_and([(0, false), (1, false)]),
                        Clause::new_xor([(1, false), (2, false)]),
                        Clause::new_and([(3, false), (4, false)]),
                        Clause::new_xor([(3, t1), (5, true)]),
                        Clause::new_and([(1, false), (1, true)]),
                        Clause::new_and([(6, false), (7, false), (8, true), (10, false)]),
                        Clause::new_xor([(7, true), (9, t), (10, false)]),
                        Clause::new_and([(11, true), (12, false)]),
                    ],
                    [(13, false)]
                )
                .unwrap()
            )
        );
    }

    assert_eq!(
        (
            ClauseCircuit::new(
                3,
                [Clause::new_and([(0, false), (1, false), (2, false)]),],
                [(3, false)]
            )
            .unwrap(),
            vec![Some(0), Some(1), Some(2)],
            vec![OutputEntry::NewIndex(0)]
        ),
        optimize_clause_circuit(
            ClauseCircuit::new(
                3,
                [
                    Clause::new_and([(0, false), (1, false)]),
                    Clause::new_xor([(3, false), (1, false), (1, false)]),
                    Clause::new_xor([(4, true), (2, true), (2, true)]),
                    Clause::new_and([(2, false), (5, true)]),
                ],
                [(6, false)]
            )
            .unwrap()
        )
    );

    assert_eq!(
        (
            ClauseCircuit::new(
                3,
                [Clause::new_and([(0, false), (1, false), (2, false)]),],
                [(3, false)]
            )
            .unwrap(),
            vec![Some(0), Some(1), Some(2)],
            vec![OutputEntry::NewIndex(0)]
        ),
        optimize_clause_circuit(
            ClauseCircuit::new(
                3,
                [
                    Clause::new_and([(0, false), (1, false)]),
                    Clause::new_and([(3, false), (3, false)]),
                    Clause::new_and([(4, true), (4, true)]),
                    Clause::new_and([(2, false), (5, true)]),
                ],
                [(6, false)]
            )
            .unwrap()
        )
    );

    assert_eq!(
        (
            ClauseCircuit::new(
                6,
                [Clause::new_and([
                    (0, false),
                    (1, false),
                    (2, false),
                    (3, false),
                    (4, false),
                    (5, false)
                ])],
                [(6, false)]
            )
            .unwrap(),
            vec![Some(0), Some(1), Some(2), Some(3), Some(4), Some(5)],
            vec![OutputEntry::NewIndex(0)]
        ),
        optimize_clause_circuit(
            ClauseCircuit::new(
                6,
                [
                    Clause::new_and([(2, false), (0, false), (3, false), (4, false)]),
                    Clause::new_and([(6, false), (6, false)]),
                    Clause::new_and([(7, false), (7, false)]),
                    Clause::new_and([(0, false), (8, false), (5, false), (7, false), (1, false)]),
                ],
                [(9, false)]
            )
            .unwrap()
        )
    );

    assert_eq!(
        (
            ClauseCircuit::new(
                6,
                [Clause::new_and([
                    (0, false),
                    (1, false),
                    (2, false),
                    (3, false),
                    (4, false),
                    (5, false)
                ])],
                [(6, false)]
            )
            .unwrap(),
            vec![Some(0), Some(1), Some(2), Some(3), Some(4), Some(5)],
            vec![OutputEntry::NewIndex(0)]
        ),
        optimize_clause_circuit(
            ClauseCircuit::new(
                6,
                [
                    Clause::new_and([(2, false), (0, false), (3, false), (4, false)]),
                    Clause::new_and([(6, false), (6, false)]),
                    Clause::new_and([(0, false), (4, false), (6, false)]),
                    Clause::new_and([(0, false), (8, false), (5, false), (7, false), (1, false)]),
                ],
                [(9, false)]
            )
            .unwrap()
        )
    );

    for tv in 0..64 {
        let t0 = (tv & 1) != 0;
        let t1 = (tv & 2) != 0;
        let t2 = (tv & 4) != 0;
        let t3 = (tv & 8) != 0;
        let t4 = (tv & 16) != 0;
        let t5 = (tv & 32) != 0;
        assert_eq!(
            (
                ClauseCircuit::new(
                    3,
                    [Clause::new_xor([(0, false), (1, false), (2, false),])],
                    [(3, t0 ^ t1 ^ t2 ^ t3 ^ t5)]
                )
                .unwrap(),
                vec![None, Some(0), None, None, Some(1), Some(2)],
                vec![OutputEntry::NewIndex(0)]
            ),
            optimize_clause_circuit(
                ClauseCircuit::new(
                    6,
                    [
                        Clause::new_xor([(2, false), (0, t4), (3, false), (4, false)]),
                        Clause::new_xor([(6, t0), (1, false), (1, false)]),
                        Clause::new_xor([(0, false), (4, t1), (6, false)]),
                        Clause::new_xor([(0, false), (8, t2), (5, false), (7, t3), (1, false)]),
                    ],
                    [(9, t5)]
                )
                .unwrap()
            )
        );
    }

    for tv in 0..128 {
        let t0 = (tv & 1) != 0;
        let t1 = (tv & 2) != 0;
        let t2 = (tv & 4) != 0;
        let t3 = (tv & 8) != 0;
        let t4 = (tv & 16) != 0;
        let t5 = (tv & 32) != 0;
        let t6 = (tv & 64) != 0;
        assert_eq!(
            (
                ClauseCircuit::new(
                    5,
                    [Clause::new_xor([
                        (0, false),
                        (1, false),
                        (2, false),
                        (3, false),
                        (4, false)
                    ])],
                    [(5, t0 ^ t1 ^ t2 ^ t3 ^ t4 ^ t5 ^ t6)]
                )
                .unwrap(),
                vec![Some(0), Some(1), Some(2), Some(3), None, Some(4)],
                vec![OutputEntry::NewIndex(0)]
            ),
            optimize_clause_circuit(
                ClauseCircuit::new(
                    6,
                    [
                        Clause::new_xor([(2, false), (0, t4), (3, false), (4, false)]),
                        Clause::new_xor([(6, t0), (1, false), (1, false)]),
                        Clause::new_xor([(0, false), (4, t1), (6, false)]),
                        Clause::new_xor([
                            (0, false),
                            (8, t2),
                            (5, false),
                            (7, t3),
                            (1, t6),
                            (6, t5)
                        ]),
                    ],
                    [(9, false)]
                )
                .unwrap()
            )
        );
    }

    assert_eq!(
        (
            ClauseCircuit::new(1, [], [(0, false)]).unwrap(),
            vec![None, Some(0), None, None],
            vec![OutputEntry::Value(false), OutputEntry::NewIndex(0)]
        ),
        optimize_clause_circuit(
            ClauseCircuit::new(
                4,
                [
                    Clause::new_and([(2, false), (0, true)]),
                    Clause::new_xor([(3, false), (1, true)]),
                    Clause::new_and([(3, false), (0, true)]),
                    Clause::new_and([(0, false), (2, true)]),
                    Clause::new_and([(5, false), (4, true), (6, false), (7, false)]),
                    Clause::new_xor([(0, false), (5, true)]),
                    Clause::new_xor([(9, false), (3, false), (0, false)]),
                ],
                [(8, false), (10, false)]
            )
            .unwrap()
        )
    );

    assert_eq!(
        (
            ClauseCircuit::new(
                4,
                [
                    Clause::new_xor([(2, false), (3, false)]),
                    Clause::new_xor([(0, false), (2, false), (3, false)]),
                ],
                [(4, true), (0, false), (1, false), (5, false)]
            )
            .unwrap(),
            vec![Some(0), Some(1), Some(2), Some(3)],
            vec![
                OutputEntry::Value(false),
                OutputEntry::NewIndex(0),
                OutputEntry::NewIndex(1),
                OutputEntry::Value(true),
                OutputEntry::NewIndex(2),
                OutputEntry::NewIndex(3),
            ]
        ),
        optimize_clause_circuit(
            ClauseCircuit::new(
                4,
                [
                    // false
                    Clause::new_and([(2, false), (0, true), (0, false)]),
                    // clause xor
                    Clause::new_xor([(2, false), (3, true)]),
                    // (0, true)
                    Clause::new_xor([(1, false), (0, true), (1, false)]),
                    // false
                    Clause::new_and([(0, false), (3, true), (3, false)]),
                    // (1, false)
                    Clause::new_and([(1, false), (1, false)]),
                    // clause xor
                    Clause::new_xor([(0, false), (2, true), (3, false)]),
                ],
                [
                    (4, false),
                    (5, false),
                    (6, true),
                    (7, true),
                    (8, false),
                    (9, true)
                ]
            )
            .unwrap()
        )
    );

    assert_eq!(
        (
            ClauseCircuit::new(
                3,
                [
                    Clause::new_xor([(1, false), (2, false)]),
                    Clause::new_xor([(0, false), (2, false)]),
                ],
                [(3, true), (1, false), (4, false)]
            )
            .unwrap(),
            vec![None, Some(0), Some(1), Some(2)],
            vec![
                OutputEntry::Value(false),
                OutputEntry::NewIndex(0),
                OutputEntry::NewIndex(1),
                OutputEntry::Value(true),
                OutputEntry::Value(false),
                OutputEntry::NewIndex(2),
            ]
        ),
        optimize_clause_circuit(
            ClauseCircuit::new(
                4,
                [
                    // false
                    Clause::new_and([(2, false), (0, true), (0, false)]),
                    // clause xor
                    Clause::new_xor([(2, false), (3, true)]),
                    // (2, true)
                    Clause::new_xor([(1, false), (2, true), (1, false)]),
                    // false
                    Clause::new_and([(0, false), (3, true), (3, false)]),
                    // false
                    Clause::new_and([(1, false), (1, true)]),
                    // clause xor
                    Clause::new_xor([(1, true), (3, false)]),
                ],
                [
                    (4, false),
                    (5, false),
                    (6, true),
                    (7, true),
                    (8, false),
                    (9, true)
                ]
            )
            .unwrap()
        )
    );

    assert_eq!(
        (
            ClauseCircuit::new(
                2,
                [
                    Clause::new_xor([(0, false), (1, false)]),
                    Clause::new_xor([(0, false), (1, false)]),
                ],
                [(2, true), (0, false), (3, false)]
            )
            .unwrap(),
            vec![None, None, Some(0), Some(1)],
            vec![
                OutputEntry::Value(false),
                OutputEntry::NewIndex(0),
                OutputEntry::NewIndex(1),
                OutputEntry::Value(true),
                OutputEntry::Value(false),
                OutputEntry::NewIndex(2),
            ]
        ),
        optimize_clause_circuit(
            ClauseCircuit::new(
                4,
                [
                    // false
                    Clause::new_and([(2, false), (0, true), (0, false)]),
                    // clause xor
                    Clause::new_xor([(2, false), (3, true)]),
                    // (2, true)
                    Clause::new_xor([(1, false), (2, true), (1, false)]),
                    // false
                    Clause::new_and([(0, false), (3, true), (3, false)]),
                    // false
                    Clause::new_and([(1, false), (1, true)]),
                    // clause xor
                    Clause::new_xor([(2, true), (3, false)]),
                ],
                [
                    (4, false),
                    (5, false),
                    (6, true),
                    (7, true),
                    (8, false),
                    (9, true)
                ]
            )
            .unwrap()
        )
    );

    assert_eq!(
        (
            ClauseCircuit::new(
                4,
                [
                    Clause::new_xor([(0, false), (1, false)]),
                    Clause::new_xor([(2, false), (3, false)]),
                    Clause::new_xor([(4, false), (5, true)]),
                    Clause::new_and([(4, false), (5, true)]),
                    Clause::new_and([(6, false), (7, true)]),
                ],
                [(8, false)]
            )
            .unwrap(),
            vec![Some(0), Some(1), Some(2), Some(3)],
            vec![OutputEntry::NewIndex(0),]
        ),
        optimize_clause_circuit(
            ClauseCircuit::new(
                4,
                [
                    Clause::new_xor([(0, false), (1, false)]),
                    Clause::new_xor([(2, false), (3, true)]),
                    Clause::new_xor([(4, false), (5, false)]),
                    Clause::new_and([(4, false), (5, false)]),
                    Clause::new_and([(6, false), (7, true)]),
                ],
                [(8, false)]
            )
            .unwrap()
        )
    );
}

#[test]
fn test_assign_to_circuit_and_optimize() {
    assert_eq!(
        (
            Circuit::new(
                3,
                [
                    Gate::new_and(0, 1),
                    Gate::new_and(0, 2),
                    // add a1*b0 + a0*b1
                    Gate::new_xor(2, 3),
                    Gate::new_and(2, 3),
                    // add c(a1*b0 + a0*b1) + a1*b1
                    Gate::new_xor(4, 6),
                    Gate::new_and(4, 6),
                ],
                [(1, false), (5, false), (7, false), (8, false)],
            )
            .unwrap(),
            vec![
                OutputEntry::Value(true),
                OutputEntry::NewIndex(0),
                OutputEntry::NewIndex(1),
                OutputEntry::NewIndex(2)
            ],
            vec![
                OutputEntry::NewIndex(0),
                OutputEntry::NewIndex(1),
                OutputEntry::NewIndex(2),
                OutputEntry::NewIndex(3)
            ],
        ),
        assign_to_circuit_and_optimize(
            &Circuit::new(
                4,
                [
                    Gate::new_and(0, 2),
                    Gate::new_and(1, 2),
                    Gate::new_and(0, 3),
                    Gate::new_and(1, 3),
                    // add a1*b0 + a0*b1
                    Gate::new_xor(5, 6),
                    Gate::new_and(5, 6),
                    // add c(a1*b0 + a0*b1) + a1*b1
                    Gate::new_xor(7, 9),
                    Gate::new_and(7, 9),
                ],
                [(4, false), (8, false), (10, false), (11, false)],
            )
            .unwrap(),
            [(0, true)],
            false
        )
    );

    assert_eq!(
        (
            Circuit::new(
                4,
                [
                    Gate::new_nor(2, 3),
                    Gate::new_xor(1, 4), // out0
                    Gate::new_xor(0, 1),
                    Gate::new_nor(1, 6), // out2
                ],
                [(5, false), (7, false)],
            )
            .unwrap(),
            vec![
                OutputEntry::NewIndex(0),
                OutputEntry::Value(false),
                OutputEntry::Value(true),
                OutputEntry::NewIndex(1),
                OutputEntry::NewIndex(2),
                OutputEntry::NewIndex(3)
            ],
            vec![
                OutputEntry::NewIndex(0),
                OutputEntry::Value(true),
                OutputEntry::NewIndex(1)
            ],
        ),
        assign_to_circuit_and_optimize(
            &Circuit::new(
                6,
                [
                    Gate::new_and(0, 1), // false
                    Gate::new_and(2, 3), // 3
                    Gate::new_nor(4, 5),
                    Gate::new_xor(6, 7),
                    Gate::new_xor(8, 9), // out0
                    Gate::new_and(3, 7),
                    Gate::new_and(2, 6),
                    Gate::new_and(11, 12), // out1=false -> true
                    Gate::new_xor(0, 3),
                    Gate::new_xor(1, 3),
                    Gate::new_nor(14, 15), // out2
                ],
                [(10, false), (13, true), (16, false)],
            )
            .unwrap(),
            [(1, false), (2, true)],
            false
        )
    );

    assert_eq!(
        (
            Circuit::new(
                3,
                [Gate::new_nimpl(0, 2), Gate::new_and(0, 1),],
                [(2, false), (3, false), (4, true)],
            )
            .unwrap(),
            vec![
                OutputEntry::NewIndex(0),
                OutputEntry::Value(true),
                OutputEntry::NewIndex(1),
                OutputEntry::Value(false),
                OutputEntry::Value(false),
                OutputEntry::NewIndex(2)
            ],
            vec![
                OutputEntry::NewIndex(0),
                OutputEntry::Value(true),
                OutputEntry::NewIndex(1),
                OutputEntry::NewIndex(2)
            ],
        ),
        assign_to_circuit_and_optimize(
            &Circuit::new(
                6,
                [
                    Gate::new_and(0, 4),   // false
                    Gate::new_and(1, 5),   // 1
                    Gate::new_nimpl(7, 6), // out0=5
                    Gate::new_xor(2, 3),
                    Gate::new_xor(1, 5),
                    Gate::new_nor(9, 10),
                    Gate::new_and(4, 11), // (out1=false,true)=true
                    Gate::new_nimpl(0, 5),
                    Gate::new_and(1, 4),   // false
                    Gate::new_xor(13, 14), // out2=nimpl(0,5)
                    Gate::new_and(2, 4),
                    Gate::new_and(0, 2),
                    Gate::new_xor(16, 17), // out3=and(0,2)
                ],
                [(8, false), (12, true), (15, false), (18, true)],
            )
            .unwrap(),
            [(1, true), (4, false)],
            false
        )
    );

    assert_eq!(
        (
            Circuit::new(
                3,
                [Gate::new_nimpl(0, 2), Gate::new_and(0, 1),],
                [(2, false), (3, false), (4, true)],
            )
            .unwrap(),
            vec![
                OutputEntry::NewIndex(0),
                OutputEntry::Value(true),
                OutputEntry::NewIndex(1),
                OutputEntry::Value(false),
                OutputEntry::Value(false),
                OutputEntry::NewIndex(2)
            ],
            vec![
                OutputEntry::NewIndex(0),
                OutputEntry::Value(true),
                OutputEntry::NewIndex(1),
                OutputEntry::Value(false),
                OutputEntry::NewIndex(2)
            ],
        ),
        assign_to_circuit_and_optimize(
            &Circuit::new(
                6,
                [
                    Gate::new_and(0, 4),   // false
                    Gate::new_and(1, 5),   // 1
                    Gate::new_nimpl(7, 6), // out0=5
                    Gate::new_xor(2, 3),
                    Gate::new_xor(1, 5),
                    Gate::new_nor(9, 10),
                    Gate::new_and(4, 11), // (out1=false,true)=true
                    Gate::new_nimpl(0, 5),
                    Gate::new_and(1, 4),   // false
                    Gate::new_xor(13, 14), // out2=nimpl(0,5)
                    Gate::new_and(1, 4),
                    Gate::new_and(2, 4),
                    Gate::new_and(0, 2),
                    Gate::new_xor(17, 18), // out3=and(0,2)
                ],
                [(8, false), (12, true), (15, false), (16, false), (19, true)],
            )
            .unwrap(),
            [(1, true), (4, false)],
            false
        )
    );
}

#[test]
fn test_deduplicate_clause_circuit() {
    assert_eq!(
        (
            ClauseCircuit::new(
                4,
                [
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(0, false), (2, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (4, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (5, false)]
                    },
                    Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(6, false), (5, false)]
                    },
                    Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(2, false), (7, false)]
                    },
                    Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(4, false), (8, false)]
                    }
                ],
                [(7, false), (8, true), (9, false)]
            )
            .unwrap(),
            false
        ),
        deduplicate_clause_circuit(
            ClauseCircuit::new(
                4,
                [
                    Clause::new_and([(0, false), (1, false), (2, false), (3, false)]),
                    Clause::new_and([(0, false), (1, false), (2, false)]),
                    Clause::new_xor([(4, false), (5, false)]),
                    Clause::new_and([(0, false), (2, false)]),
                    Clause::new_xor([(2, false), (4, false), (5, false)]),
                    Clause::new_xor([(2, false), (4, false), (5, false), (7, false)]),
                ],
                [(6, false), (8, true), (9, false)]
            )
            .unwrap()
        ),
    );

    assert_eq!(
        (
            ClauseCircuit::new(
                4,
                [
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(0, false), (2, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (4, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (5, false)]
                    },
                    Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(6, false), (5, false)]
                    },
                    Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(2, false), (7, false)]
                    },
                    Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(4, false), (8, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (7, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(6, false), (10, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(11, false), (8, false)]
                    }
                ],
                [
                    (7, false),
                    (8, true),
                    (9, false),
                    (12, true),
                    (11, false),
                    (10, false)
                ]
            )
            .unwrap(),
            false
        ),
        deduplicate_clause_circuit(
            ClauseCircuit::new(
                4,
                [
                    Clause::new_and([(0, false), (1, false), (2, false), (3, false)]),
                    Clause::new_and([(0, false), (1, false), (2, false)]),
                    Clause::new_xor([(4, false), (5, false)]),
                    Clause::new_and([(0, false), (2, false)]),
                    Clause::new_xor([(2, false), (4, false), (5, false)]),
                    Clause::new_xor([(2, false), (4, false), (5, false), (7, false)]),
                    Clause::new_and([(1, false), (4, false), (6, false), (8, false)]),
                    Clause::new_and([(1, false), (4, false), (6, false)]),
                    Clause::new_and([(1, false), (6, false)]),
                ],
                [
                    (6, false),
                    (8, true),
                    (9, false),
                    (10, true),
                    (11, false),
                    (12, false)
                ]
            )
            .unwrap()
        ),
    );

    assert_eq!(
        (
            ClauseCircuit::new(
                4,
                [
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(0, false), (2, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (4, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (5, false)]
                    },
                    Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(6, false), (5, false)]
                    },
                    Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(2, false), (7, false)]
                    },
                    Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(4, false), (8, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (7, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(6, false), (10, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(11, false), (8, false)]
                    }
                ],
                [
                    (8, false),
                    (9, true),
                    (7, false),
                    (12, true),
                    (11, false),
                    (10, false)
                ]
            )
            .unwrap(),
            false
        ),
        deduplicate_clause_circuit(
            ClauseCircuit::new(
                4,
                [
                    Clause::new_and([(0, false), (1, false), (2, false), (3, false)]),
                    Clause::new_and([(0, false), (1, false), (2, false)]),
                    Clause::new_xor([(2, false), (4, false), (5, false)]),
                    Clause::new_and([(0, false), (2, false)]),
                    Clause::new_xor([(2, false), (4, false), (5, false), (7, false)]),
                    Clause::new_xor([(4, false), (5, false)]),
                    Clause::new_and([(1, false), (4, false), (6, false), (9, false)]),
                    Clause::new_and([(1, false), (4, false), (9, false)]),
                    Clause::new_and([(1, false), (9, false)]),
                ],
                [
                    (6, false),
                    (8, true),
                    (9, false),
                    (10, true),
                    (11, false),
                    (12, false)
                ]
            )
            .unwrap()
        ),
    );

    for i in 0..2 {
        assert_eq!(
            (
                ClauseCircuit::new(
                    3,
                    [
                        Clause::new_and([(0, false), (1, false), (2, false)]),
                        Clause::new_xor([(0, false), (1, false), (2, false)]),
                        Clause::new_and([(3, true), (4, false)]),
                        Clause::new_and([(1, true), (5, false)]),
                        Clause::new_xor([(2, false), (5, true)]),
                    ],
                    [(5, false), (6, false), (7, true)]
                )
                .unwrap(),
                false
            ),
            deduplicate_clause_circuit(
                ClauseCircuit::new(
                    3,
                    [
                        Clause::new_and([(0, false), (1, false), (2, false)]),
                        Clause::new_xor([(0, false), (1, false), (2, false)]),
                        Clause::new_and([(3, true), (4, false)]),
                        Clause::new_and([(3, true), (4, false)]),
                        Clause::new_and([(1, true), (5, false)]),
                        Clause::new_xor([(2, false), (6, true)]),
                    ],
                    [(5 + i, false), (7, false), (8, true)]
                )
                .unwrap()
            ),
        );
    }

    assert_eq!(
        (
            ClauseCircuit::new(
                3,
                [
                    Clause::new_and([(0, false), (1, false), (2, false)]),
                    Clause::new_xor([(0, false), (1, false), (2, false)]),
                    Clause::new_and([(3, true), (4, false)]),
                    Clause::new_and([(5, false), (5, false)]),
                ],
                [(5, false), (6, true)]
            )
            .unwrap(),
            true
        ),
        deduplicate_clause_circuit(
            ClauseCircuit::new(
                3,
                [
                    Clause::new_and([(0, false), (1, false), (2, false)]),
                    Clause::new_xor([(0, false), (1, false), (2, false)]),
                    Clause::new_and([(3, true), (4, false)]),
                    Clause::new_and([(3, true), (4, false)]),
                    Clause::new_and([(5, false), (6, false)]),
                ],
                [(6, false), (7, true)]
            )
            .unwrap()
        ),
    );

    assert_eq!(
        (
            ClauseCircuit::new(
                3,
                [
                    Clause::new_xor([(0, false), (1, false), (2, false)]),
                    Clause::new_and([(0, false), (1, false), (2, false)]),
                    Clause::new_xor([(3, true), (4, false)]),
                    Clause::new_xor([(5, false), (5, false)]),
                ],
                [(5, false), (6, true)]
            )
            .unwrap(),
            true
        ),
        deduplicate_clause_circuit(
            ClauseCircuit::new(
                3,
                [
                    Clause::new_xor([(0, false), (1, false), (2, false)]),
                    Clause::new_and([(0, false), (1, false), (2, false)]),
                    Clause::new_xor([(3, true), (4, false)]),
                    Clause::new_xor([(3, true), (4, false)]),
                    Clause::new_xor([(5, false), (6, false)]),
                ],
                [(6, false), (7, true)]
            )
            .unwrap()
        ),
    );

    let circuit = ClauseCircuit::new(
        4,
        [
            Clause::new_xor([(0, false), (1, false)]),
            Clause::new_xor([(2, false), (3, false)]),
            Clause::new_xor([(4, false), (5, true)]),
            Clause::new_and([(4, false), (5, true)]),
            Clause::new_and([(6, false), (7, true)]),
        ],
        [(8, false)],
    )
    .unwrap();
    assert_eq!(
        (circuit.clone(), false),
        deduplicate_clause_circuit(circuit),
    );

    // important testcase: old extra clause reordering avoiding.
    assert_eq!(
        (
            ClauseCircuit::new(
                9,
                [
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (2, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (9, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(5, false), (6, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(4, false), (9, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(11, false), (12, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (13, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(7, false), (13, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(0, false), (8, false), (11, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(8, false), (9, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(0, false), (1, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(5, false), (7, false)]
                    }
                ],
                [
                    (10, false),
                    (14, true),
                    (13, false),
                    (15, true),
                    (11, true),
                    (16, false),
                    (17, true),
                    (18, false),
                    (19, false)
                ]
            )
            .unwrap(),
            false
        ),
        deduplicate_clause_circuit(
            ClauseCircuit::new(
                9,
                [
                    Clause::new_and([(1, false), (2, false), (3, false)]),
                    Clause::new_and([
                        (1, false),
                        (2, false),
                        (3, false),
                        (4, false),
                        (5, false),
                        (6, false),
                    ]),
                    Clause::new_and([(1, false), (2, false), (4, false), (5, false), (6, false)]),
                    Clause::new_and([
                        (1, false),
                        (2, false),
                        (4, false),
                        (5, false),
                        (6, false),
                        (7, false),
                    ]),
                    Clause::new_and([(5, false), (6, false)]),
                    Clause::new_and([(0, false), (5, false), (6, false), (8, false)]),
                    Clause::new_and([(1, false), (2, false), (8, false)]),
                    Clause::new_and([(0, false), (1, false)]),
                    Clause::new_and([(5, false), (7, false)]),
                ],
                [
                    (9, false),
                    (10, true),
                    (11, false),
                    (12, true),
                    (13, true),
                    (14, false),
                    (15, true),
                    (16, false),
                    (17, false)
                ]
            )
            .unwrap()
        ),
    );

    assert_eq!(
        (
            ClauseCircuit::new(
                8,
                [
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(2, false), (5, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (3, false), (4, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(8, false), (9, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(0, false), (6, false), (7, false), (10, false)]
                    }
                ],
                [(11, false), (10, true), (9, false)]
            )
            .unwrap(),
            false
        ),
        deduplicate_clause_circuit(
            ClauseCircuit::new(
                8,
                [
                    Clause::new_and([
                        (0, false),
                        (1, false),
                        (2, false),
                        (3, false),
                        (4, false),
                        (5, false),
                        (6, false),
                        (7, false)
                    ]),
                    Clause::new_and([(1, false), (2, false), (3, false), (4, false), (5, false)]),
                    Clause::new_and([(1, false), (3, false), (4, false)]),
                ],
                [(8, false), (9, true), (10, false)]
            )
            .unwrap()
        ),
    );

    assert_eq!(
        (
            ClauseCircuit::new(
                8,
                [
                    Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(2, false), (5, false)]
                    },
                    Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(1, false), (3, false), (4, false)]
                    },
                    Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(8, false), (9, false)]
                    },
                    Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(0, false), (6, false), (7, false), (10, false)]
                    }
                ],
                [(11, false), (10, true), (9, false)]
            )
            .unwrap(),
            false
        ),
        deduplicate_clause_circuit(
            ClauseCircuit::new(
                8,
                [
                    Clause::new_xor([
                        (0, false),
                        (1, false),
                        (2, false),
                        (3, false),
                        (4, false),
                        (5, false),
                        (6, false),
                        (7, false)
                    ]),
                    Clause::new_xor([(1, false), (2, false), (3, false), (4, false), (5, false)]),
                    Clause::new_xor([(1, false), (3, false), (4, false)]),
                ],
                [(8, false), (9, true), (10, false)]
            )
            .unwrap()
        ),
    );

    assert_eq!(
        (
            ClauseCircuit::new(
                4,
                [
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(0, false), (2, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (4, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (5, false)]
                    }
                ],
                [(6, false), (5, true), (5, false), (4, false), (4, false)]
            )
            .unwrap(),
            false
        ),
        deduplicate_clause_circuit(
            ClauseCircuit::new(
                4,
                [
                    Clause::new_and([(0, false), (1, false), (2, false), (3, false)]),
                    Clause::new_and([(0, false), (1, false), (2, false)]),
                    Clause::new_and([(0, false), (1, false), (2, false)]),
                    Clause::new_and([(0, false), (2, false)]),
                    Clause::new_and([(0, false), (2, false)]),
                ],
                [(4, false), (5, true), (6, false), (7, false), (8, false)]
            )
            .unwrap()
        ),
    );

    assert_eq!(
        (
            ClauseCircuit::new(
                4,
                [
                    Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(0, false), (2, false)]
                    },
                    Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(1, false), (4, false)]
                    },
                    Clause {
                        kind: ClauseKind::Xor,
                        literals: vec![(3, false), (5, false)]
                    }
                ],
                [(6, false), (5, true), (5, false), (4, false), (4, false)]
            )
            .unwrap(),
            false
        ),
        deduplicate_clause_circuit(
            ClauseCircuit::new(
                4,
                [
                    Clause::new_xor([(0, false), (1, false), (2, false), (3, false)]),
                    Clause::new_xor([(0, false), (1, false), (2, false)]),
                    Clause::new_xor([(0, false), (1, false), (2, false)]),
                    Clause::new_xor([(0, false), (2, false)]),
                    Clause::new_xor([(0, false), (2, false)]),
                ],
                [(4, false), (5, true), (6, false), (7, false), (8, false)]
            )
            .unwrap()
        ),
    );
}

#[test]
fn test_optimize_and_dedup_clause_circuit() {
    assert_eq!(
        (
            ClauseCircuit::new(
                9,
                [
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (2, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (9, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(5, false), (6, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(4, false), (9, false), (11, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (12, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(7, false), (12, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(0, false), (8, false), (11, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(8, false), (9, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(0, false), (1, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(5, false), (7, false)]
                    }
                ],
                [
                    (10, false),
                    (13, true),
                    (12, false),
                    (14, true),
                    (11, true),
                    (15, false),
                    (16, true),
                    (17, false),
                    (18, false)
                ]
            )
            .unwrap(),
            (0..9).map(|x| Some(x)).collect::<Vec<_>>(),
            (0..9).map(|x| OutputEntry::NewIndex(x)).collect::<Vec<_>>(),
        ),
        optimize_and_dedup_clause_circuit(
            ClauseCircuit::new(
                9,
                [
                    Clause::new_and([(1, false), (2, false), (3, false)]),
                    Clause::new_and([
                        (1, false),
                        (2, false),
                        (3, false),
                        (4, false),
                        (5, false),
                        (6, false),
                    ]),
                    Clause::new_and([(1, false), (2, false), (4, false), (5, false), (6, false)]),
                    Clause::new_and([
                        (1, false),
                        (2, false),
                        (4, false),
                        (5, false),
                        (6, false),
                        (7, false),
                    ]),
                    Clause::new_and([(5, false), (6, false)]),
                    Clause::new_and([(0, false), (5, false), (6, false), (8, false)]),
                    Clause::new_and([(1, false), (2, false), (8, false)]),
                    Clause::new_and([(0, false), (1, false)]),
                    Clause::new_and([(5, false), (7, false)]),
                ],
                [
                    (9, false),
                    (10, true),
                    (11, false),
                    (12, true),
                    (13, true),
                    (14, false),
                    (15, true),
                    (16, false),
                    (17, false)
                ]
            )
            .unwrap()
        ),
    );

    // unordered version of previous testcase
    assert_eq!(
        (
            ClauseCircuit::new(
                9,
                [
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(1, false), (2, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (9, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(5, false), (6, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(4, false), (9, false), (11, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(3, false), (12, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(7, false), (12, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(0, false), (8, false), (11, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(8, false), (9, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(0, false), (1, false)]
                    },
                    Clause {
                        kind: ClauseKind::And,
                        literals: vec![(5, false), (7, false)]
                    }
                ],
                [
                    (10, false),
                    (13, true),
                    (12, false),
                    (14, true),
                    (11, true),
                    (15, false),
                    (16, true),
                    (17, false),
                    (18, false)
                ]
            )
            .unwrap(),
            (0..9).map(|x| Some(x)).collect::<Vec<_>>(),
            (0..9).map(|x| OutputEntry::NewIndex(x)).collect::<Vec<_>>(),
        ),
        optimize_and_dedup_clause_circuit(
            ClauseCircuit::new(
                9,
                [
                    Clause::new_and([(1, false), (3, false), (2, false)]),
                    Clause::new_and([
                        (3, false),
                        (1, false),
                        (6, false),
                        (5, false),
                        (4, false),
                        (2, false),
                    ]),
                    Clause::new_and([(5, false), (1, false), (6, false), (2, false), (4, false)]),
                    Clause::new_and([
                        (2, false),
                        (1, false),
                        (7, false),
                        (5, false),
                        (6, false),
                        (4, false),
                    ]),
                    Clause::new_and([(6, false), (5, false)]),
                    Clause::new_and([(6, false), (0, false), (8, false), (5, false)]),
                    Clause::new_and([(2, false), (8, false), (1, false)]),
                    Clause::new_and([(0, false), (1, false)]),
                    Clause::new_and([(7, false), (5, false)]),
                ],
                [
                    (9, false),
                    (10, true),
                    (11, false),
                    (12, true),
                    (13, true),
                    (14, false),
                    (15, true),
                    (16, false),
                    (17, false)
                ]
            )
            .unwrap()
        ),
    );

    assert_eq!(
        (
            ClauseCircuit::new(2, [Clause::new_xor([(0, false), (1, false)])], [(2, false)])
                .unwrap(),
            vec![Some(0), None, None, Some(1), None],
            vec![
                OutputEntry::Value(false),
                OutputEntry::NewIndex(0),
                OutputEntry::Value(false)
            ],
        ),
        optimize_and_dedup_clause_circuit(
            ClauseCircuit::new(
                5,
                [
                    Clause::new_and([(0, false), (1, false), (1, true)]), // 5
                    Clause::new_and([(0, false), (5, false)]),            // 6: false
                    Clause::new_xor([(2, false), (4, true)]),             // 7
                    Clause::new_and([(7, true), (3, true)]),              // 8
                    Clause::new_xor([(2, false), (4, true)]),             // 9
                    Clause::new_and([(4, true), (9, false), (8, false)]), // 10: false
                    Clause::new_xor([(0, false), (3, false)]),            // 11
                ],
                [(6, false), (11, false), (10, false)]
            )
            .unwrap()
        ),
    );

    for t in [false, true] {
        assert_eq!(
            (
                ClauseCircuit::new(2, [Clause::new_xor([(0, false), (1, false)])], [(2, false)])
                    .unwrap(),
                vec![Some(0), None, None, Some(1), None],
                vec![
                    OutputEntry::Value(false),
                    OutputEntry::NewIndex(0),
                    OutputEntry::Value(false),
                    OutputEntry::Value(t),
                ],
            ),
            optimize_and_dedup_clause_circuit(
                ClauseCircuit::new(
                    5,
                    [
                        Clause::new_and([(0, false), (1, false), (1, true)]), // 5
                        Clause::new_and([(0, false), (5, false)]),            // 6: false
                        Clause::new_xor([(2, false), (4, true)]),             // 7
                        Clause::new_and([(7, true), (3, true)]),              // 8
                        Clause::new_xor([(2, false), (4, true)]),             // 9
                        Clause::new_and([(4, true), (9, false), (8, false)]), // 10: false
                        Clause::new_xor([(0, false), (3, false)]),            // 11
                        Clause::new_and([(0, false), (4, false)]),            // 12
                        Clause::new_and([(0, false), (4, false)]),            // 13
                        Clause::new_xor([(12, false), (13, t)]),              // 14: false|true
                    ],
                    [(6, false), (11, false), (10, false), (14, false)]
                )
                .unwrap()
            ),
        );
    }

    assert_eq!(
        (
            ClauseCircuit::new(
                3,
                [
                    Clause::new_xor([(0, false), (1, false)]),
                    Clause::new_and([(0, false), (2, false)])
                ],
                [(3, false), (4, true)]
            )
            .unwrap(),
            vec![Some(0), None, None, Some(1), Some(2)],
            vec![
                OutputEntry::Value(false),
                OutputEntry::NewIndex(0),
                OutputEntry::Value(false),
                OutputEntry::NewIndex(1),
            ],
        ),
        optimize_and_dedup_clause_circuit(
            ClauseCircuit::new(
                5,
                [
                    Clause::new_and([(0, false), (1, false), (1, true)]), // 5
                    Clause::new_and([(0, false), (5, false)]),            // 6: false
                    Clause::new_xor([(2, false), (4, true)]),             // 7
                    Clause::new_and([(7, true), (3, true)]),              // 8
                    Clause::new_xor([(2, false), (4, true)]),             // 9
                    Clause::new_and([(4, true), (9, false), (8, false)]), // 10: false
                    Clause::new_xor([(0, false), (3, false)]),            // 11
                    Clause::new_and([(0, false), (4, false)]),            // 12
                    Clause::new_and([(0, false), (4, false)]),            // 13
                    Clause::new_and([(12, true), (13, true)]),            // 14: 12
                ],
                [(6, false), (11, false), (10, false), (14, false)]
            )
            .unwrap()
        ),
    );
}

#[test]
fn test_assign_to_circuit_optimize_and_dedup() {
    assert_eq!(
        (
            Circuit::new(
                3,
                [Gate::new_nimpl(0, 2), Gate::new_and(0, 1),],
                [(2, false), (3, false), (4, true)],
            )
            .unwrap(),
            vec![
                OutputEntry::NewIndex(0),
                OutputEntry::Value(true),
                OutputEntry::NewIndex(1),
                OutputEntry::Value(false),
                OutputEntry::Value(false),
                OutputEntry::NewIndex(2)
            ],
            vec![
                OutputEntry::NewIndex(0),
                OutputEntry::Value(true),
                OutputEntry::NewIndex(1),
                OutputEntry::Value(false),
                OutputEntry::NewIndex(2)
            ],
        ),
        assign_to_circuit_optimize_and_dedup(
            &Circuit::new(
                6,
                [
                    Gate::new_and(0, 4),   // false
                    Gate::new_and(1, 5),   // 1
                    Gate::new_nimpl(7, 6), // out0=5
                    Gate::new_xor(2, 3),
                    Gate::new_xor(1, 5),
                    Gate::new_nor(9, 10),
                    Gate::new_and(4, 11), // (out1=false,true)=true
                    Gate::new_nimpl(0, 5),
                    Gate::new_and(1, 4),   // false
                    Gate::new_xor(13, 14), // out2=nimpl(0,5)
                    Gate::new_and(1, 4),
                    Gate::new_and(2, 4),
                    Gate::new_and(0, 2),
                    Gate::new_xor(17, 18), // out3=and(0,2)
                ],
                [(8, false), (12, true), (15, false), (16, false), (19, true)],
            )
            .unwrap(),
            [(1, true), (4, false)],
            false
        )
    );
}
