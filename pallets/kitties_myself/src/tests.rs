use crate::{mock::*, Error, *};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_mint_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1); // resolve error https://substrate.stackexchange.com/questions/4511/test-panic-with-randomnessrandom-attempt-to-subtract-with-overflow
		assert_eq!(MaxKittyOwned::get(), 1);

		// check mint kitty success
		assert_eq!(<KittiesTotal<Test>>::get(), 0);
		assert_ok!(KittiesMyself::mint_kitty(RuntimeOrigin::signed(1), 100));
		assert_eq!(<KittiesTotal<Test>>::get(), 1);

		// check owned kitty
		let kitties_owned = <KittiesOwned<Test>>::get(1);
		assert_eq!(kitties_owned.len(), 1);

		// check kitty exists
		let kitty_dna = *kitties_owned.last().unwrap();
		let kitty = <Kitties<Test>>::get(kitty_dna).unwrap();
		assert_eq!(kitty.price, 100);
		assert_eq!(kitty.owner, 1);
	})
}

#[test]
fn test_transfer_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(KittiesMyself::mint_kitty(RuntimeOrigin::signed(1), 100));
		let kitties_owned = <KittiesOwned<Test>>::get(1);
		assert_eq!(kitties_owned.len(), 1);
        let kitty_dna = *kitties_owned.last().unwrap();

		assert_ok!(KittiesMyself::transfer(RuntimeOrigin::signed(1), 2, kitty_dna));

		let kitties_owned_1 = <KittiesOwned<Test>>::get(1);
		assert_eq!(kitties_owned_1.len(), 0);
		let kitties_owned_2 = <KittiesOwned<Test>>::get(2);
		assert_eq!(kitties_owned_2.len(), 1);

		// check kitty change owner
		let kitty_dna = *kitties_owned_2.last().unwrap();
		let kitty = <Kitties<Test>>::get(kitty_dna).unwrap();
		assert_eq!(kitty.owner, 2);
	})
}

#[test]
fn test_transfer_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(KittiesMyself::mint_kitty(RuntimeOrigin::signed(1), 100));
		let kitties_owned = <KittiesOwned<Test>>::get(1);
		assert_eq!(kitties_owned.len(), 1);
		let kitty_dna = *kitties_owned.last().unwrap();

		assert_noop!(
			KittiesMyself::transfer(RuntimeOrigin::signed(1), 1, kitty_dna),
			Error::<Test>::NotTransferToSelf
		);

		assert_noop!(
			KittiesMyself::transfer(RuntimeOrigin::signed(2), 1, kitty_dna),
			Error::<Test>::NotOwner
		);
	})
}
