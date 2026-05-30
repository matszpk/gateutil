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
        (
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
            1
        ),
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
        (
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
            2
        ),
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
    assert_eq!(
        (
            Circuit::from_str(
                r##"    {
        0:30
        1:31
        2:32
        3
        4
        5
        6
        7
        8:35
        9:37
        10:38
        11:39
        12:40
        13:41
        14:42
        15:43
        16:44
        17:45
        18:46
        19:47
        20:48
        21:49
        22:50
        23:51
        24:52
        25:53
        26:54
        27:55
        28:56
        29:57
        30:58
        31:59
        32:60
        33:61
        34:62
        35
        36
        37
        38
        39
        40:65
        41:67
        42:68
        43:69
        44:70
        45:71
        46:72
        47:73
        48:74
        49:75
        50:76
        51:77
        52:78
        53:79
        54:80
        55:81
        56:82
        57:83
        58:84
        59:85
        60:86
        61:87
        62:88
        63:89
        64:90
        65
        66
        67
        68
        69
        70:93
        71:95
        72:96
        73:97
        74:98
        75:99
        76:100
        77:101
        78:102
        79:103
        80:104
        81:105
        82:106
        83:107
        84:108
        85:109
        86:110
        87:111
        88:112
        89:113
        90:114
        91:115
        92:116
        93
        94
        95
        96
        97
        98:119
        99:121
        100:122
        101:123
        102:124
        103:125
        104:126
        105:127
        106:128
        107:129
        108:130
        109:131
        110:132
        111:133
        112:134
        113:135
        114:136
        115:137
        116:138
        117:139
        118:140
        119
        120
        121
        122
        123
        124:143
        125:145
        126:146
        127:147
        128:148
        129:149
        130:150
        131:151
        132:152
        133:153
        134:154
        135:155
        136:156
        137:157
        138:158
        139:159
        140:160
        141:161
        142:162
        143
        144
        145
        146
        147
        148:165
        149:167
        150:168
        151:169
        152:170n
        153:171n
        154:172n
        155:173n
        156:174n
        157:175n
        158:176n
        159:177n
        160:178n
        161:179n
        162:180n
        163:181n
        164:182n
        165
        166
        167
        168
        169
        170
        171
        172
        173
        174
        175
        176
        177
        178
        179
        180
        181
        182
        183
        184
        185
        186
        187
        188
        189
        190
        191
        192
        193
        194
        195
        196
        197
        198
        199
        xor(168,184):0
        xor(169,185)
        and(168,184)
        xor(170,186)
        and(169,185)
        xor(171,187):3
        and(170,186):5
        xor(172,188):6
        and(171,187):7
        xor(173,189):8
        and(172,188):9
        xor(174,190):10
        and(173,189):11
        xor(175,191):12
        and(174,190):13
        xor(176,192):14
        and(175,191):15
        xor(177,193):16
        and(176,192):17
        xor(178,194):18
        and(177,193):19
        xor(179,195):20
        and(178,194):21
        xor(180,196):22
        and(179,195):23
        xor(181,197):24
        and(180,196):25
        xor(182,198):26
        and(181,197):27
        xor(183,199):28
        and(182,198):29
        xor(201,202):1
        and(201,202)
        nor(232,204)
        xor(203,233):2
        nimpl(203,233):4
        nor(4,5)
        xor(3,236):33
        nimpl(3,236)
        nor(238,7)
        xor(6,239):34
        nimpl(6,239):36
        nor(36,37)
        xor(35,242):63
        nimpl(35,242)
        nor(244,39)
        xor(38,245):64
        nimpl(38,245):66
        nor(66,67)
        xor(65,248):91
        nimpl(65,248)
        nor(250,69)
        xor(68,251):92
        nimpl(68,251):94
        nor(94,95)
        xor(93,254):117
        nimpl(93,254)
        nor(256,97)
        xor(96,257):118
        nimpl(96,257):120
        nor(120,121)
        xor(119,260):141
        nimpl(119,260)
        nor(262,123)
        xor(122,263):142
        nimpl(122,263):144
        nor(144,145)
        xor(143,266):163
        nimpl(143,266)
        nor(268,147)
        xor(146,269):164
        nimpl(146,269):166
        nor(166,167)
        xor(165,272):183n
    }(200)
"##
            )
            .unwrap(),
            8
        ),
        simple_pipeliner(
            Circuit::<usize>::from_str(
                r##"    {
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
            )
            .unwrap(),
            4
        )
    );
    // next testcase
    assert_eq!(
        (
            Circuit::from_str(
                r##"{
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
        32
        33
        34
        35
        36
        37
        38
        39
        40
        41
        and(10,11)
        nor(12,13)
        xor(14,15)
        nimpl(16,17)
        nor(18,19)
        nimpl(21,20)
        xor(22,23)
        nor(24,25)
        nimpl(26,27)
        xor(28,29)
        and(30,31)
        nor(32,33)
        nor(34,35)
        nimpl(37,36)
        xor(38,39)
        and(40,41)
        nor(42,43):0
        nimpl(44,45):1
        xor(46,47):2
        and(48,49):3
        xor(50,51):4
        nor(52,53):5
        nimpl(55,54):6
        xor(56,57):7
        nor(0,1)
        nor(2,3)
        xor(4,5)
        and(6,7)
        nimpl(67,66):8
        nimpl(68,69):9
        and(8,9):10
    }(42)
"##
            )
            .unwrap(),
            3
        ),
        simple_pipeliner(
            Circuit::<usize>::from_str(
                r##"    {
                0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15
                16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31
                and(0,1)
                nor(2,3)
                xor(4,5)
                nimpl(6,7)
                nor(8,9)
                nimpl(11,10)
                xor(12,13)
                nor(14,15)
                nimpl(16,17)
                xor(18,19)
                and(20,21)
                nor(22,23)
                nor(24,25)
                nimpl(27,26)
                xor(28,29)
                and(30,31)
                nor(32,33)
                nimpl(34,35)
                xor(36,37)
                and(38,39)
                xor(40,41)
                nor(42,43)
                nimpl(45,44)
                xor(46,47)
                nor(48,49)
                nor(50,51)
                xor(52,53)
                and(54,55)
                nimpl(57,56)
                nimpl(58,59)
                and(60,61):0
}(32)
"##
            )
            .unwrap(),
            2,
        )
    );
    // fullmul 4x4
    assert_eq!(
        (
            Circuit::from_str(
                r##"    {
        0:16
        1:17
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
        16:28
        17:29
        18:30
        19:31
        20
        21
        22
        23:33
        24
        25:35
        26:36
        27:37
        28:38
        29:39
        30:40
        31:41
        32:42
        33
        34
        35
        36
        37:46
        38:47
        39:48
        40:49
        41:50n
        42:51
        43:52
        44:53
        45
        46
        47
        48
        49
        50
        51
        52
        53
        54
        and(47,51):0
        and(47,52)
        and(48,51)
        and(47,53)
        and(48,52)
        and(49,51)
        and(47,54)
        and(48,53)
        and(49,52)
        and(50,51)
        and(48,54)
        and(49,53)
        and(50,52)
        and(49,54)
        and(50,53)
        and(50,54):13
        xor(56,57):1
        xor(59,60)
        and(56,57):3
        xor(61,62)
        xor(63,64)
        and(59,60):5
        xor(66,67)
        and(63,64):8
        and(61,62):10
        xor(68,69)
        and(66,67)
        and(68,69):15
        xor(58,72):2
        xor(74,75):4
        and(58,72):6
        xor(65,77):7
        and(74,75):9
        xor(80,81):11
        and(65,77):12
        and(80,81):14
        xor(2,3):18
        xor(4,5)
        and(2,3)
        xor(7,8)
        nor(9,10)
        and(4,5):22
        and(7,8)
        nor(14,15)
        nor(93,6)
        xor(94,95):20
        nor(97,12)
        nimpl(94,95):24
        xor(13,98):25
        nimpl(13,98):27
        xor(92,99):19
        nimpl(92,99):21
        xor(11,101):23
        nimpl(11,101):26
        nor(21,22)
        xor(20,109):32
        nor(20,109)
        nor(111,24):34
        xor(33,34):43
        nor(33,34)
        nor(114,36)
        xor(35,115):44
        nor(35,115):45
        nor(45,46):54n
    }(55)
"##
            )
            .unwrap(),
            5
        ),
        simple_pipeliner(
            Circuit::<usize>::from_str(
                r##"
    {
        0
        1
        2
        3
        4
        5
        6
        7
        and(0,4):0
        and(0,5)
        and(1,4)
        xor(9,10):1
        and(0,6)
        and(1,5)
        and(2,4)
        xor(13,14)
        xor(12,15)
        and(9,10)
        xor(16,17):2
        and(0,7)
        and(1,6)
        xor(19,20)
        and(2,5)
        and(3,4)
        xor(22,23)
        xor(21,24)
        and(13,14)
        xor(25,26)
        and(16,17)
        and(12,15)
        nor(28,29)
        xor(27,30):3n
        and(1,7)
        and(2,6)
        and(3,5)
        xor(33,34)
        xor(32,35)
        and(22,23)
        xor(36,37)
        and(21,24)
        and(19,20)
        nor(39,40)
        xor(38,41)
        nimpl(27,30)
        and(25,26)
        nor(43,44)
        xor(42,45):4
        and(2,7)
        and(3,6)
        xor(47,48)
        and(33,34)
        xor(49,50)
        and(36,37)
        and(32,35)
        nor(52,53)
        xor(51,54)
        nor(42,45)
        nimpl(38,41)
        nor(56,57)
        xor(55,58):5
        and(3,7)
        and(49,50)
        and(47,48)
        nor(61,62)
        xor(60,63)
        nor(55,58)
        nimpl(51,54)
        nor(65,66)
        xor(64,67):6
        nor(64,67)
        nimpl(60,63)
        nor(69,70):7n
    }(8)
"##
            )
            .unwrap(),
            3
        )
    );
}

