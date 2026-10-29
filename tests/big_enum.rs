//! A 500-variant enum of mixed forms compiles at the default `recursion_limit`
//! (NFR-6, DEF-27, E-3): the fast path does not recurse per variant.
//!
//! The variants follow a fixed pattern of three forms; `tests/big_enum.rs` is
//! written by a short script, and the test checks the first and last of each.

use define_err::define_errors;

define_errors! {
    /// Five hundred variants.
    pub enum Big {
        /// Variant 0.
        Unit0 => "unit 0",
        Braced1 { value: u32, #[source] cause: std::fmt::Error } => "braced 1 {value}",
        Tuple2(first: u8, second: String) => "tuple 2 {first} {second}",
        /// Variant 3.
        Unit3 => "unit 3",
        Braced4 { value: u32, #[source] cause: std::fmt::Error } => "braced 4 {value}",
        Tuple5(first: u8, second: String) => "tuple 5 {first} {second}",
        /// Variant 6.
        Unit6 => "unit 6",
        Braced7 { value: u32, #[source] cause: std::fmt::Error } => "braced 7 {value}",
        Tuple8(first: u8, second: String) => "tuple 8 {first} {second}",
        /// Variant 9.
        Unit9 => "unit 9",
        Braced10 { value: u32, #[source] cause: std::fmt::Error } => "braced 10 {value}",
        Tuple11(first: u8, second: String) => "tuple 11 {first} {second}",
        /// Variant 12.
        Unit12 => "unit 12",
        Braced13 { value: u32, #[source] cause: std::fmt::Error } => "braced 13 {value}",
        Tuple14(first: u8, second: String) => "tuple 14 {first} {second}",
        /// Variant 15.
        Unit15 => "unit 15",
        Braced16 { value: u32, #[source] cause: std::fmt::Error } => "braced 16 {value}",
        Tuple17(first: u8, second: String) => "tuple 17 {first} {second}",
        /// Variant 18.
        Unit18 => "unit 18",
        Braced19 { value: u32, #[source] cause: std::fmt::Error } => "braced 19 {value}",
        Tuple20(first: u8, second: String) => "tuple 20 {first} {second}",
        /// Variant 21.
        Unit21 => "unit 21",
        Braced22 { value: u32, #[source] cause: std::fmt::Error } => "braced 22 {value}",
        Tuple23(first: u8, second: String) => "tuple 23 {first} {second}",
        /// Variant 24.
        Unit24 => "unit 24",
        Braced25 { value: u32, #[source] cause: std::fmt::Error } => "braced 25 {value}",
        Tuple26(first: u8, second: String) => "tuple 26 {first} {second}",
        /// Variant 27.
        Unit27 => "unit 27",
        Braced28 { value: u32, #[source] cause: std::fmt::Error } => "braced 28 {value}",
        Tuple29(first: u8, second: String) => "tuple 29 {first} {second}",
        /// Variant 30.
        Unit30 => "unit 30",
        Braced31 { value: u32, #[source] cause: std::fmt::Error } => "braced 31 {value}",
        Tuple32(first: u8, second: String) => "tuple 32 {first} {second}",
        /// Variant 33.
        Unit33 => "unit 33",
        Braced34 { value: u32, #[source] cause: std::fmt::Error } => "braced 34 {value}",
        Tuple35(first: u8, second: String) => "tuple 35 {first} {second}",
        /// Variant 36.
        Unit36 => "unit 36",
        Braced37 { value: u32, #[source] cause: std::fmt::Error } => "braced 37 {value}",
        Tuple38(first: u8, second: String) => "tuple 38 {first} {second}",
        /// Variant 39.
        Unit39 => "unit 39",
        Braced40 { value: u32, #[source] cause: std::fmt::Error } => "braced 40 {value}",
        Tuple41(first: u8, second: String) => "tuple 41 {first} {second}",
        /// Variant 42.
        Unit42 => "unit 42",
        Braced43 { value: u32, #[source] cause: std::fmt::Error } => "braced 43 {value}",
        Tuple44(first: u8, second: String) => "tuple 44 {first} {second}",
        /// Variant 45.
        Unit45 => "unit 45",
        Braced46 { value: u32, #[source] cause: std::fmt::Error } => "braced 46 {value}",
        Tuple47(first: u8, second: String) => "tuple 47 {first} {second}",
        /// Variant 48.
        Unit48 => "unit 48",
        Braced49 { value: u32, #[source] cause: std::fmt::Error } => "braced 49 {value}",
        Tuple50(first: u8, second: String) => "tuple 50 {first} {second}",
        /// Variant 51.
        Unit51 => "unit 51",
        Braced52 { value: u32, #[source] cause: std::fmt::Error } => "braced 52 {value}",
        Tuple53(first: u8, second: String) => "tuple 53 {first} {second}",
        /// Variant 54.
        Unit54 => "unit 54",
        Braced55 { value: u32, #[source] cause: std::fmt::Error } => "braced 55 {value}",
        Tuple56(first: u8, second: String) => "tuple 56 {first} {second}",
        /// Variant 57.
        Unit57 => "unit 57",
        Braced58 { value: u32, #[source] cause: std::fmt::Error } => "braced 58 {value}",
        Tuple59(first: u8, second: String) => "tuple 59 {first} {second}",
        /// Variant 60.
        Unit60 => "unit 60",
        Braced61 { value: u32, #[source] cause: std::fmt::Error } => "braced 61 {value}",
        Tuple62(first: u8, second: String) => "tuple 62 {first} {second}",
        /// Variant 63.
        Unit63 => "unit 63",
        Braced64 { value: u32, #[source] cause: std::fmt::Error } => "braced 64 {value}",
        Tuple65(first: u8, second: String) => "tuple 65 {first} {second}",
        /// Variant 66.
        Unit66 => "unit 66",
        Braced67 { value: u32, #[source] cause: std::fmt::Error } => "braced 67 {value}",
        Tuple68(first: u8, second: String) => "tuple 68 {first} {second}",
        /// Variant 69.
        Unit69 => "unit 69",
        Braced70 { value: u32, #[source] cause: std::fmt::Error } => "braced 70 {value}",
        Tuple71(first: u8, second: String) => "tuple 71 {first} {second}",
        /// Variant 72.
        Unit72 => "unit 72",
        Braced73 { value: u32, #[source] cause: std::fmt::Error } => "braced 73 {value}",
        Tuple74(first: u8, second: String) => "tuple 74 {first} {second}",
        /// Variant 75.
        Unit75 => "unit 75",
        Braced76 { value: u32, #[source] cause: std::fmt::Error } => "braced 76 {value}",
        Tuple77(first: u8, second: String) => "tuple 77 {first} {second}",
        /// Variant 78.
        Unit78 => "unit 78",
        Braced79 { value: u32, #[source] cause: std::fmt::Error } => "braced 79 {value}",
        Tuple80(first: u8, second: String) => "tuple 80 {first} {second}",
        /// Variant 81.
        Unit81 => "unit 81",
        Braced82 { value: u32, #[source] cause: std::fmt::Error } => "braced 82 {value}",
        Tuple83(first: u8, second: String) => "tuple 83 {first} {second}",
        /// Variant 84.
        Unit84 => "unit 84",
        Braced85 { value: u32, #[source] cause: std::fmt::Error } => "braced 85 {value}",
        Tuple86(first: u8, second: String) => "tuple 86 {first} {second}",
        /// Variant 87.
        Unit87 => "unit 87",
        Braced88 { value: u32, #[source] cause: std::fmt::Error } => "braced 88 {value}",
        Tuple89(first: u8, second: String) => "tuple 89 {first} {second}",
        /// Variant 90.
        Unit90 => "unit 90",
        Braced91 { value: u32, #[source] cause: std::fmt::Error } => "braced 91 {value}",
        Tuple92(first: u8, second: String) => "tuple 92 {first} {second}",
        /// Variant 93.
        Unit93 => "unit 93",
        Braced94 { value: u32, #[source] cause: std::fmt::Error } => "braced 94 {value}",
        Tuple95(first: u8, second: String) => "tuple 95 {first} {second}",
        /// Variant 96.
        Unit96 => "unit 96",
        Braced97 { value: u32, #[source] cause: std::fmt::Error } => "braced 97 {value}",
        Tuple98(first: u8, second: String) => "tuple 98 {first} {second}",
        /// Variant 99.
        Unit99 => "unit 99",
        Braced100 { value: u32, #[source] cause: std::fmt::Error } => "braced 100 {value}",
        Tuple101(first: u8, second: String) => "tuple 101 {first} {second}",
        /// Variant 102.
        Unit102 => "unit 102",
        Braced103 { value: u32, #[source] cause: std::fmt::Error } => "braced 103 {value}",
        Tuple104(first: u8, second: String) => "tuple 104 {first} {second}",
        /// Variant 105.
        Unit105 => "unit 105",
        Braced106 { value: u32, #[source] cause: std::fmt::Error } => "braced 106 {value}",
        Tuple107(first: u8, second: String) => "tuple 107 {first} {second}",
        /// Variant 108.
        Unit108 => "unit 108",
        Braced109 { value: u32, #[source] cause: std::fmt::Error } => "braced 109 {value}",
        Tuple110(first: u8, second: String) => "tuple 110 {first} {second}",
        /// Variant 111.
        Unit111 => "unit 111",
        Braced112 { value: u32, #[source] cause: std::fmt::Error } => "braced 112 {value}",
        Tuple113(first: u8, second: String) => "tuple 113 {first} {second}",
        /// Variant 114.
        Unit114 => "unit 114",
        Braced115 { value: u32, #[source] cause: std::fmt::Error } => "braced 115 {value}",
        Tuple116(first: u8, second: String) => "tuple 116 {first} {second}",
        /// Variant 117.
        Unit117 => "unit 117",
        Braced118 { value: u32, #[source] cause: std::fmt::Error } => "braced 118 {value}",
        Tuple119(first: u8, second: String) => "tuple 119 {first} {second}",
        /// Variant 120.
        Unit120 => "unit 120",
        Braced121 { value: u32, #[source] cause: std::fmt::Error } => "braced 121 {value}",
        Tuple122(first: u8, second: String) => "tuple 122 {first} {second}",
        /// Variant 123.
        Unit123 => "unit 123",
        Braced124 { value: u32, #[source] cause: std::fmt::Error } => "braced 124 {value}",
        Tuple125(first: u8, second: String) => "tuple 125 {first} {second}",
        /// Variant 126.
        Unit126 => "unit 126",
        Braced127 { value: u32, #[source] cause: std::fmt::Error } => "braced 127 {value}",
        Tuple128(first: u8, second: String) => "tuple 128 {first} {second}",
        /// Variant 129.
        Unit129 => "unit 129",
        Braced130 { value: u32, #[source] cause: std::fmt::Error } => "braced 130 {value}",
        Tuple131(first: u8, second: String) => "tuple 131 {first} {second}",
        /// Variant 132.
        Unit132 => "unit 132",
        Braced133 { value: u32, #[source] cause: std::fmt::Error } => "braced 133 {value}",
        Tuple134(first: u8, second: String) => "tuple 134 {first} {second}",
        /// Variant 135.
        Unit135 => "unit 135",
        Braced136 { value: u32, #[source] cause: std::fmt::Error } => "braced 136 {value}",
        Tuple137(first: u8, second: String) => "tuple 137 {first} {second}",
        /// Variant 138.
        Unit138 => "unit 138",
        Braced139 { value: u32, #[source] cause: std::fmt::Error } => "braced 139 {value}",
        Tuple140(first: u8, second: String) => "tuple 140 {first} {second}",
        /// Variant 141.
        Unit141 => "unit 141",
        Braced142 { value: u32, #[source] cause: std::fmt::Error } => "braced 142 {value}",
        Tuple143(first: u8, second: String) => "tuple 143 {first} {second}",
        /// Variant 144.
        Unit144 => "unit 144",
        Braced145 { value: u32, #[source] cause: std::fmt::Error } => "braced 145 {value}",
        Tuple146(first: u8, second: String) => "tuple 146 {first} {second}",
        /// Variant 147.
        Unit147 => "unit 147",
        Braced148 { value: u32, #[source] cause: std::fmt::Error } => "braced 148 {value}",
        Tuple149(first: u8, second: String) => "tuple 149 {first} {second}",
        /// Variant 150.
        Unit150 => "unit 150",
        Braced151 { value: u32, #[source] cause: std::fmt::Error } => "braced 151 {value}",
        Tuple152(first: u8, second: String) => "tuple 152 {first} {second}",
        /// Variant 153.
        Unit153 => "unit 153",
        Braced154 { value: u32, #[source] cause: std::fmt::Error } => "braced 154 {value}",
        Tuple155(first: u8, second: String) => "tuple 155 {first} {second}",
        /// Variant 156.
        Unit156 => "unit 156",
        Braced157 { value: u32, #[source] cause: std::fmt::Error } => "braced 157 {value}",
        Tuple158(first: u8, second: String) => "tuple 158 {first} {second}",
        /// Variant 159.
        Unit159 => "unit 159",
        Braced160 { value: u32, #[source] cause: std::fmt::Error } => "braced 160 {value}",
        Tuple161(first: u8, second: String) => "tuple 161 {first} {second}",
        /// Variant 162.
        Unit162 => "unit 162",
        Braced163 { value: u32, #[source] cause: std::fmt::Error } => "braced 163 {value}",
        Tuple164(first: u8, second: String) => "tuple 164 {first} {second}",
        /// Variant 165.
        Unit165 => "unit 165",
        Braced166 { value: u32, #[source] cause: std::fmt::Error } => "braced 166 {value}",
        Tuple167(first: u8, second: String) => "tuple 167 {first} {second}",
        /// Variant 168.
        Unit168 => "unit 168",
        Braced169 { value: u32, #[source] cause: std::fmt::Error } => "braced 169 {value}",
        Tuple170(first: u8, second: String) => "tuple 170 {first} {second}",
        /// Variant 171.
        Unit171 => "unit 171",
        Braced172 { value: u32, #[source] cause: std::fmt::Error } => "braced 172 {value}",
        Tuple173(first: u8, second: String) => "tuple 173 {first} {second}",
        /// Variant 174.
        Unit174 => "unit 174",
        Braced175 { value: u32, #[source] cause: std::fmt::Error } => "braced 175 {value}",
        Tuple176(first: u8, second: String) => "tuple 176 {first} {second}",
        /// Variant 177.
        Unit177 => "unit 177",
        Braced178 { value: u32, #[source] cause: std::fmt::Error } => "braced 178 {value}",
        Tuple179(first: u8, second: String) => "tuple 179 {first} {second}",
        /// Variant 180.
        Unit180 => "unit 180",
        Braced181 { value: u32, #[source] cause: std::fmt::Error } => "braced 181 {value}",
        Tuple182(first: u8, second: String) => "tuple 182 {first} {second}",
        /// Variant 183.
        Unit183 => "unit 183",
        Braced184 { value: u32, #[source] cause: std::fmt::Error } => "braced 184 {value}",
        Tuple185(first: u8, second: String) => "tuple 185 {first} {second}",
        /// Variant 186.
        Unit186 => "unit 186",
        Braced187 { value: u32, #[source] cause: std::fmt::Error } => "braced 187 {value}",
        Tuple188(first: u8, second: String) => "tuple 188 {first} {second}",
        /// Variant 189.
        Unit189 => "unit 189",
        Braced190 { value: u32, #[source] cause: std::fmt::Error } => "braced 190 {value}",
        Tuple191(first: u8, second: String) => "tuple 191 {first} {second}",
        /// Variant 192.
        Unit192 => "unit 192",
        Braced193 { value: u32, #[source] cause: std::fmt::Error } => "braced 193 {value}",
        Tuple194(first: u8, second: String) => "tuple 194 {first} {second}",
        /// Variant 195.
        Unit195 => "unit 195",
        Braced196 { value: u32, #[source] cause: std::fmt::Error } => "braced 196 {value}",
        Tuple197(first: u8, second: String) => "tuple 197 {first} {second}",
        /// Variant 198.
        Unit198 => "unit 198",
        Braced199 { value: u32, #[source] cause: std::fmt::Error } => "braced 199 {value}",
        Tuple200(first: u8, second: String) => "tuple 200 {first} {second}",
        /// Variant 201.
        Unit201 => "unit 201",
        Braced202 { value: u32, #[source] cause: std::fmt::Error } => "braced 202 {value}",
        Tuple203(first: u8, second: String) => "tuple 203 {first} {second}",
        /// Variant 204.
        Unit204 => "unit 204",
        Braced205 { value: u32, #[source] cause: std::fmt::Error } => "braced 205 {value}",
        Tuple206(first: u8, second: String) => "tuple 206 {first} {second}",
        /// Variant 207.
        Unit207 => "unit 207",
        Braced208 { value: u32, #[source] cause: std::fmt::Error } => "braced 208 {value}",
        Tuple209(first: u8, second: String) => "tuple 209 {first} {second}",
        /// Variant 210.
        Unit210 => "unit 210",
        Braced211 { value: u32, #[source] cause: std::fmt::Error } => "braced 211 {value}",
        Tuple212(first: u8, second: String) => "tuple 212 {first} {second}",
        /// Variant 213.
        Unit213 => "unit 213",
        Braced214 { value: u32, #[source] cause: std::fmt::Error } => "braced 214 {value}",
        Tuple215(first: u8, second: String) => "tuple 215 {first} {second}",
        /// Variant 216.
        Unit216 => "unit 216",
        Braced217 { value: u32, #[source] cause: std::fmt::Error } => "braced 217 {value}",
        Tuple218(first: u8, second: String) => "tuple 218 {first} {second}",
        /// Variant 219.
        Unit219 => "unit 219",
        Braced220 { value: u32, #[source] cause: std::fmt::Error } => "braced 220 {value}",
        Tuple221(first: u8, second: String) => "tuple 221 {first} {second}",
        /// Variant 222.
        Unit222 => "unit 222",
        Braced223 { value: u32, #[source] cause: std::fmt::Error } => "braced 223 {value}",
        Tuple224(first: u8, second: String) => "tuple 224 {first} {second}",
        /// Variant 225.
        Unit225 => "unit 225",
        Braced226 { value: u32, #[source] cause: std::fmt::Error } => "braced 226 {value}",
        Tuple227(first: u8, second: String) => "tuple 227 {first} {second}",
        /// Variant 228.
        Unit228 => "unit 228",
        Braced229 { value: u32, #[source] cause: std::fmt::Error } => "braced 229 {value}",
        Tuple230(first: u8, second: String) => "tuple 230 {first} {second}",
        /// Variant 231.
        Unit231 => "unit 231",
        Braced232 { value: u32, #[source] cause: std::fmt::Error } => "braced 232 {value}",
        Tuple233(first: u8, second: String) => "tuple 233 {first} {second}",
        /// Variant 234.
        Unit234 => "unit 234",
        Braced235 { value: u32, #[source] cause: std::fmt::Error } => "braced 235 {value}",
        Tuple236(first: u8, second: String) => "tuple 236 {first} {second}",
        /// Variant 237.
        Unit237 => "unit 237",
        Braced238 { value: u32, #[source] cause: std::fmt::Error } => "braced 238 {value}",
        Tuple239(first: u8, second: String) => "tuple 239 {first} {second}",
        /// Variant 240.
        Unit240 => "unit 240",
        Braced241 { value: u32, #[source] cause: std::fmt::Error } => "braced 241 {value}",
        Tuple242(first: u8, second: String) => "tuple 242 {first} {second}",
        /// Variant 243.
        Unit243 => "unit 243",
        Braced244 { value: u32, #[source] cause: std::fmt::Error } => "braced 244 {value}",
        Tuple245(first: u8, second: String) => "tuple 245 {first} {second}",
        /// Variant 246.
        Unit246 => "unit 246",
        Braced247 { value: u32, #[source] cause: std::fmt::Error } => "braced 247 {value}",
        Tuple248(first: u8, second: String) => "tuple 248 {first} {second}",
        /// Variant 249.
        Unit249 => "unit 249",
        Braced250 { value: u32, #[source] cause: std::fmt::Error } => "braced 250 {value}",
        Tuple251(first: u8, second: String) => "tuple 251 {first} {second}",
        /// Variant 252.
        Unit252 => "unit 252",
        Braced253 { value: u32, #[source] cause: std::fmt::Error } => "braced 253 {value}",
        Tuple254(first: u8, second: String) => "tuple 254 {first} {second}",
        /// Variant 255.
        Unit255 => "unit 255",
        Braced256 { value: u32, #[source] cause: std::fmt::Error } => "braced 256 {value}",
        Tuple257(first: u8, second: String) => "tuple 257 {first} {second}",
        /// Variant 258.
        Unit258 => "unit 258",
        Braced259 { value: u32, #[source] cause: std::fmt::Error } => "braced 259 {value}",
        Tuple260(first: u8, second: String) => "tuple 260 {first} {second}",
        /// Variant 261.
        Unit261 => "unit 261",
        Braced262 { value: u32, #[source] cause: std::fmt::Error } => "braced 262 {value}",
        Tuple263(first: u8, second: String) => "tuple 263 {first} {second}",
        /// Variant 264.
        Unit264 => "unit 264",
        Braced265 { value: u32, #[source] cause: std::fmt::Error } => "braced 265 {value}",
        Tuple266(first: u8, second: String) => "tuple 266 {first} {second}",
        /// Variant 267.
        Unit267 => "unit 267",
        Braced268 { value: u32, #[source] cause: std::fmt::Error } => "braced 268 {value}",
        Tuple269(first: u8, second: String) => "tuple 269 {first} {second}",
        /// Variant 270.
        Unit270 => "unit 270",
        Braced271 { value: u32, #[source] cause: std::fmt::Error } => "braced 271 {value}",
        Tuple272(first: u8, second: String) => "tuple 272 {first} {second}",
        /// Variant 273.
        Unit273 => "unit 273",
        Braced274 { value: u32, #[source] cause: std::fmt::Error } => "braced 274 {value}",
        Tuple275(first: u8, second: String) => "tuple 275 {first} {second}",
        /// Variant 276.
        Unit276 => "unit 276",
        Braced277 { value: u32, #[source] cause: std::fmt::Error } => "braced 277 {value}",
        Tuple278(first: u8, second: String) => "tuple 278 {first} {second}",
        /// Variant 279.
        Unit279 => "unit 279",
        Braced280 { value: u32, #[source] cause: std::fmt::Error } => "braced 280 {value}",
        Tuple281(first: u8, second: String) => "tuple 281 {first} {second}",
        /// Variant 282.
        Unit282 => "unit 282",
        Braced283 { value: u32, #[source] cause: std::fmt::Error } => "braced 283 {value}",
        Tuple284(first: u8, second: String) => "tuple 284 {first} {second}",
        /// Variant 285.
        Unit285 => "unit 285",
        Braced286 { value: u32, #[source] cause: std::fmt::Error } => "braced 286 {value}",
        Tuple287(first: u8, second: String) => "tuple 287 {first} {second}",
        /// Variant 288.
        Unit288 => "unit 288",
        Braced289 { value: u32, #[source] cause: std::fmt::Error } => "braced 289 {value}",
        Tuple290(first: u8, second: String) => "tuple 290 {first} {second}",
        /// Variant 291.
        Unit291 => "unit 291",
        Braced292 { value: u32, #[source] cause: std::fmt::Error } => "braced 292 {value}",
        Tuple293(first: u8, second: String) => "tuple 293 {first} {second}",
        /// Variant 294.
        Unit294 => "unit 294",
        Braced295 { value: u32, #[source] cause: std::fmt::Error } => "braced 295 {value}",
        Tuple296(first: u8, second: String) => "tuple 296 {first} {second}",
        /// Variant 297.
        Unit297 => "unit 297",
        Braced298 { value: u32, #[source] cause: std::fmt::Error } => "braced 298 {value}",
        Tuple299(first: u8, second: String) => "tuple 299 {first} {second}",
        /// Variant 300.
        Unit300 => "unit 300",
        Braced301 { value: u32, #[source] cause: std::fmt::Error } => "braced 301 {value}",
        Tuple302(first: u8, second: String) => "tuple 302 {first} {second}",
        /// Variant 303.
        Unit303 => "unit 303",
        Braced304 { value: u32, #[source] cause: std::fmt::Error } => "braced 304 {value}",
        Tuple305(first: u8, second: String) => "tuple 305 {first} {second}",
        /// Variant 306.
        Unit306 => "unit 306",
        Braced307 { value: u32, #[source] cause: std::fmt::Error } => "braced 307 {value}",
        Tuple308(first: u8, second: String) => "tuple 308 {first} {second}",
        /// Variant 309.
        Unit309 => "unit 309",
        Braced310 { value: u32, #[source] cause: std::fmt::Error } => "braced 310 {value}",
        Tuple311(first: u8, second: String) => "tuple 311 {first} {second}",
        /// Variant 312.
        Unit312 => "unit 312",
        Braced313 { value: u32, #[source] cause: std::fmt::Error } => "braced 313 {value}",
        Tuple314(first: u8, second: String) => "tuple 314 {first} {second}",
        /// Variant 315.
        Unit315 => "unit 315",
        Braced316 { value: u32, #[source] cause: std::fmt::Error } => "braced 316 {value}",
        Tuple317(first: u8, second: String) => "tuple 317 {first} {second}",
        /// Variant 318.
        Unit318 => "unit 318",
        Braced319 { value: u32, #[source] cause: std::fmt::Error } => "braced 319 {value}",
        Tuple320(first: u8, second: String) => "tuple 320 {first} {second}",
        /// Variant 321.
        Unit321 => "unit 321",
        Braced322 { value: u32, #[source] cause: std::fmt::Error } => "braced 322 {value}",
        Tuple323(first: u8, second: String) => "tuple 323 {first} {second}",
        /// Variant 324.
        Unit324 => "unit 324",
        Braced325 { value: u32, #[source] cause: std::fmt::Error } => "braced 325 {value}",
        Tuple326(first: u8, second: String) => "tuple 326 {first} {second}",
        /// Variant 327.
        Unit327 => "unit 327",
        Braced328 { value: u32, #[source] cause: std::fmt::Error } => "braced 328 {value}",
        Tuple329(first: u8, second: String) => "tuple 329 {first} {second}",
        /// Variant 330.
        Unit330 => "unit 330",
        Braced331 { value: u32, #[source] cause: std::fmt::Error } => "braced 331 {value}",
        Tuple332(first: u8, second: String) => "tuple 332 {first} {second}",
        /// Variant 333.
        Unit333 => "unit 333",
        Braced334 { value: u32, #[source] cause: std::fmt::Error } => "braced 334 {value}",
        Tuple335(first: u8, second: String) => "tuple 335 {first} {second}",
        /// Variant 336.
        Unit336 => "unit 336",
        Braced337 { value: u32, #[source] cause: std::fmt::Error } => "braced 337 {value}",
        Tuple338(first: u8, second: String) => "tuple 338 {first} {second}",
        /// Variant 339.
        Unit339 => "unit 339",
        Braced340 { value: u32, #[source] cause: std::fmt::Error } => "braced 340 {value}",
        Tuple341(first: u8, second: String) => "tuple 341 {first} {second}",
        /// Variant 342.
        Unit342 => "unit 342",
        Braced343 { value: u32, #[source] cause: std::fmt::Error } => "braced 343 {value}",
        Tuple344(first: u8, second: String) => "tuple 344 {first} {second}",
        /// Variant 345.
        Unit345 => "unit 345",
        Braced346 { value: u32, #[source] cause: std::fmt::Error } => "braced 346 {value}",
        Tuple347(first: u8, second: String) => "tuple 347 {first} {second}",
        /// Variant 348.
        Unit348 => "unit 348",
        Braced349 { value: u32, #[source] cause: std::fmt::Error } => "braced 349 {value}",
        Tuple350(first: u8, second: String) => "tuple 350 {first} {second}",
        /// Variant 351.
        Unit351 => "unit 351",
        Braced352 { value: u32, #[source] cause: std::fmt::Error } => "braced 352 {value}",
        Tuple353(first: u8, second: String) => "tuple 353 {first} {second}",
        /// Variant 354.
        Unit354 => "unit 354",
        Braced355 { value: u32, #[source] cause: std::fmt::Error } => "braced 355 {value}",
        Tuple356(first: u8, second: String) => "tuple 356 {first} {second}",
        /// Variant 357.
        Unit357 => "unit 357",
        Braced358 { value: u32, #[source] cause: std::fmt::Error } => "braced 358 {value}",
        Tuple359(first: u8, second: String) => "tuple 359 {first} {second}",
        /// Variant 360.
        Unit360 => "unit 360",
        Braced361 { value: u32, #[source] cause: std::fmt::Error } => "braced 361 {value}",
        Tuple362(first: u8, second: String) => "tuple 362 {first} {second}",
        /// Variant 363.
        Unit363 => "unit 363",
        Braced364 { value: u32, #[source] cause: std::fmt::Error } => "braced 364 {value}",
        Tuple365(first: u8, second: String) => "tuple 365 {first} {second}",
        /// Variant 366.
        Unit366 => "unit 366",
        Braced367 { value: u32, #[source] cause: std::fmt::Error } => "braced 367 {value}",
        Tuple368(first: u8, second: String) => "tuple 368 {first} {second}",
        /// Variant 369.
        Unit369 => "unit 369",
        Braced370 { value: u32, #[source] cause: std::fmt::Error } => "braced 370 {value}",
        Tuple371(first: u8, second: String) => "tuple 371 {first} {second}",
        /// Variant 372.
        Unit372 => "unit 372",
        Braced373 { value: u32, #[source] cause: std::fmt::Error } => "braced 373 {value}",
        Tuple374(first: u8, second: String) => "tuple 374 {first} {second}",
        /// Variant 375.
        Unit375 => "unit 375",
        Braced376 { value: u32, #[source] cause: std::fmt::Error } => "braced 376 {value}",
        Tuple377(first: u8, second: String) => "tuple 377 {first} {second}",
        /// Variant 378.
        Unit378 => "unit 378",
        Braced379 { value: u32, #[source] cause: std::fmt::Error } => "braced 379 {value}",
        Tuple380(first: u8, second: String) => "tuple 380 {first} {second}",
        /// Variant 381.
        Unit381 => "unit 381",
        Braced382 { value: u32, #[source] cause: std::fmt::Error } => "braced 382 {value}",
        Tuple383(first: u8, second: String) => "tuple 383 {first} {second}",
        /// Variant 384.
        Unit384 => "unit 384",
        Braced385 { value: u32, #[source] cause: std::fmt::Error } => "braced 385 {value}",
        Tuple386(first: u8, second: String) => "tuple 386 {first} {second}",
        /// Variant 387.
        Unit387 => "unit 387",
        Braced388 { value: u32, #[source] cause: std::fmt::Error } => "braced 388 {value}",
        Tuple389(first: u8, second: String) => "tuple 389 {first} {second}",
        /// Variant 390.
        Unit390 => "unit 390",
        Braced391 { value: u32, #[source] cause: std::fmt::Error } => "braced 391 {value}",
        Tuple392(first: u8, second: String) => "tuple 392 {first} {second}",
        /// Variant 393.
        Unit393 => "unit 393",
        Braced394 { value: u32, #[source] cause: std::fmt::Error } => "braced 394 {value}",
        Tuple395(first: u8, second: String) => "tuple 395 {first} {second}",
        /// Variant 396.
        Unit396 => "unit 396",
        Braced397 { value: u32, #[source] cause: std::fmt::Error } => "braced 397 {value}",
        Tuple398(first: u8, second: String) => "tuple 398 {first} {second}",
        /// Variant 399.
        Unit399 => "unit 399",
        Braced400 { value: u32, #[source] cause: std::fmt::Error } => "braced 400 {value}",
        Tuple401(first: u8, second: String) => "tuple 401 {first} {second}",
        /// Variant 402.
        Unit402 => "unit 402",
        Braced403 { value: u32, #[source] cause: std::fmt::Error } => "braced 403 {value}",
        Tuple404(first: u8, second: String) => "tuple 404 {first} {second}",
        /// Variant 405.
        Unit405 => "unit 405",
        Braced406 { value: u32, #[source] cause: std::fmt::Error } => "braced 406 {value}",
        Tuple407(first: u8, second: String) => "tuple 407 {first} {second}",
        /// Variant 408.
        Unit408 => "unit 408",
        Braced409 { value: u32, #[source] cause: std::fmt::Error } => "braced 409 {value}",
        Tuple410(first: u8, second: String) => "tuple 410 {first} {second}",
        /// Variant 411.
        Unit411 => "unit 411",
        Braced412 { value: u32, #[source] cause: std::fmt::Error } => "braced 412 {value}",
        Tuple413(first: u8, second: String) => "tuple 413 {first} {second}",
        /// Variant 414.
        Unit414 => "unit 414",
        Braced415 { value: u32, #[source] cause: std::fmt::Error } => "braced 415 {value}",
        Tuple416(first: u8, second: String) => "tuple 416 {first} {second}",
        /// Variant 417.
        Unit417 => "unit 417",
        Braced418 { value: u32, #[source] cause: std::fmt::Error } => "braced 418 {value}",
        Tuple419(first: u8, second: String) => "tuple 419 {first} {second}",
        /// Variant 420.
        Unit420 => "unit 420",
        Braced421 { value: u32, #[source] cause: std::fmt::Error } => "braced 421 {value}",
        Tuple422(first: u8, second: String) => "tuple 422 {first} {second}",
        /// Variant 423.
        Unit423 => "unit 423",
        Braced424 { value: u32, #[source] cause: std::fmt::Error } => "braced 424 {value}",
        Tuple425(first: u8, second: String) => "tuple 425 {first} {second}",
        /// Variant 426.
        Unit426 => "unit 426",
        Braced427 { value: u32, #[source] cause: std::fmt::Error } => "braced 427 {value}",
        Tuple428(first: u8, second: String) => "tuple 428 {first} {second}",
        /// Variant 429.
        Unit429 => "unit 429",
        Braced430 { value: u32, #[source] cause: std::fmt::Error } => "braced 430 {value}",
        Tuple431(first: u8, second: String) => "tuple 431 {first} {second}",
        /// Variant 432.
        Unit432 => "unit 432",
        Braced433 { value: u32, #[source] cause: std::fmt::Error } => "braced 433 {value}",
        Tuple434(first: u8, second: String) => "tuple 434 {first} {second}",
        /// Variant 435.
        Unit435 => "unit 435",
        Braced436 { value: u32, #[source] cause: std::fmt::Error } => "braced 436 {value}",
        Tuple437(first: u8, second: String) => "tuple 437 {first} {second}",
        /// Variant 438.
        Unit438 => "unit 438",
        Braced439 { value: u32, #[source] cause: std::fmt::Error } => "braced 439 {value}",
        Tuple440(first: u8, second: String) => "tuple 440 {first} {second}",
        /// Variant 441.
        Unit441 => "unit 441",
        Braced442 { value: u32, #[source] cause: std::fmt::Error } => "braced 442 {value}",
        Tuple443(first: u8, second: String) => "tuple 443 {first} {second}",
        /// Variant 444.
        Unit444 => "unit 444",
        Braced445 { value: u32, #[source] cause: std::fmt::Error } => "braced 445 {value}",
        Tuple446(first: u8, second: String) => "tuple 446 {first} {second}",
        /// Variant 447.
        Unit447 => "unit 447",
        Braced448 { value: u32, #[source] cause: std::fmt::Error } => "braced 448 {value}",
        Tuple449(first: u8, second: String) => "tuple 449 {first} {second}",
        /// Variant 450.
        Unit450 => "unit 450",
        Braced451 { value: u32, #[source] cause: std::fmt::Error } => "braced 451 {value}",
        Tuple452(first: u8, second: String) => "tuple 452 {first} {second}",
        /// Variant 453.
        Unit453 => "unit 453",
        Braced454 { value: u32, #[source] cause: std::fmt::Error } => "braced 454 {value}",
        Tuple455(first: u8, second: String) => "tuple 455 {first} {second}",
        /// Variant 456.
        Unit456 => "unit 456",
        Braced457 { value: u32, #[source] cause: std::fmt::Error } => "braced 457 {value}",
        Tuple458(first: u8, second: String) => "tuple 458 {first} {second}",
        /// Variant 459.
        Unit459 => "unit 459",
        Braced460 { value: u32, #[source] cause: std::fmt::Error } => "braced 460 {value}",
        Tuple461(first: u8, second: String) => "tuple 461 {first} {second}",
        /// Variant 462.
        Unit462 => "unit 462",
        Braced463 { value: u32, #[source] cause: std::fmt::Error } => "braced 463 {value}",
        Tuple464(first: u8, second: String) => "tuple 464 {first} {second}",
        /// Variant 465.
        Unit465 => "unit 465",
        Braced466 { value: u32, #[source] cause: std::fmt::Error } => "braced 466 {value}",
        Tuple467(first: u8, second: String) => "tuple 467 {first} {second}",
        /// Variant 468.
        Unit468 => "unit 468",
        Braced469 { value: u32, #[source] cause: std::fmt::Error } => "braced 469 {value}",
        Tuple470(first: u8, second: String) => "tuple 470 {first} {second}",
        /// Variant 471.
        Unit471 => "unit 471",
        Braced472 { value: u32, #[source] cause: std::fmt::Error } => "braced 472 {value}",
        Tuple473(first: u8, second: String) => "tuple 473 {first} {second}",
        /// Variant 474.
        Unit474 => "unit 474",
        Braced475 { value: u32, #[source] cause: std::fmt::Error } => "braced 475 {value}",
        Tuple476(first: u8, second: String) => "tuple 476 {first} {second}",
        /// Variant 477.
        Unit477 => "unit 477",
        Braced478 { value: u32, #[source] cause: std::fmt::Error } => "braced 478 {value}",
        Tuple479(first: u8, second: String) => "tuple 479 {first} {second}",
        /// Variant 480.
        Unit480 => "unit 480",
        Braced481 { value: u32, #[source] cause: std::fmt::Error } => "braced 481 {value}",
        Tuple482(first: u8, second: String) => "tuple 482 {first} {second}",
        /// Variant 483.
        Unit483 => "unit 483",
        Braced484 { value: u32, #[source] cause: std::fmt::Error } => "braced 484 {value}",
        Tuple485(first: u8, second: String) => "tuple 485 {first} {second}",
        /// Variant 486.
        Unit486 => "unit 486",
        Braced487 { value: u32, #[source] cause: std::fmt::Error } => "braced 487 {value}",
        Tuple488(first: u8, second: String) => "tuple 488 {first} {second}",
        /// Variant 489.
        Unit489 => "unit 489",
        Braced490 { value: u32, #[source] cause: std::fmt::Error } => "braced 490 {value}",
        Tuple491(first: u8, second: String) => "tuple 491 {first} {second}",
        /// Variant 492.
        Unit492 => "unit 492",
        Braced493 { value: u32, #[source] cause: std::fmt::Error } => "braced 493 {value}",
        Tuple494(first: u8, second: String) => "tuple 494 {first} {second}",
        /// Variant 495.
        Unit495 => "unit 495",
        Braced496 { value: u32, #[source] cause: std::fmt::Error } => "braced 496 {value}",
        Tuple497(first: u8, second: String) => "tuple 497 {first} {second}",
        /// Variant 498.
        Unit498 => "unit 498",
        Braced499 { value: u32, #[source] cause: std::fmt::Error } => "braced 499 {value}",
    }
}

#[test]
fn test_big_enum_first_and_last_variants() {
    assert_eq!(Big::Unit0.to_string(), "unit 0");
    assert_eq!(
        Big::Braced1 {
            value: 9,
            cause: std::fmt::Error
        }
        .to_string(),
        "braced 1 9"
    );
    assert_eq!(Big::Tuple2(1, "x".into()).to_string(), "tuple 2 1 x");
    assert_eq!(
        Big::Braced499 {
            value: 1,
            cause: std::fmt::Error
        }
        .to_string(),
        "braced 499 1"
    );
    assert_eq!(Big::Tuple497(3, "y".into()).to_string(), "tuple 497 3 y");
    assert_eq!(Big::Unit498.to_string(), "unit 498");
    assert!(std::error::Error::source(&Big::Braced4 {
        value: 0,
        cause: std::fmt::Error
    })
    .is_some());
}
