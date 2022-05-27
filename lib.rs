#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

mod utils {

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

#[ink::contract]
mod bridge_cherry_contract {

    use crate::utils::{MultiChainAddrHash, U256};

    #[ink(storage)]
    #[derive(ink_storage::traits::SpreadAllocate)]
    pub struct Bridge {
        owner: ink_env::AccountId,
        total_supply: U256,
        balances: ink_storage::Mapping<MultiChainAddrHash, U256>,
        allowances: ink_storage::Mapping<(MultiChainAddrHash, MultiChainAddrHash), U256>,
    }

    #[ink(event)]
    pub struct BridgeInComplex {
        #[ink(topic)]
        token_address: MultiChainAddrHash,
        #[ink(topic)]
        token_amount: U256,
        #[ink(topic)]
        recipient: MultiChainAddrHash,
        #[ink(topic)]
        from_chain: Option<U256>,
    }

    #[ink(event)]
    pub struct BridgeInSimple {
        token_address: String,
        token_amount: String,
        recipient: String,
        from_chain: Option<String>,
    }

    #[ink(event)]
    pub struct BridgeOutComplex {
        #[ink(topic)]
        token_address: MultiChainAddrHash,
        #[ink(topic)]
        token_amount: U256,
        #[ink(topic)]
        recipient: MultiChainAddrHash,
        #[ink(topic)]
        target_chain: Option<U256>,
    }

    #[ink(event)]
    pub struct BridgeOutSimple {
        token_address: String,
        token_amount: String,
        recipient: String,
        target_chain: Option<String>,
    }

    #[ink(event)]
    pub struct Initiate {
        initiated: bool,
        by: String,
        initial_balance: String,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: String,
        #[ink(topic)]
        spender: String,
        value_decimal: String,
        value_hex: String,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<String>,
        #[ink(topic)]
        to: Option<String>,
        value_decimal: String,
        value_hex: String,
    }

    #[derive(scale::Encode, scale::Decode, scale_info::TypeInfo)]
    pub enum BridgeContractError {
        ErrorApproving(String),
        ErrorTransferringFrom(String),
        ErrorTransferringTo(String),
        ErrorTransferringFromTo(String),
    }

    impl Bridge {
        #[ink(constructor)]
        pub fn new(initial_supply: U256) -> Self {
            ink_lang::utils::initialize_contract(|contract| {
                Self::new_init(contract, initial_supply)
            })
        }

        fn new_init(&mut self, initial_supply: U256) {
            let caller = self.env().caller();
            let caller_arr: &[u8] = caller.as_ref();

            let addr_multi: MultiChainAddrHash = caller_arr.into();

            self.balances.insert(&addr_multi, &initial_supply);
            self.total_supply = initial_supply;

            Self::env().emit_event(Initiate {
                initiated: true,
                by: addr_multi.to_string(),
                initial_balance: initial_supply.to_decimal(),
            });
        }

        #[ink(message)]
        pub fn get_balance_of(&self, owner: String) -> Option<U256> {
            let mcah: MultiChainAddrHash = owner.into();

            self.balances.get(mcah)
        }

        #[ink(message)]
        pub fn get_allowance_of(&self, owner: String, spender: String) -> Option<U256> {
            let mcah_owner: MultiChainAddrHash = owner.into();
            let mcah_spender: MultiChainAddrHash = spender.into();

            self.allowances.get((mcah_owner, mcah_spender))
        }

        fn transfer_from_to(
            &mut self,
            from: &MultiChainAddrHash,
            to: &MultiChainAddrHash,
            value: U256,
        ) -> Result<(), BridgeContractError> {
            let from_balance = self.get_balance_of(from.to_string()).unwrap();

            if U256::a_greater_than_b(&value, &from_balance) {
                return Err(BridgeContractError::ErrorTransferringFromTo(
                    "Not enough funds".to_string(),
                ));
            }

            let sub_from = U256::subtract_b_from_a(&from_balance, &value);

            self.balances.insert(from, &sub_from);

            let to_balance = self.get_balance_of(to.to_string()).unwrap();

            let sub_to = U256::subtract_b_from_a(&to_balance, &value);

            self.balances.insert(to, &sub_to);

            Self::env().emit_event(Transfer {
                from: Some(from.to_string()),
                to: Some(to.to_string()),
                value_decimal: value.to_decimal(),
                value_hex: value.to_hex(),
            });

            Ok(())
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: MultiChainAddrHash, value: U256) {
            let owner = self.env().caller();

            let caller_arr: &[u8] = owner.as_ref();

            let owner: MultiChainAddrHash = caller_arr.into();

            self.allowances.insert((&owner, &spender), &value);
            self.env().emit_event(Approval {
                owner: owner.to_string(),
                spender: spender.to_string(),
                value_decimal: value.to_decimal(),
                value_hex: value.to_hex(),
            });
        }

