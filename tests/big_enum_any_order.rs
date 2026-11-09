//! A 500-variant enum written the way people write them, not in the
//! canonical form: metadata attributes in any order, a doc comment after
//! them, attributes passed through, and unnamed tuple fields with and without
//! a field marker. It compiles at the default `recursion_limit` (NFR-6, E-3,
//! D-087). Written by a short script, like `tests/big_enum.rs`; the test
//! checks the first and last variant of each form.

use define_err::{define_errors, Category, ErrorInfo, Severity};
use std::error::Error as _;

define_errors! {
    namespace = "ANY";

    /// Five hundred variants in four irregular forms.
    pub enum AnyOrder {
        #[category(NotFound)]
        #[code(1, help = "Check item 0.")]
        /// Variant 0, its doc comment after its metadata.
        Late0 { id: u64 } => "late 0 {id}",
        Wrapped1(#[source] std::fmt::Error) => "wrapped 1",
        #[allow(dead_code)]
        Pair2(u8, String) => "pair 2",
        #[severity(WARNING)]
        #[code(4, help = "Retry item 3.")]
        #[retryable]
        Unit3 => "unit 3",
        #[category(Unavailable)]
        #[code(5, help = "Check item 4.")]
        /// Variant 4, its doc comment after its metadata.
        Late4 { id: u64 } => "late 4 {id}",
        Wrapped5(#[source] std::fmt::Error) => "wrapped 5",
        #[allow(dead_code)]
        Pair6(u8, String) => "pair 6",
        #[severity(WARNING)]
        #[code(8, help = "Retry item 7.")]
        #[retryable]
        Unit7 => "unit 7",
        #[category(InvalidArgument)]
        #[code(9, help = "Check item 8.")]
        /// Variant 8, its doc comment after its metadata.
        Late8 { id: u64 } => "late 8 {id}",
        Wrapped9(#[source] std::fmt::Error) => "wrapped 9",
        #[allow(dead_code)]
        Pair10(u8, String) => "pair 10",
        #[severity(WARNING)]
        #[code(12, help = "Retry item 11.")]
        #[retryable]
        Unit11 => "unit 11",
        #[category(Internal)]
        #[code(13, help = "Check item 12.")]
        /// Variant 12, its doc comment after its metadata.
        Late12 { id: u64 } => "late 12 {id}",
        Wrapped13(#[source] std::fmt::Error) => "wrapped 13",
        #[allow(dead_code)]
        Pair14(u8, String) => "pair 14",
        #[severity(WARNING)]
        #[code(16, help = "Retry item 15.")]
        #[retryable]
        Unit15 => "unit 15",
        #[category(NotFound)]
        #[code(17, help = "Check item 16.")]
        /// Variant 16, its doc comment after its metadata.
        Late16 { id: u64 } => "late 16 {id}",
        Wrapped17(#[source] std::fmt::Error) => "wrapped 17",
        #[allow(dead_code)]
        Pair18(u8, String) => "pair 18",
        #[severity(WARNING)]
        #[code(20, help = "Retry item 19.")]
        #[retryable]
        Unit19 => "unit 19",
        #[category(Unavailable)]
        #[code(21, help = "Check item 20.")]
        /// Variant 20, its doc comment after its metadata.
        Late20 { id: u64 } => "late 20 {id}",
        Wrapped21(#[source] std::fmt::Error) => "wrapped 21",
        #[allow(dead_code)]
        Pair22(u8, String) => "pair 22",
        #[severity(WARNING)]
        #[code(24, help = "Retry item 23.")]
        #[retryable]
        Unit23 => "unit 23",
        #[category(InvalidArgument)]
        #[code(25, help = "Check item 24.")]
        /// Variant 24, its doc comment after its metadata.
        Late24 { id: u64 } => "late 24 {id}",
        Wrapped25(#[source] std::fmt::Error) => "wrapped 25",
        #[allow(dead_code)]
        Pair26(u8, String) => "pair 26",
        #[severity(WARNING)]
        #[code(28, help = "Retry item 27.")]
        #[retryable]
        Unit27 => "unit 27",
        #[category(Internal)]
        #[code(29, help = "Check item 28.")]
        /// Variant 28, its doc comment after its metadata.
        Late28 { id: u64 } => "late 28 {id}",
        Wrapped29(#[source] std::fmt::Error) => "wrapped 29",
        #[allow(dead_code)]
        Pair30(u8, String) => "pair 30",
        #[severity(WARNING)]
        #[code(32, help = "Retry item 31.")]
        #[retryable]
        Unit31 => "unit 31",
        #[category(NotFound)]
        #[code(33, help = "Check item 32.")]
        /// Variant 32, its doc comment after its metadata.
        Late32 { id: u64 } => "late 32 {id}",
        Wrapped33(#[source] std::fmt::Error) => "wrapped 33",
        #[allow(dead_code)]
        Pair34(u8, String) => "pair 34",
        #[severity(WARNING)]
        #[code(36, help = "Retry item 35.")]
        #[retryable]
        Unit35 => "unit 35",
        #[category(Unavailable)]
        #[code(37, help = "Check item 36.")]
        /// Variant 36, its doc comment after its metadata.
        Late36 { id: u64 } => "late 36 {id}",
        Wrapped37(#[source] std::fmt::Error) => "wrapped 37",
        #[allow(dead_code)]
        Pair38(u8, String) => "pair 38",
        #[severity(WARNING)]
        #[code(40, help = "Retry item 39.")]
        #[retryable]
        Unit39 => "unit 39",
        #[category(InvalidArgument)]
        #[code(41, help = "Check item 40.")]
        /// Variant 40, its doc comment after its metadata.
        Late40 { id: u64 } => "late 40 {id}",
        Wrapped41(#[source] std::fmt::Error) => "wrapped 41",
        #[allow(dead_code)]
        Pair42(u8, String) => "pair 42",
        #[severity(WARNING)]
        #[code(44, help = "Retry item 43.")]
        #[retryable]
        Unit43 => "unit 43",
        #[category(Internal)]
        #[code(45, help = "Check item 44.")]
        /// Variant 44, its doc comment after its metadata.
        Late44 { id: u64 } => "late 44 {id}",
        Wrapped45(#[source] std::fmt::Error) => "wrapped 45",
        #[allow(dead_code)]
        Pair46(u8, String) => "pair 46",
        #[severity(WARNING)]
        #[code(48, help = "Retry item 47.")]
        #[retryable]
        Unit47 => "unit 47",
        #[category(NotFound)]
        #[code(49, help = "Check item 48.")]
        /// Variant 48, its doc comment after its metadata.
        Late48 { id: u64 } => "late 48 {id}",
        Wrapped49(#[source] std::fmt::Error) => "wrapped 49",
        #[allow(dead_code)]
        Pair50(u8, String) => "pair 50",
        #[severity(WARNING)]
        #[code(52, help = "Retry item 51.")]
        #[retryable]
        Unit51 => "unit 51",
        #[category(Unavailable)]
        #[code(53, help = "Check item 52.")]
        /// Variant 52, its doc comment after its metadata.
        Late52 { id: u64 } => "late 52 {id}",
        Wrapped53(#[source] std::fmt::Error) => "wrapped 53",
        #[allow(dead_code)]
        Pair54(u8, String) => "pair 54",
        #[severity(WARNING)]
        #[code(56, help = "Retry item 55.")]
        #[retryable]
        Unit55 => "unit 55",
        #[category(InvalidArgument)]
        #[code(57, help = "Check item 56.")]
        /// Variant 56, its doc comment after its metadata.
        Late56 { id: u64 } => "late 56 {id}",
        Wrapped57(#[source] std::fmt::Error) => "wrapped 57",
        #[allow(dead_code)]
        Pair58(u8, String) => "pair 58",
        #[severity(WARNING)]
        #[code(60, help = "Retry item 59.")]
        #[retryable]
        Unit59 => "unit 59",
        #[category(Internal)]
        #[code(61, help = "Check item 60.")]
        /// Variant 60, its doc comment after its metadata.
        Late60 { id: u64 } => "late 60 {id}",
        Wrapped61(#[source] std::fmt::Error) => "wrapped 61",
        #[allow(dead_code)]
        Pair62(u8, String) => "pair 62",
        #[severity(WARNING)]
        #[code(64, help = "Retry item 63.")]
        #[retryable]
        Unit63 => "unit 63",
        #[category(NotFound)]
        #[code(65, help = "Check item 64.")]
        /// Variant 64, its doc comment after its metadata.
        Late64 { id: u64 } => "late 64 {id}",
        Wrapped65(#[source] std::fmt::Error) => "wrapped 65",
        #[allow(dead_code)]
        Pair66(u8, String) => "pair 66",
        #[severity(WARNING)]
        #[code(68, help = "Retry item 67.")]
        #[retryable]
        Unit67 => "unit 67",
        #[category(Unavailable)]
        #[code(69, help = "Check item 68.")]
        /// Variant 68, its doc comment after its metadata.
        Late68 { id: u64 } => "late 68 {id}",
        Wrapped69(#[source] std::fmt::Error) => "wrapped 69",
        #[allow(dead_code)]
        Pair70(u8, String) => "pair 70",
        #[severity(WARNING)]
        #[code(72, help = "Retry item 71.")]
        #[retryable]
        Unit71 => "unit 71",
        #[category(InvalidArgument)]
        #[code(73, help = "Check item 72.")]
        /// Variant 72, its doc comment after its metadata.
        Late72 { id: u64 } => "late 72 {id}",
        Wrapped73(#[source] std::fmt::Error) => "wrapped 73",
        #[allow(dead_code)]
        Pair74(u8, String) => "pair 74",
        #[severity(WARNING)]
        #[code(76, help = "Retry item 75.")]
        #[retryable]
        Unit75 => "unit 75",
        #[category(Internal)]
        #[code(77, help = "Check item 76.")]
        /// Variant 76, its doc comment after its metadata.
        Late76 { id: u64 } => "late 76 {id}",
        Wrapped77(#[source] std::fmt::Error) => "wrapped 77",
        #[allow(dead_code)]
        Pair78(u8, String) => "pair 78",
        #[severity(WARNING)]
        #[code(80, help = "Retry item 79.")]
        #[retryable]
        Unit79 => "unit 79",
        #[category(NotFound)]
        #[code(81, help = "Check item 80.")]
        /// Variant 80, its doc comment after its metadata.
        Late80 { id: u64 } => "late 80 {id}",
        Wrapped81(#[source] std::fmt::Error) => "wrapped 81",
        #[allow(dead_code)]
        Pair82(u8, String) => "pair 82",
        #[severity(WARNING)]
        #[code(84, help = "Retry item 83.")]
        #[retryable]
        Unit83 => "unit 83",
        #[category(Unavailable)]
        #[code(85, help = "Check item 84.")]
        /// Variant 84, its doc comment after its metadata.
        Late84 { id: u64 } => "late 84 {id}",
        Wrapped85(#[source] std::fmt::Error) => "wrapped 85",
        #[allow(dead_code)]
        Pair86(u8, String) => "pair 86",
        #[severity(WARNING)]
        #[code(88, help = "Retry item 87.")]
        #[retryable]
        Unit87 => "unit 87",
        #[category(InvalidArgument)]
        #[code(89, help = "Check item 88.")]
        /// Variant 88, its doc comment after its metadata.
        Late88 { id: u64 } => "late 88 {id}",
        Wrapped89(#[source] std::fmt::Error) => "wrapped 89",
        #[allow(dead_code)]
        Pair90(u8, String) => "pair 90",
        #[severity(WARNING)]
        #[code(92, help = "Retry item 91.")]
        #[retryable]
        Unit91 => "unit 91",
        #[category(Internal)]
        #[code(93, help = "Check item 92.")]
        /// Variant 92, its doc comment after its metadata.
        Late92 { id: u64 } => "late 92 {id}",
        Wrapped93(#[source] std::fmt::Error) => "wrapped 93",
        #[allow(dead_code)]
        Pair94(u8, String) => "pair 94",
        #[severity(WARNING)]
        #[code(96, help = "Retry item 95.")]
        #[retryable]
        Unit95 => "unit 95",
        #[category(NotFound)]
        #[code(97, help = "Check item 96.")]
        /// Variant 96, its doc comment after its metadata.
        Late96 { id: u64 } => "late 96 {id}",
        Wrapped97(#[source] std::fmt::Error) => "wrapped 97",
        #[allow(dead_code)]
        Pair98(u8, String) => "pair 98",
        #[severity(WARNING)]
        #[code(100, help = "Retry item 99.")]
        #[retryable]
        Unit99 => "unit 99",
        #[category(Unavailable)]
        #[code(101, help = "Check item 100.")]
        /// Variant 100, its doc comment after its metadata.
        Late100 { id: u64 } => "late 100 {id}",
        Wrapped101(#[source] std::fmt::Error) => "wrapped 101",
        #[allow(dead_code)]
        Pair102(u8, String) => "pair 102",
        #[severity(WARNING)]
        #[code(104, help = "Retry item 103.")]
        #[retryable]
        Unit103 => "unit 103",
        #[category(InvalidArgument)]
        #[code(105, help = "Check item 104.")]
        /// Variant 104, its doc comment after its metadata.
        Late104 { id: u64 } => "late 104 {id}",
        Wrapped105(#[source] std::fmt::Error) => "wrapped 105",
        #[allow(dead_code)]
        Pair106(u8, String) => "pair 106",
        #[severity(WARNING)]
        #[code(108, help = "Retry item 107.")]
        #[retryable]
        Unit107 => "unit 107",
        #[category(Internal)]
        #[code(109, help = "Check item 108.")]
        /// Variant 108, its doc comment after its metadata.
        Late108 { id: u64 } => "late 108 {id}",
        Wrapped109(#[source] std::fmt::Error) => "wrapped 109",
        #[allow(dead_code)]
        Pair110(u8, String) => "pair 110",
        #[severity(WARNING)]
        #[code(112, help = "Retry item 111.")]
        #[retryable]
        Unit111 => "unit 111",
        #[category(NotFound)]
        #[code(113, help = "Check item 112.")]
        /// Variant 112, its doc comment after its metadata.
        Late112 { id: u64 } => "late 112 {id}",
        Wrapped113(#[source] std::fmt::Error) => "wrapped 113",
        #[allow(dead_code)]
        Pair114(u8, String) => "pair 114",
        #[severity(WARNING)]
        #[code(116, help = "Retry item 115.")]
        #[retryable]
        Unit115 => "unit 115",
        #[category(Unavailable)]
        #[code(117, help = "Check item 116.")]
        /// Variant 116, its doc comment after its metadata.
        Late116 { id: u64 } => "late 116 {id}",
        Wrapped117(#[source] std::fmt::Error) => "wrapped 117",
        #[allow(dead_code)]
        Pair118(u8, String) => "pair 118",
        #[severity(WARNING)]
        #[code(120, help = "Retry item 119.")]
        #[retryable]
        Unit119 => "unit 119",
        #[category(InvalidArgument)]
        #[code(121, help = "Check item 120.")]
        /// Variant 120, its doc comment after its metadata.
        Late120 { id: u64 } => "late 120 {id}",
        Wrapped121(#[source] std::fmt::Error) => "wrapped 121",
        #[allow(dead_code)]
        Pair122(u8, String) => "pair 122",
        #[severity(WARNING)]
        #[code(124, help = "Retry item 123.")]
        #[retryable]
        Unit123 => "unit 123",
        #[category(Internal)]
        #[code(125, help = "Check item 124.")]
        /// Variant 124, its doc comment after its metadata.
        Late124 { id: u64 } => "late 124 {id}",
        Wrapped125(#[source] std::fmt::Error) => "wrapped 125",
        #[allow(dead_code)]
        Pair126(u8, String) => "pair 126",
        #[severity(WARNING)]
        #[code(128, help = "Retry item 127.")]
        #[retryable]
        Unit127 => "unit 127",
        #[category(NotFound)]
        #[code(129, help = "Check item 128.")]
        /// Variant 128, its doc comment after its metadata.
        Late128 { id: u64 } => "late 128 {id}",
        Wrapped129(#[source] std::fmt::Error) => "wrapped 129",
        #[allow(dead_code)]
        Pair130(u8, String) => "pair 130",
        #[severity(WARNING)]
        #[code(132, help = "Retry item 131.")]
        #[retryable]
        Unit131 => "unit 131",
        #[category(Unavailable)]
        #[code(133, help = "Check item 132.")]
        /// Variant 132, its doc comment after its metadata.
        Late132 { id: u64 } => "late 132 {id}",
        Wrapped133(#[source] std::fmt::Error) => "wrapped 133",
        #[allow(dead_code)]
        Pair134(u8, String) => "pair 134",
        #[severity(WARNING)]
        #[code(136, help = "Retry item 135.")]
        #[retryable]
        Unit135 => "unit 135",
        #[category(InvalidArgument)]
        #[code(137, help = "Check item 136.")]
        /// Variant 136, its doc comment after its metadata.
        Late136 { id: u64 } => "late 136 {id}",
        Wrapped137(#[source] std::fmt::Error) => "wrapped 137",
        #[allow(dead_code)]
        Pair138(u8, String) => "pair 138",
        #[severity(WARNING)]
        #[code(140, help = "Retry item 139.")]
        #[retryable]
        Unit139 => "unit 139",
        #[category(Internal)]
        #[code(141, help = "Check item 140.")]
        /// Variant 140, its doc comment after its metadata.
        Late140 { id: u64 } => "late 140 {id}",
        Wrapped141(#[source] std::fmt::Error) => "wrapped 141",
        #[allow(dead_code)]
        Pair142(u8, String) => "pair 142",
        #[severity(WARNING)]
        #[code(144, help = "Retry item 143.")]
        #[retryable]
        Unit143 => "unit 143",
        #[category(NotFound)]
        #[code(145, help = "Check item 144.")]
        /// Variant 144, its doc comment after its metadata.
        Late144 { id: u64 } => "late 144 {id}",
        Wrapped145(#[source] std::fmt::Error) => "wrapped 145",
        #[allow(dead_code)]
        Pair146(u8, String) => "pair 146",
        #[severity(WARNING)]
        #[code(148, help = "Retry item 147.")]
        #[retryable]
        Unit147 => "unit 147",
        #[category(Unavailable)]
        #[code(149, help = "Check item 148.")]
        /// Variant 148, its doc comment after its metadata.
        Late148 { id: u64 } => "late 148 {id}",
        Wrapped149(#[source] std::fmt::Error) => "wrapped 149",
        #[allow(dead_code)]
        Pair150(u8, String) => "pair 150",
        #[severity(WARNING)]
        #[code(152, help = "Retry item 151.")]
        #[retryable]
        Unit151 => "unit 151",
        #[category(InvalidArgument)]
        #[code(153, help = "Check item 152.")]
        /// Variant 152, its doc comment after its metadata.
        Late152 { id: u64 } => "late 152 {id}",
        Wrapped153(#[source] std::fmt::Error) => "wrapped 153",
        #[allow(dead_code)]
        Pair154(u8, String) => "pair 154",
        #[severity(WARNING)]
        #[code(156, help = "Retry item 155.")]
        #[retryable]
        Unit155 => "unit 155",
        #[category(Internal)]
        #[code(157, help = "Check item 156.")]
        /// Variant 156, its doc comment after its metadata.
        Late156 { id: u64 } => "late 156 {id}",
        Wrapped157(#[source] std::fmt::Error) => "wrapped 157",
        #[allow(dead_code)]
        Pair158(u8, String) => "pair 158",
        #[severity(WARNING)]
        #[code(160, help = "Retry item 159.")]
        #[retryable]
        Unit159 => "unit 159",
        #[category(NotFound)]
        #[code(161, help = "Check item 160.")]
        /// Variant 160, its doc comment after its metadata.
        Late160 { id: u64 } => "late 160 {id}",
        Wrapped161(#[source] std::fmt::Error) => "wrapped 161",
        #[allow(dead_code)]
        Pair162(u8, String) => "pair 162",
        #[severity(WARNING)]
        #[code(164, help = "Retry item 163.")]
        #[retryable]
        Unit163 => "unit 163",
        #[category(Unavailable)]
        #[code(165, help = "Check item 164.")]
        /// Variant 164, its doc comment after its metadata.
        Late164 { id: u64 } => "late 164 {id}",
        Wrapped165(#[source] std::fmt::Error) => "wrapped 165",
        #[allow(dead_code)]
        Pair166(u8, String) => "pair 166",
        #[severity(WARNING)]
        #[code(168, help = "Retry item 167.")]
        #[retryable]
        Unit167 => "unit 167",
        #[category(InvalidArgument)]
        #[code(169, help = "Check item 168.")]
        /// Variant 168, its doc comment after its metadata.
        Late168 { id: u64 } => "late 168 {id}",
        Wrapped169(#[source] std::fmt::Error) => "wrapped 169",
        #[allow(dead_code)]
        Pair170(u8, String) => "pair 170",
        #[severity(WARNING)]
        #[code(172, help = "Retry item 171.")]
        #[retryable]
        Unit171 => "unit 171",
        #[category(Internal)]
        #[code(173, help = "Check item 172.")]
        /// Variant 172, its doc comment after its metadata.
        Late172 { id: u64 } => "late 172 {id}",
        Wrapped173(#[source] std::fmt::Error) => "wrapped 173",
        #[allow(dead_code)]
        Pair174(u8, String) => "pair 174",
        #[severity(WARNING)]
        #[code(176, help = "Retry item 175.")]
        #[retryable]
        Unit175 => "unit 175",
        #[category(NotFound)]
        #[code(177, help = "Check item 176.")]
        /// Variant 176, its doc comment after its metadata.
        Late176 { id: u64 } => "late 176 {id}",
        Wrapped177(#[source] std::fmt::Error) => "wrapped 177",
        #[allow(dead_code)]
        Pair178(u8, String) => "pair 178",
        #[severity(WARNING)]
        #[code(180, help = "Retry item 179.")]
        #[retryable]
        Unit179 => "unit 179",
        #[category(Unavailable)]
        #[code(181, help = "Check item 180.")]
        /// Variant 180, its doc comment after its metadata.
        Late180 { id: u64 } => "late 180 {id}",
        Wrapped181(#[source] std::fmt::Error) => "wrapped 181",
        #[allow(dead_code)]
        Pair182(u8, String) => "pair 182",
        #[severity(WARNING)]
        #[code(184, help = "Retry item 183.")]
        #[retryable]
        Unit183 => "unit 183",
        #[category(InvalidArgument)]
        #[code(185, help = "Check item 184.")]
        /// Variant 184, its doc comment after its metadata.
        Late184 { id: u64 } => "late 184 {id}",
        Wrapped185(#[source] std::fmt::Error) => "wrapped 185",
        #[allow(dead_code)]
        Pair186(u8, String) => "pair 186",
        #[severity(WARNING)]
        #[code(188, help = "Retry item 187.")]
        #[retryable]
        Unit187 => "unit 187",
        #[category(Internal)]
        #[code(189, help = "Check item 188.")]
        /// Variant 188, its doc comment after its metadata.
        Late188 { id: u64 } => "late 188 {id}",
        Wrapped189(#[source] std::fmt::Error) => "wrapped 189",
        #[allow(dead_code)]
        Pair190(u8, String) => "pair 190",
        #[severity(WARNING)]
        #[code(192, help = "Retry item 191.")]
        #[retryable]
        Unit191 => "unit 191",
        #[category(NotFound)]
        #[code(193, help = "Check item 192.")]
        /// Variant 192, its doc comment after its metadata.
        Late192 { id: u64 } => "late 192 {id}",
        Wrapped193(#[source] std::fmt::Error) => "wrapped 193",
        #[allow(dead_code)]
        Pair194(u8, String) => "pair 194",
        #[severity(WARNING)]
        #[code(196, help = "Retry item 195.")]
        #[retryable]
        Unit195 => "unit 195",
        #[category(Unavailable)]
        #[code(197, help = "Check item 196.")]
        /// Variant 196, its doc comment after its metadata.
        Late196 { id: u64 } => "late 196 {id}",
        Wrapped197(#[source] std::fmt::Error) => "wrapped 197",
        #[allow(dead_code)]
        Pair198(u8, String) => "pair 198",
        #[severity(WARNING)]
        #[code(200, help = "Retry item 199.")]
        #[retryable]
        Unit199 => "unit 199",
        #[category(InvalidArgument)]
        #[code(201, help = "Check item 200.")]
        /// Variant 200, its doc comment after its metadata.
        Late200 { id: u64 } => "late 200 {id}",
        Wrapped201(#[source] std::fmt::Error) => "wrapped 201",
        #[allow(dead_code)]
        Pair202(u8, String) => "pair 202",
        #[severity(WARNING)]
        #[code(204, help = "Retry item 203.")]
        #[retryable]
        Unit203 => "unit 203",
        #[category(Internal)]
        #[code(205, help = "Check item 204.")]
        /// Variant 204, its doc comment after its metadata.
        Late204 { id: u64 } => "late 204 {id}",
        Wrapped205(#[source] std::fmt::Error) => "wrapped 205",
        #[allow(dead_code)]
        Pair206(u8, String) => "pair 206",
        #[severity(WARNING)]
        #[code(208, help = "Retry item 207.")]
        #[retryable]
        Unit207 => "unit 207",
        #[category(NotFound)]
        #[code(209, help = "Check item 208.")]
        /// Variant 208, its doc comment after its metadata.
        Late208 { id: u64 } => "late 208 {id}",
        Wrapped209(#[source] std::fmt::Error) => "wrapped 209",
        #[allow(dead_code)]
        Pair210(u8, String) => "pair 210",
        #[severity(WARNING)]
        #[code(212, help = "Retry item 211.")]
        #[retryable]
        Unit211 => "unit 211",
        #[category(Unavailable)]
        #[code(213, help = "Check item 212.")]
        /// Variant 212, its doc comment after its metadata.
        Late212 { id: u64 } => "late 212 {id}",
        Wrapped213(#[source] std::fmt::Error) => "wrapped 213",
        #[allow(dead_code)]
        Pair214(u8, String) => "pair 214",
        #[severity(WARNING)]
        #[code(216, help = "Retry item 215.")]
        #[retryable]
        Unit215 => "unit 215",
        #[category(InvalidArgument)]
        #[code(217, help = "Check item 216.")]
        /// Variant 216, its doc comment after its metadata.
        Late216 { id: u64 } => "late 216 {id}",
        Wrapped217(#[source] std::fmt::Error) => "wrapped 217",
        #[allow(dead_code)]
        Pair218(u8, String) => "pair 218",
        #[severity(WARNING)]
        #[code(220, help = "Retry item 219.")]
        #[retryable]
        Unit219 => "unit 219",
        #[category(Internal)]
        #[code(221, help = "Check item 220.")]
        /// Variant 220, its doc comment after its metadata.
        Late220 { id: u64 } => "late 220 {id}",
        Wrapped221(#[source] std::fmt::Error) => "wrapped 221",
        #[allow(dead_code)]
        Pair222(u8, String) => "pair 222",
        #[severity(WARNING)]
        #[code(224, help = "Retry item 223.")]
        #[retryable]
        Unit223 => "unit 223",
        #[category(NotFound)]
        #[code(225, help = "Check item 224.")]
        /// Variant 224, its doc comment after its metadata.
        Late224 { id: u64 } => "late 224 {id}",
        Wrapped225(#[source] std::fmt::Error) => "wrapped 225",
        #[allow(dead_code)]
        Pair226(u8, String) => "pair 226",
        #[severity(WARNING)]
        #[code(228, help = "Retry item 227.")]
        #[retryable]
        Unit227 => "unit 227",
        #[category(Unavailable)]
        #[code(229, help = "Check item 228.")]
        /// Variant 228, its doc comment after its metadata.
        Late228 { id: u64 } => "late 228 {id}",
        Wrapped229(#[source] std::fmt::Error) => "wrapped 229",
        #[allow(dead_code)]
        Pair230(u8, String) => "pair 230",
        #[severity(WARNING)]
        #[code(232, help = "Retry item 231.")]
        #[retryable]
        Unit231 => "unit 231",
        #[category(InvalidArgument)]
        #[code(233, help = "Check item 232.")]
        /// Variant 232, its doc comment after its metadata.
        Late232 { id: u64 } => "late 232 {id}",
        Wrapped233(#[source] std::fmt::Error) => "wrapped 233",
        #[allow(dead_code)]
        Pair234(u8, String) => "pair 234",
        #[severity(WARNING)]
        #[code(236, help = "Retry item 235.")]
        #[retryable]
        Unit235 => "unit 235",
        #[category(Internal)]
        #[code(237, help = "Check item 236.")]
        /// Variant 236, its doc comment after its metadata.
        Late236 { id: u64 } => "late 236 {id}",
        Wrapped237(#[source] std::fmt::Error) => "wrapped 237",
        #[allow(dead_code)]
        Pair238(u8, String) => "pair 238",
        #[severity(WARNING)]
        #[code(240, help = "Retry item 239.")]
        #[retryable]
        Unit239 => "unit 239",
        #[category(NotFound)]
        #[code(241, help = "Check item 240.")]
        /// Variant 240, its doc comment after its metadata.
        Late240 { id: u64 } => "late 240 {id}",
        Wrapped241(#[source] std::fmt::Error) => "wrapped 241",
        #[allow(dead_code)]
        Pair242(u8, String) => "pair 242",
        #[severity(WARNING)]
        #[code(244, help = "Retry item 243.")]
        #[retryable]
        Unit243 => "unit 243",
        #[category(Unavailable)]
        #[code(245, help = "Check item 244.")]
        /// Variant 244, its doc comment after its metadata.
        Late244 { id: u64 } => "late 244 {id}",
        Wrapped245(#[source] std::fmt::Error) => "wrapped 245",
        #[allow(dead_code)]
        Pair246(u8, String) => "pair 246",
        #[severity(WARNING)]
        #[code(248, help = "Retry item 247.")]
        #[retryable]
        Unit247 => "unit 247",
        #[category(InvalidArgument)]
        #[code(249, help = "Check item 248.")]
        /// Variant 248, its doc comment after its metadata.
        Late248 { id: u64 } => "late 248 {id}",
        Wrapped249(#[source] std::fmt::Error) => "wrapped 249",
        #[allow(dead_code)]
        Pair250(u8, String) => "pair 250",
        #[severity(WARNING)]
        #[code(252, help = "Retry item 251.")]
        #[retryable]
        Unit251 => "unit 251",
        #[category(Internal)]
        #[code(253, help = "Check item 252.")]
        /// Variant 252, its doc comment after its metadata.
        Late252 { id: u64 } => "late 252 {id}",
        Wrapped253(#[source] std::fmt::Error) => "wrapped 253",
        #[allow(dead_code)]
        Pair254(u8, String) => "pair 254",
        #[severity(WARNING)]
        #[code(256, help = "Retry item 255.")]
        #[retryable]
        Unit255 => "unit 255",
        #[category(NotFound)]
        #[code(257, help = "Check item 256.")]
        /// Variant 256, its doc comment after its metadata.
        Late256 { id: u64 } => "late 256 {id}",
        Wrapped257(#[source] std::fmt::Error) => "wrapped 257",
        #[allow(dead_code)]
        Pair258(u8, String) => "pair 258",
        #[severity(WARNING)]
        #[code(260, help = "Retry item 259.")]
        #[retryable]
        Unit259 => "unit 259",
        #[category(Unavailable)]
        #[code(261, help = "Check item 260.")]
        /// Variant 260, its doc comment after its metadata.
        Late260 { id: u64 } => "late 260 {id}",
        Wrapped261(#[source] std::fmt::Error) => "wrapped 261",
        #[allow(dead_code)]
        Pair262(u8, String) => "pair 262",
        #[severity(WARNING)]
        #[code(264, help = "Retry item 263.")]
        #[retryable]
        Unit263 => "unit 263",
        #[category(InvalidArgument)]
        #[code(265, help = "Check item 264.")]
        /// Variant 264, its doc comment after its metadata.
        Late264 { id: u64 } => "late 264 {id}",
        Wrapped265(#[source] std::fmt::Error) => "wrapped 265",
        #[allow(dead_code)]
        Pair266(u8, String) => "pair 266",
        #[severity(WARNING)]
        #[code(268, help = "Retry item 267.")]
        #[retryable]
        Unit267 => "unit 267",
        #[category(Internal)]
        #[code(269, help = "Check item 268.")]
        /// Variant 268, its doc comment after its metadata.
        Late268 { id: u64 } => "late 268 {id}",
        Wrapped269(#[source] std::fmt::Error) => "wrapped 269",
        #[allow(dead_code)]
        Pair270(u8, String) => "pair 270",
        #[severity(WARNING)]
        #[code(272, help = "Retry item 271.")]
        #[retryable]
        Unit271 => "unit 271",
        #[category(NotFound)]
        #[code(273, help = "Check item 272.")]
        /// Variant 272, its doc comment after its metadata.
        Late272 { id: u64 } => "late 272 {id}",
        Wrapped273(#[source] std::fmt::Error) => "wrapped 273",
        #[allow(dead_code)]
        Pair274(u8, String) => "pair 274",
        #[severity(WARNING)]
        #[code(276, help = "Retry item 275.")]
        #[retryable]
        Unit275 => "unit 275",
        #[category(Unavailable)]
        #[code(277, help = "Check item 276.")]
        /// Variant 276, its doc comment after its metadata.
        Late276 { id: u64 } => "late 276 {id}",
        Wrapped277(#[source] std::fmt::Error) => "wrapped 277",
        #[allow(dead_code)]
        Pair278(u8, String) => "pair 278",
        #[severity(WARNING)]
        #[code(280, help = "Retry item 279.")]
        #[retryable]
        Unit279 => "unit 279",
        #[category(InvalidArgument)]
        #[code(281, help = "Check item 280.")]
        /// Variant 280, its doc comment after its metadata.
        Late280 { id: u64 } => "late 280 {id}",
        Wrapped281(#[source] std::fmt::Error) => "wrapped 281",
        #[allow(dead_code)]
        Pair282(u8, String) => "pair 282",
        #[severity(WARNING)]
        #[code(284, help = "Retry item 283.")]
        #[retryable]
        Unit283 => "unit 283",
        #[category(Internal)]
        #[code(285, help = "Check item 284.")]
        /// Variant 284, its doc comment after its metadata.
        Late284 { id: u64 } => "late 284 {id}",
        Wrapped285(#[source] std::fmt::Error) => "wrapped 285",
        #[allow(dead_code)]
        Pair286(u8, String) => "pair 286",
        #[severity(WARNING)]
        #[code(288, help = "Retry item 287.")]
        #[retryable]
        Unit287 => "unit 287",
        #[category(NotFound)]
        #[code(289, help = "Check item 288.")]
        /// Variant 288, its doc comment after its metadata.
        Late288 { id: u64 } => "late 288 {id}",
        Wrapped289(#[source] std::fmt::Error) => "wrapped 289",
        #[allow(dead_code)]
        Pair290(u8, String) => "pair 290",
        #[severity(WARNING)]
        #[code(292, help = "Retry item 291.")]
        #[retryable]
        Unit291 => "unit 291",
        #[category(Unavailable)]
        #[code(293, help = "Check item 292.")]
        /// Variant 292, its doc comment after its metadata.
        Late292 { id: u64 } => "late 292 {id}",
        Wrapped293(#[source] std::fmt::Error) => "wrapped 293",
        #[allow(dead_code)]
        Pair294(u8, String) => "pair 294",
        #[severity(WARNING)]
        #[code(296, help = "Retry item 295.")]
        #[retryable]
        Unit295 => "unit 295",
        #[category(InvalidArgument)]
        #[code(297, help = "Check item 296.")]
        /// Variant 296, its doc comment after its metadata.
        Late296 { id: u64 } => "late 296 {id}",
        Wrapped297(#[source] std::fmt::Error) => "wrapped 297",
        #[allow(dead_code)]
        Pair298(u8, String) => "pair 298",
        #[severity(WARNING)]
        #[code(300, help = "Retry item 299.")]
        #[retryable]
        Unit299 => "unit 299",
        #[category(Internal)]
        #[code(301, help = "Check item 300.")]
        /// Variant 300, its doc comment after its metadata.
        Late300 { id: u64 } => "late 300 {id}",
        Wrapped301(#[source] std::fmt::Error) => "wrapped 301",
        #[allow(dead_code)]
        Pair302(u8, String) => "pair 302",
        #[severity(WARNING)]
        #[code(304, help = "Retry item 303.")]
        #[retryable]
        Unit303 => "unit 303",
        #[category(NotFound)]
        #[code(305, help = "Check item 304.")]
        /// Variant 304, its doc comment after its metadata.
        Late304 { id: u64 } => "late 304 {id}",
        Wrapped305(#[source] std::fmt::Error) => "wrapped 305",
        #[allow(dead_code)]
        Pair306(u8, String) => "pair 306",
        #[severity(WARNING)]
        #[code(308, help = "Retry item 307.")]
        #[retryable]
        Unit307 => "unit 307",
        #[category(Unavailable)]
        #[code(309, help = "Check item 308.")]
        /// Variant 308, its doc comment after its metadata.
        Late308 { id: u64 } => "late 308 {id}",
        Wrapped309(#[source] std::fmt::Error) => "wrapped 309",
        #[allow(dead_code)]
        Pair310(u8, String) => "pair 310",
        #[severity(WARNING)]
        #[code(312, help = "Retry item 311.")]
        #[retryable]
        Unit311 => "unit 311",
        #[category(InvalidArgument)]
        #[code(313, help = "Check item 312.")]
        /// Variant 312, its doc comment after its metadata.
        Late312 { id: u64 } => "late 312 {id}",
        Wrapped313(#[source] std::fmt::Error) => "wrapped 313",
        #[allow(dead_code)]
        Pair314(u8, String) => "pair 314",
        #[severity(WARNING)]
        #[code(316, help = "Retry item 315.")]
        #[retryable]
        Unit315 => "unit 315",
        #[category(Internal)]
        #[code(317, help = "Check item 316.")]
        /// Variant 316, its doc comment after its metadata.
        Late316 { id: u64 } => "late 316 {id}",
        Wrapped317(#[source] std::fmt::Error) => "wrapped 317",
        #[allow(dead_code)]
        Pair318(u8, String) => "pair 318",
        #[severity(WARNING)]
        #[code(320, help = "Retry item 319.")]
        #[retryable]
        Unit319 => "unit 319",
        #[category(NotFound)]
        #[code(321, help = "Check item 320.")]
        /// Variant 320, its doc comment after its metadata.
        Late320 { id: u64 } => "late 320 {id}",
        Wrapped321(#[source] std::fmt::Error) => "wrapped 321",
        #[allow(dead_code)]
        Pair322(u8, String) => "pair 322",
        #[severity(WARNING)]
        #[code(324, help = "Retry item 323.")]
        #[retryable]
        Unit323 => "unit 323",
        #[category(Unavailable)]
        #[code(325, help = "Check item 324.")]
        /// Variant 324, its doc comment after its metadata.
        Late324 { id: u64 } => "late 324 {id}",
        Wrapped325(#[source] std::fmt::Error) => "wrapped 325",
        #[allow(dead_code)]
        Pair326(u8, String) => "pair 326",
        #[severity(WARNING)]
        #[code(328, help = "Retry item 327.")]
        #[retryable]
        Unit327 => "unit 327",
        #[category(InvalidArgument)]
        #[code(329, help = "Check item 328.")]
        /// Variant 328, its doc comment after its metadata.
        Late328 { id: u64 } => "late 328 {id}",
        Wrapped329(#[source] std::fmt::Error) => "wrapped 329",
        #[allow(dead_code)]
        Pair330(u8, String) => "pair 330",
        #[severity(WARNING)]
        #[code(332, help = "Retry item 331.")]
        #[retryable]
        Unit331 => "unit 331",
        #[category(Internal)]
        #[code(333, help = "Check item 332.")]
        /// Variant 332, its doc comment after its metadata.
        Late332 { id: u64 } => "late 332 {id}",
        Wrapped333(#[source] std::fmt::Error) => "wrapped 333",
        #[allow(dead_code)]
        Pair334(u8, String) => "pair 334",
        #[severity(WARNING)]
        #[code(336, help = "Retry item 335.")]
        #[retryable]
        Unit335 => "unit 335",
        #[category(NotFound)]
        #[code(337, help = "Check item 336.")]
        /// Variant 336, its doc comment after its metadata.
        Late336 { id: u64 } => "late 336 {id}",
        Wrapped337(#[source] std::fmt::Error) => "wrapped 337",
        #[allow(dead_code)]
        Pair338(u8, String) => "pair 338",
        #[severity(WARNING)]
        #[code(340, help = "Retry item 339.")]
        #[retryable]
        Unit339 => "unit 339",
        #[category(Unavailable)]
        #[code(341, help = "Check item 340.")]
        /// Variant 340, its doc comment after its metadata.
        Late340 { id: u64 } => "late 340 {id}",
        Wrapped341(#[source] std::fmt::Error) => "wrapped 341",
        #[allow(dead_code)]
        Pair342(u8, String) => "pair 342",
        #[severity(WARNING)]
        #[code(344, help = "Retry item 343.")]
        #[retryable]
        Unit343 => "unit 343",
        #[category(InvalidArgument)]
        #[code(345, help = "Check item 344.")]
        /// Variant 344, its doc comment after its metadata.
        Late344 { id: u64 } => "late 344 {id}",
        Wrapped345(#[source] std::fmt::Error) => "wrapped 345",
        #[allow(dead_code)]
        Pair346(u8, String) => "pair 346",
        #[severity(WARNING)]
        #[code(348, help = "Retry item 347.")]
        #[retryable]
        Unit347 => "unit 347",
        #[category(Internal)]
        #[code(349, help = "Check item 348.")]
        /// Variant 348, its doc comment after its metadata.
        Late348 { id: u64 } => "late 348 {id}",
        Wrapped349(#[source] std::fmt::Error) => "wrapped 349",
        #[allow(dead_code)]
        Pair350(u8, String) => "pair 350",
        #[severity(WARNING)]
        #[code(352, help = "Retry item 351.")]
        #[retryable]
        Unit351 => "unit 351",
        #[category(NotFound)]
        #[code(353, help = "Check item 352.")]
        /// Variant 352, its doc comment after its metadata.
        Late352 { id: u64 } => "late 352 {id}",
        Wrapped353(#[source] std::fmt::Error) => "wrapped 353",
        #[allow(dead_code)]
        Pair354(u8, String) => "pair 354",
        #[severity(WARNING)]
        #[code(356, help = "Retry item 355.")]
        #[retryable]
        Unit355 => "unit 355",
        #[category(Unavailable)]
        #[code(357, help = "Check item 356.")]
        /// Variant 356, its doc comment after its metadata.
        Late356 { id: u64 } => "late 356 {id}",
        Wrapped357(#[source] std::fmt::Error) => "wrapped 357",
        #[allow(dead_code)]
        Pair358(u8, String) => "pair 358",
        #[severity(WARNING)]
        #[code(360, help = "Retry item 359.")]
        #[retryable]
        Unit359 => "unit 359",
        #[category(InvalidArgument)]
        #[code(361, help = "Check item 360.")]
        /// Variant 360, its doc comment after its metadata.
        Late360 { id: u64 } => "late 360 {id}",
        Wrapped361(#[source] std::fmt::Error) => "wrapped 361",
        #[allow(dead_code)]
        Pair362(u8, String) => "pair 362",
        #[severity(WARNING)]
        #[code(364, help = "Retry item 363.")]
        #[retryable]
        Unit363 => "unit 363",
        #[category(Internal)]
        #[code(365, help = "Check item 364.")]
        /// Variant 364, its doc comment after its metadata.
        Late364 { id: u64 } => "late 364 {id}",
        Wrapped365(#[source] std::fmt::Error) => "wrapped 365",
        #[allow(dead_code)]
        Pair366(u8, String) => "pair 366",
        #[severity(WARNING)]
        #[code(368, help = "Retry item 367.")]
        #[retryable]
        Unit367 => "unit 367",
        #[category(NotFound)]
        #[code(369, help = "Check item 368.")]
        /// Variant 368, its doc comment after its metadata.
        Late368 { id: u64 } => "late 368 {id}",
        Wrapped369(#[source] std::fmt::Error) => "wrapped 369",
        #[allow(dead_code)]
        Pair370(u8, String) => "pair 370",
        #[severity(WARNING)]
        #[code(372, help = "Retry item 371.")]
        #[retryable]
        Unit371 => "unit 371",
        #[category(Unavailable)]
        #[code(373, help = "Check item 372.")]
        /// Variant 372, its doc comment after its metadata.
        Late372 { id: u64 } => "late 372 {id}",
        Wrapped373(#[source] std::fmt::Error) => "wrapped 373",
        #[allow(dead_code)]
        Pair374(u8, String) => "pair 374",
        #[severity(WARNING)]
        #[code(376, help = "Retry item 375.")]
        #[retryable]
        Unit375 => "unit 375",
        #[category(InvalidArgument)]
        #[code(377, help = "Check item 376.")]
        /// Variant 376, its doc comment after its metadata.
        Late376 { id: u64 } => "late 376 {id}",
        Wrapped377(#[source] std::fmt::Error) => "wrapped 377",
        #[allow(dead_code)]
        Pair378(u8, String) => "pair 378",
        #[severity(WARNING)]
        #[code(380, help = "Retry item 379.")]
        #[retryable]
        Unit379 => "unit 379",
        #[category(Internal)]
        #[code(381, help = "Check item 380.")]
        /// Variant 380, its doc comment after its metadata.
        Late380 { id: u64 } => "late 380 {id}",
        Wrapped381(#[source] std::fmt::Error) => "wrapped 381",
        #[allow(dead_code)]
        Pair382(u8, String) => "pair 382",
        #[severity(WARNING)]
        #[code(384, help = "Retry item 383.")]
        #[retryable]
        Unit383 => "unit 383",
        #[category(NotFound)]
        #[code(385, help = "Check item 384.")]
        /// Variant 384, its doc comment after its metadata.
        Late384 { id: u64 } => "late 384 {id}",
        Wrapped385(#[source] std::fmt::Error) => "wrapped 385",
        #[allow(dead_code)]
        Pair386(u8, String) => "pair 386",
        #[severity(WARNING)]
        #[code(388, help = "Retry item 387.")]
        #[retryable]
        Unit387 => "unit 387",
        #[category(Unavailable)]
        #[code(389, help = "Check item 388.")]
        /// Variant 388, its doc comment after its metadata.
        Late388 { id: u64 } => "late 388 {id}",
        Wrapped389(#[source] std::fmt::Error) => "wrapped 389",
        #[allow(dead_code)]
        Pair390(u8, String) => "pair 390",
        #[severity(WARNING)]
        #[code(392, help = "Retry item 391.")]
        #[retryable]
        Unit391 => "unit 391",
        #[category(InvalidArgument)]
        #[code(393, help = "Check item 392.")]
        /// Variant 392, its doc comment after its metadata.
        Late392 { id: u64 } => "late 392 {id}",
        Wrapped393(#[source] std::fmt::Error) => "wrapped 393",
        #[allow(dead_code)]
        Pair394(u8, String) => "pair 394",
        #[severity(WARNING)]
        #[code(396, help = "Retry item 395.")]
        #[retryable]
        Unit395 => "unit 395",
        #[category(Internal)]
        #[code(397, help = "Check item 396.")]
        /// Variant 396, its doc comment after its metadata.
        Late396 { id: u64 } => "late 396 {id}",
        Wrapped397(#[source] std::fmt::Error) => "wrapped 397",
        #[allow(dead_code)]
        Pair398(u8, String) => "pair 398",
        #[severity(WARNING)]
        #[code(400, help = "Retry item 399.")]
        #[retryable]
        Unit399 => "unit 399",
        #[category(NotFound)]
        #[code(401, help = "Check item 400.")]
        /// Variant 400, its doc comment after its metadata.
        Late400 { id: u64 } => "late 400 {id}",
        Wrapped401(#[source] std::fmt::Error) => "wrapped 401",
        #[allow(dead_code)]
        Pair402(u8, String) => "pair 402",
        #[severity(WARNING)]
        #[code(404, help = "Retry item 403.")]
        #[retryable]
        Unit403 => "unit 403",
        #[category(Unavailable)]
        #[code(405, help = "Check item 404.")]
        /// Variant 404, its doc comment after its metadata.
        Late404 { id: u64 } => "late 404 {id}",
        Wrapped405(#[source] std::fmt::Error) => "wrapped 405",
        #[allow(dead_code)]
        Pair406(u8, String) => "pair 406",
        #[severity(WARNING)]
        #[code(408, help = "Retry item 407.")]
        #[retryable]
        Unit407 => "unit 407",
        #[category(InvalidArgument)]
        #[code(409, help = "Check item 408.")]
        /// Variant 408, its doc comment after its metadata.
        Late408 { id: u64 } => "late 408 {id}",
        Wrapped409(#[source] std::fmt::Error) => "wrapped 409",
        #[allow(dead_code)]
        Pair410(u8, String) => "pair 410",
        #[severity(WARNING)]
        #[code(412, help = "Retry item 411.")]
        #[retryable]
        Unit411 => "unit 411",
        #[category(Internal)]
        #[code(413, help = "Check item 412.")]
        /// Variant 412, its doc comment after its metadata.
        Late412 { id: u64 } => "late 412 {id}",
        Wrapped413(#[source] std::fmt::Error) => "wrapped 413",
        #[allow(dead_code)]
        Pair414(u8, String) => "pair 414",
        #[severity(WARNING)]
        #[code(416, help = "Retry item 415.")]
        #[retryable]
        Unit415 => "unit 415",
        #[category(NotFound)]
        #[code(417, help = "Check item 416.")]
        /// Variant 416, its doc comment after its metadata.
        Late416 { id: u64 } => "late 416 {id}",
        Wrapped417(#[source] std::fmt::Error) => "wrapped 417",
        #[allow(dead_code)]
        Pair418(u8, String) => "pair 418",
        #[severity(WARNING)]
        #[code(420, help = "Retry item 419.")]
        #[retryable]
        Unit419 => "unit 419",
        #[category(Unavailable)]
        #[code(421, help = "Check item 420.")]
        /// Variant 420, its doc comment after its metadata.
        Late420 { id: u64 } => "late 420 {id}",
        Wrapped421(#[source] std::fmt::Error) => "wrapped 421",
        #[allow(dead_code)]
        Pair422(u8, String) => "pair 422",
        #[severity(WARNING)]
        #[code(424, help = "Retry item 423.")]
        #[retryable]
        Unit423 => "unit 423",
        #[category(InvalidArgument)]
        #[code(425, help = "Check item 424.")]
        /// Variant 424, its doc comment after its metadata.
        Late424 { id: u64 } => "late 424 {id}",
        Wrapped425(#[source] std::fmt::Error) => "wrapped 425",
        #[allow(dead_code)]
        Pair426(u8, String) => "pair 426",
        #[severity(WARNING)]
        #[code(428, help = "Retry item 427.")]
        #[retryable]
        Unit427 => "unit 427",
        #[category(Internal)]
        #[code(429, help = "Check item 428.")]
        /// Variant 428, its doc comment after its metadata.
        Late428 { id: u64 } => "late 428 {id}",
        Wrapped429(#[source] std::fmt::Error) => "wrapped 429",
        #[allow(dead_code)]
        Pair430(u8, String) => "pair 430",
        #[severity(WARNING)]
        #[code(432, help = "Retry item 431.")]
        #[retryable]
        Unit431 => "unit 431",
        #[category(NotFound)]
        #[code(433, help = "Check item 432.")]
        /// Variant 432, its doc comment after its metadata.
        Late432 { id: u64 } => "late 432 {id}",
        Wrapped433(#[source] std::fmt::Error) => "wrapped 433",
        #[allow(dead_code)]
        Pair434(u8, String) => "pair 434",
        #[severity(WARNING)]
        #[code(436, help = "Retry item 435.")]
        #[retryable]
        Unit435 => "unit 435",
        #[category(Unavailable)]
        #[code(437, help = "Check item 436.")]
        /// Variant 436, its doc comment after its metadata.
        Late436 { id: u64 } => "late 436 {id}",
        Wrapped437(#[source] std::fmt::Error) => "wrapped 437",
        #[allow(dead_code)]
        Pair438(u8, String) => "pair 438",
        #[severity(WARNING)]
        #[code(440, help = "Retry item 439.")]
        #[retryable]
        Unit439 => "unit 439",
        #[category(InvalidArgument)]
        #[code(441, help = "Check item 440.")]
        /// Variant 440, its doc comment after its metadata.
        Late440 { id: u64 } => "late 440 {id}",
        Wrapped441(#[source] std::fmt::Error) => "wrapped 441",
        #[allow(dead_code)]
        Pair442(u8, String) => "pair 442",
        #[severity(WARNING)]
        #[code(444, help = "Retry item 443.")]
        #[retryable]
        Unit443 => "unit 443",
        #[category(Internal)]
        #[code(445, help = "Check item 444.")]
        /// Variant 444, its doc comment after its metadata.
        Late444 { id: u64 } => "late 444 {id}",
        Wrapped445(#[source] std::fmt::Error) => "wrapped 445",
        #[allow(dead_code)]
        Pair446(u8, String) => "pair 446",
        #[severity(WARNING)]
        #[code(448, help = "Retry item 447.")]
        #[retryable]
        Unit447 => "unit 447",
        #[category(NotFound)]
        #[code(449, help = "Check item 448.")]
        /// Variant 448, its doc comment after its metadata.
        Late448 { id: u64 } => "late 448 {id}",
        Wrapped449(#[source] std::fmt::Error) => "wrapped 449",
        #[allow(dead_code)]
        Pair450(u8, String) => "pair 450",
        #[severity(WARNING)]
        #[code(452, help = "Retry item 451.")]
        #[retryable]
        Unit451 => "unit 451",
        #[category(Unavailable)]
        #[code(453, help = "Check item 452.")]
        /// Variant 452, its doc comment after its metadata.
        Late452 { id: u64 } => "late 452 {id}",
        Wrapped453(#[source] std::fmt::Error) => "wrapped 453",
        #[allow(dead_code)]
        Pair454(u8, String) => "pair 454",
        #[severity(WARNING)]
        #[code(456, help = "Retry item 455.")]
        #[retryable]
        Unit455 => "unit 455",
        #[category(InvalidArgument)]
        #[code(457, help = "Check item 456.")]
        /// Variant 456, its doc comment after its metadata.
        Late456 { id: u64 } => "late 456 {id}",
        Wrapped457(#[source] std::fmt::Error) => "wrapped 457",
        #[allow(dead_code)]
        Pair458(u8, String) => "pair 458",
        #[severity(WARNING)]
        #[code(460, help = "Retry item 459.")]
        #[retryable]
        Unit459 => "unit 459",
        #[category(Internal)]
        #[code(461, help = "Check item 460.")]
        /// Variant 460, its doc comment after its metadata.
        Late460 { id: u64 } => "late 460 {id}",
        Wrapped461(#[source] std::fmt::Error) => "wrapped 461",
        #[allow(dead_code)]
        Pair462(u8, String) => "pair 462",
        #[severity(WARNING)]
        #[code(464, help = "Retry item 463.")]
        #[retryable]
        Unit463 => "unit 463",
        #[category(NotFound)]
        #[code(465, help = "Check item 464.")]
        /// Variant 464, its doc comment after its metadata.
        Late464 { id: u64 } => "late 464 {id}",
        Wrapped465(#[source] std::fmt::Error) => "wrapped 465",
        #[allow(dead_code)]
        Pair466(u8, String) => "pair 466",
        #[severity(WARNING)]
        #[code(468, help = "Retry item 467.")]
        #[retryable]
        Unit467 => "unit 467",
        #[category(Unavailable)]
        #[code(469, help = "Check item 468.")]
        /// Variant 468, its doc comment after its metadata.
        Late468 { id: u64 } => "late 468 {id}",
        Wrapped469(#[source] std::fmt::Error) => "wrapped 469",
        #[allow(dead_code)]
        Pair470(u8, String) => "pair 470",
        #[severity(WARNING)]
        #[code(472, help = "Retry item 471.")]
        #[retryable]
        Unit471 => "unit 471",
        #[category(InvalidArgument)]
        #[code(473, help = "Check item 472.")]
        /// Variant 472, its doc comment after its metadata.
        Late472 { id: u64 } => "late 472 {id}",
        Wrapped473(#[source] std::fmt::Error) => "wrapped 473",
        #[allow(dead_code)]
        Pair474(u8, String) => "pair 474",
        #[severity(WARNING)]
        #[code(476, help = "Retry item 475.")]
        #[retryable]
        Unit475 => "unit 475",
        #[category(Internal)]
        #[code(477, help = "Check item 476.")]
        /// Variant 476, its doc comment after its metadata.
        Late476 { id: u64 } => "late 476 {id}",
        Wrapped477(#[source] std::fmt::Error) => "wrapped 477",
        #[allow(dead_code)]
        Pair478(u8, String) => "pair 478",
        #[severity(WARNING)]
        #[code(480, help = "Retry item 479.")]
        #[retryable]
        Unit479 => "unit 479",
        #[category(NotFound)]
        #[code(481, help = "Check item 480.")]
        /// Variant 480, its doc comment after its metadata.
        Late480 { id: u64 } => "late 480 {id}",
        Wrapped481(#[source] std::fmt::Error) => "wrapped 481",
        #[allow(dead_code)]
        Pair482(u8, String) => "pair 482",
        #[severity(WARNING)]
        #[code(484, help = "Retry item 483.")]
        #[retryable]
        Unit483 => "unit 483",
        #[category(Unavailable)]
        #[code(485, help = "Check item 484.")]
        /// Variant 484, its doc comment after its metadata.
        Late484 { id: u64 } => "late 484 {id}",
        Wrapped485(#[source] std::fmt::Error) => "wrapped 485",
        #[allow(dead_code)]
        Pair486(u8, String) => "pair 486",
        #[severity(WARNING)]
        #[code(488, help = "Retry item 487.")]
        #[retryable]
        Unit487 => "unit 487",
        #[category(InvalidArgument)]
        #[code(489, help = "Check item 488.")]
        /// Variant 488, its doc comment after its metadata.
        Late488 { id: u64 } => "late 488 {id}",
        Wrapped489(#[source] std::fmt::Error) => "wrapped 489",
        #[allow(dead_code)]
        Pair490(u8, String) => "pair 490",
        #[severity(WARNING)]
        #[code(492, help = "Retry item 491.")]
        #[retryable]
        Unit491 => "unit 491",
        #[category(Internal)]
        #[code(493, help = "Check item 492.")]
        /// Variant 492, its doc comment after its metadata.
        Late492 { id: u64 } => "late 492 {id}",
        Wrapped493(#[source] std::fmt::Error) => "wrapped 493",
        #[allow(dead_code)]
        Pair494(u8, String) => "pair 494",
        #[severity(WARNING)]
        #[code(496, help = "Retry item 495.")]
        #[retryable]
        Unit495 => "unit 495",
        #[category(NotFound)]
        #[code(497, help = "Check item 496.")]
        /// Variant 496, its doc comment after its metadata.
        Late496 { id: u64 } => "late 496 {id}",
        Wrapped497(#[source] std::fmt::Error) => "wrapped 497",
        #[allow(dead_code)]
        Pair498(u8, String) => "pair 498",
        #[severity(WARNING)]
        #[code(500, help = "Retry item 499.")]
        #[retryable]
        Unit499 => "unit 499",
    }
}

#[test]
fn test_any_order_enum_first_and_last_variants() {
    let late = AnyOrder::Late0 { id: 7 };
    assert_eq!(late.to_string(), "late 0 7");
    assert_eq!(
        late.code().map(|code| code.to_string()),
        Some("ANY-0001".to_string())
    );
    assert_eq!(late.category(), Category::NotFound);
    let late = AnyOrder::Late496 { id: 1 };
    assert_eq!(late.to_string(), "late 496 1");
    assert_eq!(late.code().map(|code| code.number()), Some(497));

    let wrapped = AnyOrder::Wrapped1(std::fmt::Error);
    assert_eq!(wrapped.to_string(), "wrapped 1");
    assert!(wrapped.source().is_some());
    assert!(AnyOrder::Wrapped497(std::fmt::Error).source().is_some());

    assert_eq!(AnyOrder::Pair2(1, "x".into()).to_string(), "pair 2");
    assert_eq!(AnyOrder::Pair498(1, "x".into()).to_string(), "pair 498");
    assert!(AnyOrder::Pair2(1, "x".into()).code().is_none());

    let unit = AnyOrder::Unit3;
    assert_eq!(unit.to_string(), "unit 3");
    assert_eq!(unit.severity(), Severity::WARNING);
    assert!(unit.is_retryable());
    assert_eq!(
        AnyOrder::Unit499.code().map(|code| code.number()),
        Some(500)
    );
    assert_eq!(AnyOrder::CATALOG.len(), 250);
}