#[test]
fn test_circuit_table() {
    // assign 0 input to empty circuit
    assert_eq!(
        (Circuit::new(0, [], []).unwrap(), vec![], vec![vec![]],),
        circuit_table(&Circuit::new(0, [], []).unwrap(), [])
    );
    // simple circuits without index inputs.
    assert_eq!(
        (
            Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
            vec![0, 1],
            vec![vec![OutputEntry::NewIndex(0)]],
        ),
        circuit_table(
            &Circuit::new(2, [Gate::new_and(0, 1)], [(2, false)]).unwrap(),
            []
        )
    );
    assert_eq!(
        (
            Circuit::new(
                2,
                [
                    Gate::new_and(0, 1),
                    Gate::new_nor(0, 1),
                    Gate::new_xor(2, 3),
                ],
                [(4, false)]
            )
            .unwrap(),
            vec![0, 1],
            vec![vec![OutputEntry::NewIndex(0)]],
        ),
        circuit_table(
            &Circuit::new(
                2,
                [
                    Gate::new_and(0, 1),
                    Gate::new_nor(0, 1),
                    Gate::new_xor(2, 3),
                ],
                [(4, false)]
            )
            .unwrap(),
            []
        )
    );
    assert_eq!(
        (
            Circuit::new(
                5,
                [
                    Gate::new_and(3, 0),
                    Gate::new_nor(0, 2),
                    Gate::new_xor(5, 6),
                ],
                [(4, false), (7, true), (1, true), (0, false)]
            )
            .unwrap(),
            vec![0, 1, 2, 3, 4],
            vec![vec![
                OutputEntry::NewIndex(0),
                OutputEntry::NewIndex(1),
                OutputEntry::NewIndex(2),
                OutputEntry::NewIndex(3)
            ]],
        ),
        circuit_table(
            &Circuit::new(
                5,
                [
                    Gate::new_and(3, 0),
                    Gate::new_nor(0, 2),
                    Gate::new_xor(5, 6),
                ],
                [(4, false), (7, true), (1, true), (0, false)]
            )
            .unwrap(),
            []
        )
    );
    assert_eq!(
        (
            Circuit::new(
                6,
                [
                    Gate::new_and(0, 4),   // false
                    Gate::new_nor(1, 5),   // 1
                    Gate::new_nimpl(7, 6), // out0=5
                    Gate::new_xor(2, 3),
                    Gate::new_xor(1, 5),
                    Gate::new_nor(9, 10),
                    Gate::new_and(4, 11), // (out1=false,true)=true
                    Gate::new_nimpl(0, 5),
                    Gate::new_and(1, 4),   // false
                    Gate::new_xor(13, 14), // out2=nimpl(0,5)
                    Gate::new_nor(1, 4),
                    Gate::new_xor(2, 4),
                    Gate::new_and(0, 2),
                    Gate::new_xor(17, 18), // out3=and(0,2)
                ],
                [(8, false), (12, true), (15, false), (16, false), (19, true)],
            )
            .unwrap(),
            vec![0, 1, 2, 3, 4, 5],
            vec![vec![
                OutputEntry::NewIndex(0),
                OutputEntry::NewIndex(1),
                OutputEntry::NewIndex(2),
                OutputEntry::NewIndex(3),
                OutputEntry::NewIndex(4),
            ]],
        ),
        circuit_table(
            &Circuit::new(
                6,
                [
                    Gate::new_and(0, 4),   // false
                    Gate::new_nor(1, 5),   // 1
                    Gate::new_nimpl(7, 6), // out0=5
                    Gate::new_xor(2, 3),
                    Gate::new_xor(1, 5),
                    Gate::new_nor(9, 10),
                    Gate::new_and(4, 11), // (out1=false,true)=true
                    Gate::new_nimpl(0, 5),
                    Gate::new_and(1, 4),   // false
                    Gate::new_xor(13, 14), // out2=nimpl(0,5)
                    Gate::new_nor(1, 4),
                    Gate::new_xor(2, 4),
                    Gate::new_and(0, 2),
                    Gate::new_xor(17, 18), // out3=and(0,2)
                ],
                [(8, false), (12, true), (15, false), (16, false), (19, true)],
            )
            .unwrap(),
            []
        )
    );
}
