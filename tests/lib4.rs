use gatesim::*;
use gateutil::*;

#[test]
fn test_min_and_max_depth() {
    assert_eq!(
        (3, 4),
        min_and_max_depth(
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
}
