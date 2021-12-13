//	A reusbale membership pallet with a rating system


#![cfg_attr(not(feature = "std"), no_std)]
use codec::Codec;

use frame_support::{pallet_prelude::{Member, MaybeSerializeDeserialize}, RuntimeDebug, print, ensure};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
//	Substrate MarketPlace including Buyer and Seller Reputation
//	Uses: Ride Sharing rider and Driver profile 
pub use pallet::*;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
use codec::{Encode, Decode};
use sp_runtime::{ArithmeticError, DispatchError};



#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::{DispatchResult}, pallet_prelude::*, traits::UnixTime, Blake2_128Concat};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Moment: UnixTime;

	}
	type AccountId<T> = <T as frame_system::Config>::AccountId;

	#[pallet::pallet]
 	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::storage]
	#[pallet::getter(fn members_score)]
	pub type MemberScore<T> = StorageMap<
		_,
		Blake2_128Concat, 
		AccountId<T>, 
		(Reputation_Score, u32), 
		ValueQuery
		>;

	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T> = StorageValue<_, Vec<AccountId<T>>, ValueQuery>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AddedRating { 
			rater: T::AccountId, 
			owner: T::AccountId, 
		},
		AddedMember { 
			new_member: T::AccountId, 
		},
		RemovedMember { 
			member: T::AccountId, 
		}
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NotFoundSeller,
		NotFound,
		NotFoundBuyer,
		NotMember, 
		AlreadyMember

	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn add_member(
			origin: OriginFor<T>,
			new_member: T::AccountId
		) -> DispatchResult { 
			let new_member = ensure_signed(origin)?;
			let mut member_list = Members::<T>::get();
			
			let location = member_list.binary_search(&new_member).err().ok_or(Error::<T>::AlreadyMember)?;
			member_list.insert(location, new_member.clone());
			Members::<T>::put(&member_list);
			Self::deposit_event(Event::AddedMember { 
				new_member
			});

			Ok(())
		}
		#[pallet::weight(10_000)]
		pub fn remove_member(
			origin: OriginFor<T>,
			curr_member: T::AccountId
		) -> DispatchResult { 
			let curr_member = ensure_signed(origin)?;

			let mut member_list = Members::<T>::get();
			let location = member_list.binary_search(&curr_member).ok().ok_or(Error::<T>::NotFound)?;
			member_list.remove(location);
			Members::<T>::put(&member_list);
			
			Self::deposit_event(Event::RemovedMember { 
				member: curr_member
			});
			Ok(())
		}
	} 
	impl<T: Config> Pallet<T> { 

	}
}
//	Reputation Trait 
pub trait Reputation<AccountId, Moment> { 
	//	The reputation of the vendor 
	type Reputation_Score;
	type Feedback: Member + Codec + MaybeSerializeDeserialize;
	//	Allow an account to give ratings to the vendor 
	fn rate_seller(
		buyer: AccountId,
		seller: AccountId, 
		review: Self::Feedback, 
		now: Moment

	) -> Result<u32, DispatchError> ;
	//	Assigns a rating to another person 
	fn rate_buyer(
		buyer: AccountId,
		seller: AccountId, 
		review: Self::Feedback,
		now: Moment

	) -> Result<u32, DispatchError> ;
	//	The current ratings of an account 
	fn reputation(sender: AccountId) -> Self::Reputation_Score;
}
//	Out of 5 
pub type Reputation_Score = u32;
#[derive(Encode, Clone, Decode, PartialEq, RuntimeDebug, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Feedback { 
	Terrible,
	Bad,
	Okay,
	Good,
	Great,
}
impl Feedback { 
	fn rating(&self) -> u32 { 
		match self { 
			Feedback::Terrible => 1,
			Feedback::Bad => 2,
			Feedback::Okay => 3,
			Feedback::Good => 4,
			Feedback::Great => 5
		}
	}
}
//	Implementing out of 5 ratings 
impl<T: Config> Reputation<T::AccountId, T::Moment> for Pallet<T> { 
	type Reputation_Score = Reputation_Score;
	type Feedback = Feedback;

	fn rate_seller(
		buyer: T::AccountId, 
		seller: T::AccountId, 
		review: Feedback,
		now: T::Moment
	) -> Result<u32, DispatchError> { 
		let rating = review.rating();
		//	Insert in Rating in to seller's account and then update an the average  
		let members = Self::members();
		match !members.contains(&seller) { 
			true => return Err(Error::<T>::NotFoundSeller.into()),
			false => { 
				MemberScore::<T>::mutate(&seller, |(score, count)| -> Result<(u32, u32), DispatchError> { 
					let curr = *score;
					let cccount = *count;
					//	Get the average and insert into storage 
					*score = score.checked_add(rating).ok_or(ArithmeticError::Overflow)?;
					*count = count.checked_add(1).ok_or(ArithmeticError::Overflow)?;
					*score = score.checked_div(*count).ok_or(ArithmeticError::DivisionByZero)?;
					
					Ok((curr, cccount))
				});
			}
		}
		Ok(rating)
	}
	fn rate_buyer(
		buyer: T::AccountId, 
		seller: T::AccountId, 
		review: Feedback,
		now: T::Moment
	) -> Result<u32, DispatchError>  { 

		let rating = review.rating();
		//	Insert in Rating in to seller's account and then update an the average  
		let members = Self::members();
		match !members.contains(&buyer) { 
			true => return Err(Error::<T>::NotFoundBuyer.into()),
			false => { 
				MemberScore::<T>::mutate(&buyer, |(score, count)| -> Result<(u32, u32), DispatchError> { 
					let curr = *score;
					let cccount = *count;
					//	Get the average and insert into storage 
					*score = score.checked_add(rating).ok_or(ArithmeticError::Overflow)?;
					*count = count.checked_add(1).ok_or(ArithmeticError::Overflow)?;
					*score = score.checked_div(*count).ok_or(ArithmeticError::DivisionByZero)?;
					
					Ok((curr, cccount))
				});
			}
		}
		Ok(rating)
	}
	fn reputation(sender: T::AccountId) -> Self::Reputation_Score {
		let members = Self::members();
		
		let score = MemberScore::<T>::get(&sender);
		//	return the average score 
		score.0
	}
}

