use gatesim::*;
use gateutil::*;

use std::str::FromStr;

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

#[test]
fn test_simple_pipeliner() {
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
        simple_pipeliner(
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
            5
        )
    );
    assert_eq!(
        Circuit::new(
            6,
            [
                Gate::new_xor(3, 4),
                Gate::new_xor(2, 5),
                Gate::new_nor(3, 4),
                Gate::new_nor(2, 5),
                Gate::new_and(6, 7),
                Gate::new_and(8, 9),
                Gate::new_nimpl(0, 1),
                Gate::new_nimpl(1, 12),
            ],
            [(10, false), (11, false), (13, true)],
        )
        .unwrap(),
        simple_pipeliner(
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
            2
        )
    );
    let circuit = simple_pipeliner(
        Circuit::<usize>::from_str(r##"    {
        0
        1
        2
        3
        4
        5
        6
        7
        8
        9
        10
        11
        12
        13
        14
        15
        16
        17
        18
        19
        20
        21
        22
        23
        24
        25
        26
        27
        28
        29
        30
        31
        xor(0,16):0
        xor(1,17)
        and(0,16)
        xor(33,34):1
        xor(2,18)
        and(33,34)
        and(1,17)
        nor(37,38)
        xor(36,39):2n
        xor(3,19)
        nimpl(36,39)
        and(2,18)
        nor(42,43)
        xor(41,44):3n
        xor(4,20)
        nimpl(41,44)
        and(3,19)
        nor(47,48)
        xor(46,49):4n
        xor(5,21)
        nimpl(46,49)
        and(4,20)
        nor(52,53)
        xor(51,54):5n
        xor(6,22)
        nimpl(51,54)
        and(5,21)
        nor(57,58)
        xor(56,59):6n
        xor(7,23)
        nimpl(56,59)
        and(6,22)
        nor(62,63)
        xor(61,64):7n
        xor(8,24)
        nimpl(61,64)
        and(7,23)
        nor(67,68)
        xor(66,69):8n
        xor(9,25)
        nimpl(66,69)
        and(8,24)
        nor(72,73)
        xor(71,74):9n
        xor(10,26)
        nimpl(71,74)
        and(9,25)
        nor(77,78)
        xor(76,79):10n
        xor(11,27)
        nimpl(76,79)
        and(10,26)
        nor(82,83)
        xor(81,84):11n
        xor(12,28)
        nimpl(81,84)
        and(11,27)
        nor(87,88)
        xor(86,89):12n
        xor(13,29)
        nimpl(86,89)
        and(12,28)
        nor(92,93)
        xor(91,94):13n
        xor(14,30)
        nimpl(91,94)
        and(13,29)
        nor(97,98)
        xor(96,99):14n
        xor(15,31)
        nimpl(96,99)
        and(14,30)
        nor(102,103)
        xor(101,104):15n
    }(32)
"##
        ).unwrap(),
        4
    );
    println!("{}", FmtLiner::new(&circuit, 4, 8));
}
