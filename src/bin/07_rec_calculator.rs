use ferrite_session::prelude::*;

define_choice! {DivResult;
  DivOk:
    Z,
  DivZero:
    End
}

define_choice! { CalculatorOps;
  Add:
    ReceiveValue<f64, Z>,
  Mult:
    ReceiveValue<f64, Z>,
  Div:
    ReceiveValue<f64,
      InternalChoice<DivResult>>,
  Done:
    SendValue<f64, End>,
}

type Calculator = Rec<ExternalChoice<CalculatorOps>>;

fn calculator_provider(current: f64) -> Session<Calculator> {
  todo!("Implement a calculator provider here");
  // fix_session(offer_choice! {
  //   Add => {
  //     receive_value(move |val| {
  //       calculator_provider(current + val)
  //     })
  //   }
  //   Mult => {
  //     receive_value(move |val| {
  //       calculator_provider(current * val)
  //     })
  //   }
  //   Div => {
  //     receive_value(move |val| {
  //       if val == 0.0 {
  //         offer_case!(DivZero, terminate())
  //       } else {
  //         offer_case!(DivOk,
  //           calculator_provider(current / val))
  //       }
  //     })
  //   }
  //   Done => {
  //     send_value(current, terminate())
  //   }
  // })
}

fn calculator_client(
  x: f64,
  y: f64,
  z: f64,
) -> Session<ReceiveChannel<Calculator, SendValue<Option<f64>, End>>>
{
  receive_channel(move |calc| {
    todo!("implement calculator client here");
    // unfix_session(
    //   calc,
    //   choose!(
    //     calc,
    //     Mult,
    //     send_value_to(
    //       calc,
    //       x,
    //       unfix_session(
    //         calc,
    //         choose!(
    //           calc,
    //           Div,
    //           send_value_to(
    //             calc,
    //             y,
    //             case! { calc;
    //               DivOk => {
    //                 unfix_session(calc,
    //                   choose!(calc, Add,
    //                     send_value_to(calc, z,
    //                       unfix_session(calc,
    //                         choose!(calc, Done,
    //                           receive_value_from(calc, move |res| {
    //                             send_value(Some(res),
    //                               wait(calc, terminate()))
    //                           }))))))
    //               }
    //               DivZero => {
    //                 send_value(None,
    //                   wait(calc, terminate()))
    //               }
    //             }
    //           )
    //         )
    //       )
    //     )
    //   ),
    // )
  })
}

async fn calculate(init: f64, x: f64, y: f64, z: f64) {
  let res = run_session_with_result(apply_channel(
    calculator_client(x, y, z),
    calculator_provider(init),
  ))
  .await;

  match res {
    Some(res) => {
      println!(
        "result of calculating (({} * {}) / {}) + {}: {}",
        init, x, y, z, res
      );
    }
    None => {
      println!(
        "error calculating (({} * {}) / {}) + {}: divide by zero",
        init, x, y, z
      );
    }
  }
}

#[tokio::main]
pub async fn main() {
  env_logger::init();

  calculate(2.0, 3.0, 4.0, 5.0).await;

  calculate(4.0, 2.0, 0.0, -2.0).await;
}
