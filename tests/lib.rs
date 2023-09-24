use gatesim::*;
use gateutil::*;

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

    assert_eq!(
        (
            ClauseCircuit::new(
                3,
                [Clause::new_xor([(0, false), (1, false), (2, false),])],
                [(3, false)]
            )
            .unwrap(),
            vec![None, Some(0), None, None, Some(1), Some(2)],
            vec![OutputEntry::NewIndex(0)]
        ),
        optimize_clause_circuit(
            ClauseCircuit::new(
                6,
                [
                    Clause::new_xor([(2, false), (0, false), (3, false), (4, false)]),
                    Clause::new_xor([(6, false), (1, false), (1, false)]),
                    Clause::new_xor([(0, false), (4, false), (6, false)]),
                    Clause::new_xor([(0, false), (8, false), (5, false), (7, false), (1, false)]),
                ],
                [(9, false)]
            )
            .unwrap()
        )
    );
}
