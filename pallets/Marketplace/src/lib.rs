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
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::UnixTime, Blake2_128Concat};
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
		T::AccountId, 
		(Reputation_Score, u32), 
		ValueQuery
		>;

	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AddedRating { 
			rater: T::AccountId, 
			owner: T::AccountId, 
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NotFoundSeller,
		NotFound,
		NotFoundBuyer

	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		//	Add remove members 
		//	Remove Members 
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

	) -> Result<(), Error<()>>;
	//	Assigns a rating to another person 
	fn rate_buyer(
		buyer: AccountId,
		seller: AccountId, 
		review: Self::Feedback,
		now: Moment

	) -> Result<(), Error<()>>;
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
	) -> Result<(), Error<()>> { 

		let rating = review.rating();
		//	Insert in Rating in to seller's account and then update an the average  
		let members = Self::members();
		match !members.contains(&seller) { 
			true => return Err(Error::<T>::NotFoundSeller.into()),
			false => { 
				MemberScore::<T>::mutate(&seller, |(score, count)| -> Result<i32, DispatchError> { 
					let curr = *score;
					let count = *count;
					//	Get the average and insert into storage 
					*score = score.checked_add(rating).ok_or(ArithmeticError::Overflow)?;
					*count = count.checked_add(1).ok_or(ArithmeticError::Overflow)?;
					*score = score.checked_div(count).ok_or(ArithmeticError::DivisionByZero)?;
					
					Ok(curr)
				})
			}
		}

		Ok(())
	}
	fn rate_buyer(
		buyer: T::AccountId, 
		seller: T::AccountId, 
		review: Feedback,
		now: T::Moment
	) -> Result<(), Error<()>> { 

		let rating = review.rating();
		//	Insert in Rating in to seller's account and then update an the average  
		let members = Self::members();
		match !members.contains(&buyer) { 
			true => return Err(Error::<T>::NotFoundBuyer.into()),
			false => { 
				MemberScore::<T>::mutate(&buyer, |(score, count)| -> Result<i32, DispatchError> { 
					let curr = *score;
					let count = *count;
					//	Get the average and insert into storage 
					*score = score.checked_add(rating).ok_or(ArithmeticError::Overflow)?;
					*count = count.checked_add(1).ok_or(ArithmeticError::Overflow)?;
					*score = score.checked_div(count).ok_or(ArithmeticError::DivisionByZero)?;
					
					Ok(curr)
				})
			}
		}

		Ok(())
	}
	fn reputation(sender: T::AccountId) -> Self::Reputation_Score {
		let members = Self::members();
		ensure!(members.contain(&sender), Error::<T>::NotFound);
		let score = MemberScore::<T>::try_get(&sender);
		//	return the average score 
		score.0
	}
}

