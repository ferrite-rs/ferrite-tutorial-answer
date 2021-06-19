use ferrite_session::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

type Pin = u32;
type CashAmount = u64;

const MAX_RETRY: u8 = 3;

define_choice! { PinResult;
  PinOk:
    ReceiveValue<CashAmount, InternalChoice<WithdrawResult>>,
  WrongPin: Z,
  MaxRetry: End,
}

define_choice! { WithdrawResult;
  WithdrawOk: SendChannel<Cash, End>,
  InsufficientFund: End,
}

type Atm = Rec<ReceiveValue<Pin, InternalChoice<PinResult>>>;

pub struct CashVal {
  amount: CashAmount,
}
type Cash = SendValue<CashVal, End>;

fn forge_cash(amount: u64) -> Session<Cash> {
  send_value(CashVal { amount }, terminate())
}

fn atm_provider(
  actual_pin: Pin,
  attempts: u8,
  balance: u64,
) -> Session<Atm> {
  // todo!("Implement a calculator provider here");
  fix_session(receive_value(move |given_pin| {
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
    } else if attempts >= MAX_RETRY {
      println!("[Provider] Maximum attempts exceeded. Denying withdrawal access");
      offer_case!(MaxRetry, terminate())
    } else {
      println!("[Provider] Provided pin is incorrect. Remaining attempts: {}", MAX_RETRY - attempts);
      step(async move {
        sleep(Duration::from_secs(1)).await;
        offer_case!(
          WrongPin,
          atm_provider(actual_pin, attempts + 1, balance)
        )
      })
    }
  }))
}

fn atm_client(
  pin: Pin,
  withdraw_amount: CashAmount,
) -> Session<ReceiveChannel<Atm, End>> {
  receive_channel(move |atm| {
    println!("[Client] Trying to withdraw from ATM with pin {}", pin);
    unfix_session(
      atm,
      send_value_to(
        atm,
        pin,
        case! { atm;
          PinOk => {
            send_value_to(atm, withdraw_amount,
              case! { atm;
                WithdrawOk => {
                  receive_channel_from(atm, move |cash| {
                    receive_value_from(cash, move |cash_val: CashVal| {
                      println!("Receive cash amount {} from ATM.", cash_val.amount);
                      wait_all!([atm, cash], terminate())
                    })
                  })
                }
                InsufficientFund => {
                  println!("[Client] Failed to withdraw from ATM: Account has insufficient fund.");
                  wait(atm, terminate())
                }
              })
          }
          WrongPin => {
            println!("[Client] Failed with incorrect pin. Trying again with pin {}", pin+1);
            include_session(atm_client(pin+1, withdraw_amount), move |client| {
              send_channel_to(client, atm,
                forward(client))
            })
          }
          MaxRetry => {
            println!("[Client] Failed to withdraw from ATM: Maximum retry attempted.");
            wait(atm, terminate())
          }
        },
      ),
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
    atm_provider(actual_pin, 0, balance),
  ))
  .await
}

#[tokio::main]
pub async fn main() {
  env_logger::init();

  run_atm_session(1024, 1022, 1000, 900).await;

  sleep(Duration::from_secs(2)).await;
  run_atm_session(1024, 1010, 1000, 900).await;

  sleep(Duration::from_secs(2)).await;
  run_atm_session(1024, 1023, 1000, 2000).await;
}