        #[ink(message)]
        pub fn bridge(
            &mut self,
            token_address: MultiChainAddrHash,
            token_amount: U256,
            recipient: MultiChainAddrHash,
            from_chain: U256,
        ) -> Result<(), BridgeContractError> {
            let res = self.transfer_from_to(&token_address, &recipient, token_amount);

            Self::env().emit_event(BridgeInComplex {
                token_address,
                token_amount,
                recipient,
                from_chain: Some(from_chain),
            });

            res
        }

        #[ink(message)]
        pub fn bridge_string(
            &mut self,
            token_address_str: String,
            token_amount_str: String,
            recipient_str: String,
            from_chain_str: String,
            emit_simple: bool,
        ) -> Result<(), BridgeContractError> {
            let token_address = token_address_str.clone().into();
            let token_amount = token_amount_str.clone().into();
            let recipient = recipient_str.clone().into();
            let from_chain = from_chain_str.clone().into();

            let res = self.bridge(token_address, token_amount, recipient, from_chain);

            if emit_simple {
                Self::env().emit_event(BridgeInSimple {
                    token_address: token_address_str,
                    token_amount: token_amount_str,
                    recipient: recipient_str,
                    from_chain: Some(from_chain_str),
                });
            }

            res
        }

        #[ink(message)]
        pub fn bridge_in(
            &mut self,
            token_amount: U256,
            recipient: MultiChainAddrHash,
            from_chain: U256,
        ) -> Result<(), BridgeContractError> {
            let caller = self.env().caller();
            let token_address: MultiChainAddrHash = (caller.as_ref() as &[u8]).into();

            let res = self.transfer_from_to(&token_address, &recipient, token_amount);

            Self::env().emit_event(BridgeInComplex {
                token_address,
                token_amount,
                recipient,
                from_chain: Some(from_chain),
            });

            res
        }

        #[ink(message)]
        pub fn bridge_in_string(
            &mut self,
            token_amount_str: String,
            recipient_str: String,
            from_chain_str: String,
            emit_simple: bool,
        ) -> Result<(), BridgeContractError> {
            let token_amount = token_amount_str.clone().into();
            let recipient: MultiChainAddrHash = recipient_str.clone().into();
            let from_chain = from_chain_str.clone().into();

            let res = self.bridge_in(token_amount, recipient, from_chain);

            let caller = self.env().caller();
            let token_address: MultiChainAddrHash = (caller.as_ref() as &[u8]).into();

            if emit_simple {
                Self::env().emit_event(BridgeInSimple {
                    token_address: token_address.to_string(),
                    token_amount: token_amount_str,
                    recipient: recipient_str,
                    from_chain: Some(from_chain_str),
                });
            }

            res
        }

        #[ink(message)]
        pub fn bridge_out(
            &mut self,
            token_address: MultiChainAddrHash,
            token_amount: U256,
            target_chain: U256,
        ) -> Result<(), BridgeContractError> {
            let caller = self.env().caller();
            let recipient: MultiChainAddrHash = (caller.as_ref() as &[u8]).into();

            let res = self.transfer_from_to(&token_address, &recipient, token_amount);

            Self::env().emit_event(BridgeOutComplex {
                token_address,
                token_amount,
                recipient,
                target_chain: Some(target_chain),
            });

            res
        }

        #[ink(message)]
        pub fn bridge_out_string(
            &mut self,
            token_address_str: String,
            token_amount_str: String,
            target_chain_str: String,
            emit_simple: bool,
        ) -> Result<(), BridgeContractError> {
            let token_address = token_address_str.clone().into();
            let token_amount = token_amount_str.clone().into();
            let target_chain = target_chain_str.clone().into();

            let res = self.bridge_out(token_address, token_amount, target_chain);

            let caller = self.env().caller();
            let recipient: MultiChainAddrHash = (caller.as_ref() as &[u8]).into();

            if emit_simple {
                Self::env().emit_event(BridgeOutSimple {
                    token_address: token_address_str,
                    token_amount: token_amount_str,
                    recipient: recipient.to_string(),
                    target_chain: Some(target_chain_str),
                });
            }

            res
        }

        #[ink(message)]
        pub fn set_total_supply(&mut self, total_supply: U256) {
            assert!(
                self.env().caller_is_origin() && self.env().caller() == self.owner,
                "Call does not originate from origin OR not from the owner"
            );

            self.total_supply = total_supply;
        }
    }
}
