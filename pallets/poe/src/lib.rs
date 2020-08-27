#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use frame_support::{
    decl_module, decl_storage, decl_event, decl_error,
    dispatch, ensure,
};
use frame_system::ensure_signed;
// 引入 Vec 依赖(" sp_std::prelude::* "仍需配置于" poe/Cargo.toml ")
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as PoeModule {
	    // AccountId 表示存证人;BlockNumber 表示存证时间
	    // 使用到 Vec (源于" sp_std::prelude::* "[还需配置于 Cargo.toml])：需导入依赖
		Proofs get(fn proofs):map hasher(blake2_128_concat) Vec<u8> => (T::AccountId, T::BlockNumber);
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
	    ClaimCreated(AccountId, Vec<u8>),
	    ClaimRevoked(AccountId, Vec<u8>),
	    ClaimTransfer(AccountId, Vec<u8>, AccountId),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
	    ProofAlreadyExist,
	    ProofNotExist,
	    NotClaimOwner,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

        /* 创建存证:
              origin : 交易的发送方(类型为" T::Origin "[源于 system 模块])
              claim ：存证的哈希值(存储模块定义 Proofs 的 key 即 Vec<u8> )
              返回结果：DispatchResult (其内 Err 为自定义的 err )
              #[weigt = 0] : 交易权重(表示交易的时间)
         */
        #[weight = 0]
        pub fn create_claim(origin, claim: Vec<u8>) -> dispatch::DispatchResult{
            // (添加业务逻辑)校验并获取交易用户即 AccountId
            let sender = ensure_signed(origin)?;
            // 确保存储数据从未被存储(即未曾被申领过)：Proofs 及 Error 需被泛型 T 约束
            // ensure! 宏需要在" use  frame_support::{ //... } "内引入
            // 错误信息" ProofAlreadyExist "需要定义于 decl_error! 模块
            ensure!(!Proofs::<T>::contains_key(&claim),Error::<T>::ProofAlreadyExist);
            // 存储(针对 map : 键为 claim 值为两元素组成的元组(AccountId,blockNumber))
            Proofs::<T>::insert(&claim,(sender.clone(),frame_system::Module::<T>::block_number()));
            // 触发的事件: 其名称为 ClaimCreated (源于 decl_event! )
            Self::deposit_event(RawEvent::ClaimCreated(sender,claim));
            // 交易发送成功
            Ok(())
        }

        // 撤销存证
        #[weight = 0]
        pub fn revoke_claim(origin, claim: Vec<u8>) -> dispatch::DispatchResult{
            // 校验并获取交易用户即 AccountId
            let sender = ensure_signed(origin)?;
            // 确保存证存在
            ensure!(Proofs::<T>::contains_key(&claim),Error::<T>::ProofNotExist);
            /* 获取当前 claim 对应的" owner (应用人)"及" _block_number (区块数)"
               区块数" _block_numer "由于未曾使用故添加" _ "前缀避开警告
             */
            let (owner,_block_number) = Proofs::<T>::get(&claim);
            // 校验发送方是否为 owner
            ensure!(owner == sender, Error::<T>::NotClaimOwner);
            // 删除
            Proofs::<T>::remove(&claim);
            // 触发事件
            Self::deposit_event(RawEvent::ClaimRevoked(sender,claim));
            // 交易发送成功
            Ok(())
        }

        // 转移存证
        #[weight = 0]
        pub fn transfer_claim(
                origin,
                proof: Vec<u8>,
                receiver:<T as frame_system::Trait>::AccountId
            )
            -> dispatch::DispatchResult
            {
                // 校验并获取交易用户即 AccountId
                let sender = ensure_signed(origin)?;
                // 校验并获取转移目标用户 AccountId
                //let receiver = ensure_signed(receiver)?;
                // 确保存证存在
                ensure!(Proofs::<T>::contains_key(&proof),Error::<T>::ProofNotExist);
                // 获取当前 claim 对应的 owner (应用人)
                let (owner,_) = Proofs::<T>::get(&proof);
                // 校验发送方是否为 owner
                ensure!(owner == sender, Error::<T>::NotClaimOwner);
                // 转移 TODO
                // Proofs::<T>::mutate(key,||)

                // 触发事件
                Self::deposit_event(RawEvent::ClaimTransfer(sender,proof,receiver));
                // 交易发送成功
                Ok(())
        }

	}
}
