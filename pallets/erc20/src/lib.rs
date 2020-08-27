/*
0. 编辑 pallet

 */
#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use codec::{Codec, Encode, Decode};
use frame_support::Parameter;
use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch::DispatchResult, ensure};
use frame_system::{self as system, ensure_signed};
use sp_runtime::traits::{CheckedAdd, CheckedSub, Member, AtLeast32BitUnsigned};

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type TokenBalance: CheckedAdd + CheckedSub + Parameter + Codec + Member + Default + Copy + AtLeast32BitUnsigned;
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct Erc20Token<U> {
    name: Vec<u8>,
    ticker: Vec<u8>,
    total_supply: U,
}

decl_storage! {
	trait Store for Module<T: Trait> as Erc20 {
	    TokenId get(fn token_id): u32;
		Tokens get(fn token_details): map hasher(blake2_128_concat) u32 => Erc20Token<T::TokenBalance>;
		BalanceOf get(fn balance_of): map hasher(blake2_128_concat) (u32, T::AccountId) => T::TokenBalance;
		// 授权
		Allowance get(fn allowance): map hasher(blake2_128_concat) (u32, T::AccountId, T::AccountId) => T::TokenBalance;
	}
}

decl_event!(
	/*pub enum Event<T> where Aid = <T as system::Trait>::AccountId, Balance = <T as Trait>::TokenBalance {
		Transfer(u32, Aid, Aid, Balance),
		// 更新 allowance 时触发的事件
		Approval(u32, Aid, Aid, Balance),
	}*/
	pub enum Event<T> where <T as system::Trait>::AccountId, <T as Trait>::TokenBalance {
		Transfer(u32, AccountId, AccountId, TokenBalance),
		// 更新 allowance 时触发的事件
		Approval(u32, AccountId, AccountId, TokenBalance),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		StorageOverflow,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;
		#[weight = 0]
		fn init(origin, name:Vec<u8>, ticker:Vec<u8>, total_supply: T::TokenBalance) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			// 校验长度
			ensure!(name.len() <= 64, "token name cannot exceed 64 bytes");
			ensure!(ticker.len() <= 32, "token ticker cannot exceed 32 bytes");
            // 获取 token_id
            let token_id = Self::token_id();
            let next_token_id = token_id.checked_add(1).ok_or("overflow in calculating next token id")?;
            <TokenId>::put(next_token_id);
			// 设置 token
			let token = Erc20Token {
				name,
				ticker,
				total_supply,
			};
			// 颁发 token
			<Tokens<T>>::insert(token_id,token);
			<BalanceOf<T>>::insert((token_id, sender), total_supply);
			Ok(())
		}
		// 转账
		#[weight = 0]
		fn transfer(_origin, token_id: u32,  to: T::AccountId, value: T::TokenBalance) -> DispatchResult {
			let sender = ensure_signed(_origin)?;
			Self::_transfer(token_id, sender, to, value)
		}
		#[weight = 0]
		fn transfer_from(_origin, token_id: u32, from: T::AccountId, to: T::AccountId, value: T::TokenBalance) -> DispatchResult {
			let allowance = Self::allowance((token_id, from.clone(), to.clone()));
			ensure!(allowance >= value, "Not enough allowance.");
			let updated_allowance = allowance.checked_sub(&value).ok_or(Error::<T>::StorageOverflow)?;
			<Allowance<T>>::insert((token_id, from.clone(), to.clone()), updated_allowance);
			Self::deposit_event(RawEvent::Approval(token_id, from.clone(), to.clone(), value));
			Self::_transfer(token_id, from, to, value)
		}
		// 将一定数量的 token　授权给另一个账户
		#[weight = 0]
		fn approval(_origin, token_id: u32, spender: T::AccountId, value: T::TokenBalance) -> DispatchResult {
			let sender = ensure_signed(_origin)?;
			let allowance = Self::allowance((token_id, sender.clone(), spender.clone()));
			let updated_allowance = allowance + value;
			<Allowance<T>>::insert((token_id, sender.clone(), spender.clone()), updated_allowance);
			Self::deposit_event(RawEvent::Approval(token_id, sender.clone(), spender.clone(), value));
			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
    fn _transfer(
        token_id: u32,
        from: T::AccountId,
        to: T::AccountId,
        value: T::TokenBalance,
    ) -> DispatchResult {
        let sender_balance = Self::balance_of((token_id, from.clone()));
        ensure!(sender_balance >= value, "Not enough allowance.");
        let updated_from_balance = sender_balance.checked_sub(&value).ok_or(Error::<T>::StorageOverflow)?;
        let receiver_balance = Self::balance_of((token_id, to.clone()));
        let updated_to_balance = receiver_balance.checked_add(&value).ok_or(Error::<T>::StorageOverflow)?;
        <BalanceOf<T>>::insert((token_id, from.clone()), updated_from_balance);
        <BalanceOf<T>>::insert((token_id, to.clone()), updated_to_balance);
        Self::deposit_event(RawEvent::Transfer(token_id, from, to, value));
        Ok(())
    }
}