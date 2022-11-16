use crate::{mock::*};
use frame_support::{assert_err, assert_ok};


#[test]
fn error_works(){
 new_test_ext().execute_with(|| {
   assert_err!(
     Something::add_value(RuntimeOrigin::signed(1), 51),
     "value must be <= maximum add amount constant"
   );
 })
}

#[test]
fn test_should_work() {
 new_test_ext().execute_with(|| {
   assert_ok!(
     Something::add_value(RuntimeOrigin::signed(1), 10)
   );
 })
}

#[test]
fn test_should_fail() {
 new_test_ext().execute_with(|| {
   assert_ok!(
     Something::add_value(RuntimeOrigin::signed(1), 100)
   );
 })
}