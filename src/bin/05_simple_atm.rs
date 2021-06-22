use ferrite_session::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

/*
  # Excercise 5: Simple ATM

  - Implement an `atm_provider` that dispense cash if the client provides
    the correct pin and the requested amount is lower than available balance.

  - Implement an `atm_client` that attempts to withdraw a given amount of
    cash from the ATM with the given pin.

  - Print the following messages when the provider is in the following steps:

    - Received pin value:
      "[Provider] Authenticating ATM withdrawal with given pin {pin}"

    - Given pin matches the actual pin:
      "[Provider] Provided pin is correct. Asking for withdrawal amount""

    - Incorrect pin provided:
      "[Provider] Incorrect pin provided, terminating"

    - Withdraw amount <= account balance:
      "[Provider] Withdrawing ${withdraw_amount} from account. Remaining balance: ${balance}"

    - Withdraw amount > account balance:
      "[Provider] Insufficient fund: requested amount ${} is more than available balance ${}"

  - Print the following messages when the client is in the following steps:

    - Withdraw successful and cash spent
      "[Client] Spent cash of amount ${amount}"

    - Withdraw failed with incorrect pin:
      "[Client] Failed to withdraw from ATM: Wrong pin"

    - Withdraw failed with insufficient funds:
      "[Client] Failed to withdraw from ATM: Account has insufficient fund"

  The given main program will try to run your program with different
  parameters.

  After completing your solution, you should get the following result
  running the program:

  ```
  $ cargo run --bin 05_simple_atm
  *** Running new ATM session with actual pin: 1024, try pin: 1024, balance: 1000, withdrawal amount: 900 ***
  [Provider] Authenticating ATM withdrawal with given pin 1024
  [Provider] Provided pin is correct. Asking for withdrawal amount
  [Provider] Withdrawing $900 from account. Remaining balance: $100
  [Client] Spent cash of amount $900

  *** Running new ATM session with actual pin: 1024, try pin: 1010, balance: 1000, withdrawal amount: 900 ***
  [Provider] Authenticating ATM withdrawal with given pin 1010
  [Client] Failed to withdraw from ATM: Wrong pin

  *** Running new ATM session with actual pin: 1024, try pin: 1024, balance: 1000, withdrawal amount: 2000 ***
  [Provider] Authenticating ATM withdrawal with given pin 1024
  [Provider] Provided pin is correct. Asking for withdrawal amount
  [Provider] Insufficient fund: requested amount $2000 is more than available balance $1000
  [Client] Failed to withdraw from ATM: Account has insufficient fund
  ```
*/

type Pin = u32;
type CashAmount = u64;

type Atm = ReceiveValue<Pin, InternalChoice<PinResult>>;

define_choice! { PinResult;
  PinOk:
    ReceiveValue<CashAmount, InternalChoice<WithdrawResult>>,
  WrongPin: End,
}

define_choice! { WithdrawResult;
  WithdrawOk: SendChannel<Cash, End>,
  InsufficientFund: End,
}

// Mock cash as linear resource
pub struct CashVal {
  amount: CashAmount,
}
type Cash = SendValue<CashVal, End>;

// Helper for the ATM to create cash. Do not use this on the client!
fn forge_cash(amount: u64) -> Session<Cash> {
  send_value(CashVal { amount }, terminate())
}

// A helper to spend the linear cash channel. You can use this like:
//  include_session(spend_cash(), move |spend| {
//    send_channel_to(spend, cash, ...) })
fn spend_cash() -> Session<ReceiveChannel<Cash, End>> {
  receive_channel(move |cash| {
    receive_value_from(cash, move |cash_val: CashVal| {
      println!("[Client] Spent cash of amount ${}", cash_val.amount);
      wait(cash, terminate())
    })
  })
}

fn atm_provider(actual_pin: Pin, balance: u64) -> Session<Atm> {
  // todo!("Implement ATM provider here");
  receive_value(move |given_pin| {
    println!(
      "[Provider] Authenticating ATM withdrawal with given pin {}",
      given_pin
    );

    if given_pin == actual_pin {
      println!("[Provider] Provided pin is correct. Asking for withdrawal amount");
      offer_case!(
        PinOk,
        receive_value(move |amount| {
          if amount <= balance {
            println!("[Provider] Withdrawing ${} from account. Remaining balance: ${}", amount, balance - amount);
            offer_case!(
              WithdrawOk,
              include_session(forge_cash(amount), move |cash| {
                send_channel_from(cash, terminate())
              })
            )
          } else {
            println!("[Provider] Insufficient fund: requested amount ${} is more than available balance ${}", amount, balance);
            offer_case!(InsufficientFund, terminate())
          }
        })
      )
    } else {
      println!("[Provider] Incorrect pin provided, terminating");
      offer_case!(WrongPin, terminate())
    }
  })
}

fn atm_client(
  pin: Pin,
  withdraw_amount: CashAmount,
) -> Session<ReceiveChannel<Atm, End>> {
  // todo!("Implement ATM client here");
  receive_channel(move |atm| {
    send_value_to(
      atm,
      pin,
      case! { atm;
        PinOk => {
          send_value_to(atm, withdraw_amount,
            case! { atm;
              WithdrawOk => {
                receive_channel_from(atm, move |cash| {
                  include_session(spend_cash(), move |spend| {
                    send_channel_to(spend, cash,
                      wait_all!([atm, spend], terminate()))
                  })
                })
              }
              InsufficientFund => {
                println!("[Client] Failed to withdraw from ATM: Account has insufficient fund");
                wait(atm, terminate())
              }
            })
        }
        WrongPin => {
          println!("[Client] Failed to withdraw from ATM: Wrong pin");
          wait(atm, terminate())
        }
      },
    )
  })
}

async fn run_atm_session(
  actual_pin: Pin,
  try_pin: Pin,
  balance: CashAmount,
  withdraw_amount: CashAmount,
) {
  println!("*** Running new ATM session with actual pin: {}, try pin: {}, balance: {}, withdrawal amount: {} ***",
    actual_pin, try_pin, balance, withdraw_amount);

  run_session(apply_channel(
    atm_client(try_pin, withdraw_amount),
    atm_provider(actual_pin, balance),
  ))
  .await
}

#[tokio::main]
pub async fn main() {
  env_logger::init();

  run_atm_session(1024, 1024, 1000, 900).await;

  sleep(Duration::from_secs(2)).await;
  run_atm_session(1024, 1010, 1000, 900).await;

  sleep(Duration::from_secs(2)).await;
  run_atm_session(1024, 1024, 1000, 2000).await;
}
