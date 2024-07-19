use gatesim::*;
use gateutil::*;

#[test]
fn test_min_and_max_depth_list() {
    assert_eq!(
        (
            vec![
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (1, 1),
                (1, 1),
                (1, 1),
                (1, 1),
                (2, 2),
                (2, 2),
                (3, 3),
                (3, 4)
            ],
            3,
            4
        ),
        min_and_max_depth_list(
            &Circuit::new(
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
            .unwrap()
        )
    );
    assert_eq!(
        (
            vec![
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (1, 1),
                (1, 1),
                (1, 2),
                (1, 1),
                (2, 2),
                (2, 3),
                (1, 2),
                (2, 3),
                (1, 4),
            ],
            1,
            4
        ),
        min_and_max_depth_list(
            &Circuit::new(
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
            .unwrap()
        )
    );
}
