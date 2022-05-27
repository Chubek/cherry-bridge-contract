pub mod utils {

    use blake2::digest::{Update, VariableOutput};
    use blake2::Blake2bVar;
    use hex;
    use ink_storage::traits::{PackedLayout, SpreadAllocate, SpreadLayout, StorageLayout};
    use scale::{Decode, Encode};
    use scale_info::TypeInfo;

    fn hex_decode(s: &[u8]) -> [u8; 32] {
        let res = hex::decode(s).unwrap();

        let mut buffer = [0u8; 32];

        for i in (0..res.len()).rev() {
            buffer[32 - (res.len() - i)] = res[i];
        }

        buffer
    }

    fn hex_encode(h: &[u8]) -> String {
        hex::encode(h)
    }
    #[derive(
        StorageLayout,
        PackedLayout,
        SpreadLayout,
        SpreadAllocate,
        Debug,
        Default,
        TypeInfo,
        Encode,
        Decode,
        Clone,
    )]
    pub struct MultiChainAddrHash {
        digest: [u8; 32],
        address_str: String,
    }

    impl MultiChainAddrHash {
        pub fn make_hash(b: &[u8]) -> [u8; 32] {
            let mut hasher = Blake2bVar::new(32).unwrap();
            hasher.update(b);
            let mut buf = [0u8; 32];
            hasher.finalize_variable(&mut buf).unwrap();

            buf
        }

        pub fn from(addr: String) -> Self {
            let address_str = addr.clone();
            let digest = Self::make_hash(addr.as_bytes());

            MultiChainAddrHash {
                digest,
                address_str,
            }
        }

        pub fn to_string_digest(&self) -> String {
            let digest = self.digest;
            let digest_slice = digest.as_slice();

            hex_encode(digest_slice)
        }

        pub fn to_string(&self) -> String {
            self.address_str.clone()
        }
    }

    impl From<String> for MultiChainAddrHash {
        fn from(s: String) -> Self {
            assert!(
                s.len() > 34 && s.len() < 32,
                "Length must be between 32 and 34"
            );

            Self::from(s)
        }
    }

    impl From<&[u8]> for MultiChainAddrHash {
        fn from(b: &[u8]) -> Self {
            let multi_addr = core::str::from_utf8(b).unwrap();

            Self::from(multi_addr.to_string())
        }
    }

    #[derive(
        StorageLayout,
        Clone,
        Copy,
        SpreadLayout,
        PackedLayout,
        SpreadAllocate,
        Debug,
        Default,
        TypeInfo,
        Encode,
        Decode,
    )]
    pub struct U256([u8; 32]);

    impl U256 {
        pub fn from_hex(b: &[u8]) -> Self {
            let buffer = hex_decode(b);

            U256(buffer)
        }

        pub fn get(&self) -> &[u8; 32] {
            let U256(b) = self;

            b
        }

        pub fn from_decimal(s: String) -> Self {
            fn add(s1: String, s2: String) -> String {
                let (mut result, mut stmp) = ("".to_string(), "".to_string());
                let mut mark = 0;

                fn string_to_array_reversed(s: String) -> [u32; 256] {
                    let mut chars = s.chars();

                    let mut ret = [03u32; 256];

                    for i in 0..chars.clone().count() {
                        ret[i] = chars.next_back().unwrap().to_digit(10).unwrap();
                    }

                    ret
                }

                let smaller = if s1.len() > s2.len() {
                    s2.clone()
                } else {
                    s1.clone()
                };
                let larger = if s1.len() > s2.len() {
                    s1.clone()
                } else {
                    s2.clone()
                };

                let a1 = string_to_array_reversed(larger.clone());
                let a2 = string_to_array_reversed(smaller.clone());

                for i in 0..larger.len() {
                    let ctmp = a1[i] + (if i < smaller.len() { a2[i] } else { 0 }) + mark;

                    match ctmp {
                        0 => {
                            stmp = "0".to_string();
                            mark = 0;
                        }
                        1 => {
                            stmp = "1".to_string();
                            mark = 0;
                        }
                        2 => {
                            stmp = "0".to_string();
                            mark = 1;
                        }
                        3 => {
                            stmp = "1".to_string();
                            mark = 1;
                        }
                        _ => (),
                    }

                    let mut r = "".to_string();
                    r.push_str(stmp.as_str());
                    r.push_str(result.as_str());

                    result = r;
                }

                if mark > 0 {
                    let mut r = "1".to_string();
                    r.push_str(result.as_str());

                    result = r
                }

                result
            }

            fn digit_to_bin(c: char) -> &'static str {
                match c {
                    '0' => "0",
                    '1' => "1",
                    '2' => "10",
                    '3' => "11",
                    '4' => "100",
                    '5' => "101",
                    '6' => "110",
                    '7' => "111",
                    '8' => "1000",
                    '9' => "1001",
                    _ => "",
                }
            }

            let mut result = "".to_string();
            let mut s_chars = s.chars();

            for _ in 0..s.len() {
                let mut r1 = result.clone();
                let mut r2 = result.clone();

                r1.push_str("0");
                r2.push_str("000");

                result = add(r1, r2);
                result = add(result, digit_to_bin(s_chars.next().unwrap()).to_string());
            }

            let diff = 256 - result.len();

            let mut r = "".to_string();

            for _ in 0..diff {
                r.push_str("0");
            }

            r.push_str(result.as_str());

            result = r;

            let mut buffer = [0u8; 32];

            for i in (0..256).step_by(8) {
                let sub_string = &result[i..i + 8];

                let byte = u8::from_str_radix(sub_string, 2).unwrap();

                buffer[i / 8] = byte;
            }

            Self(buffer)
        }

        pub fn to_decimal(&self) -> String {
            let b = self.get();

            let mut digits = [0u16; 78];
            let mut length = 1;

            for j in 0..32 {
                let (mut i, mut carry) = (0usize, b[j] as u16);

                while i < length || carry != 0 {
                    let mut value = digits[i] * 256 + carry;

                    carry = value / 10;
                    value = value % 10;

                    digits[i] = value;

                    i += 1;
                }

                if i > length {
                    length = i;
                }
            }

            let mut number = "".to_string();
            println!("l {length}");
            for k in (0..length).rev() {
                number.push_str(digits[k].to_string().as_str());
            }

            number
        }

        pub fn to_hex(&self) -> String {
            let arr = self.get();

            hex_encode(arr as &[u8])
        }

        pub fn a_greater_than_b(a: &Self, b: &Self) -> bool {
            let a_arr = a.get();
            let b_arr = b.get();

            let mut score_a = 0u32;
            let mut score_b = 0u32;

            for i in 0usize..32usize {
                let a_least_significant = a_arr[i];
                let b_least_significant = b_arr[i];

                score_a += (a_least_significant > b_least_significant) as u32;
                score_b += (b_least_significant > a_least_significant) as u32;
            }

            score_a > score_b
        }

        pub fn add_a_with_b(a: &Self, b: &Self) -> Self {
            let a_arr = a.get();
            let b_arr = b.get();

            let mut addition_arr = [0u8; 32];
            let mut carry = 0u16;

            for i in 0usize..32usize {
                let a_least_significant = a_arr[i] as u16;
                let b_least_significant = b_arr[i] as u16;

                let apbpc = a_least_significant + b_least_significant + carry;

                match apbpc > u8::MAX as u16 {
                    true => {
                        addition_arr[i] = (apbpc - (u8::MAX as u16)) as u8;
                        carry = 1;
                    }
                    false => {
                        addition_arr[i] = apbpc as u8;
                        carry = 0;
                    }
                }
            }

            Self(addition_arr)
        }

        pub fn subtract_b_from_a(a: &Self, b: &Self) -> Self {
            let a_arr = a.get();
            let b_arr = b.get();

            let mut addition_arr = [0u8; 32];
            let mut carry = 0i16;

            for i in 0usize..32usize {
                let a_least_significant = a_arr[i] as i16;
                let b_least_significant = b_arr[i] as i16;

                let apbpc = a_least_significant - b_least_significant - carry;

                match apbpc < u8::MIN as i16 {
                    true => {
                        addition_arr[i] = ((u8::MAX as i16) + apbpc) as u8;
                        carry = 1;
                    }
                    false => {
                        addition_arr[i] = apbpc as u8;
                        carry = 0;
                    }
                }
            }

            Self(addition_arr)
        }

        pub fn new_zero() -> Self {
            let b = [0u8; 32];

            Self(b)
        }

        pub fn new_ff() -> Self {
            let b = [0xffu8; 32];

            Self(b)
        }
    }

    impl From<String> for U256 {
        fn from(s: String) -> Self {
            match s.chars().next().unwrap() {
                '0' => {
                    let b = s.as_bytes();
                    Self::from_hex(b)
                }
                _ => Self::from_decimal(s),
            }
        }
    }
}
