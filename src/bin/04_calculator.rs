use ferrite_session::prelude::*;

define_choice! { CalculatorOps;
  Add: ReceiveValue<(i32, i32), SendValue<i32, End>>,
  Mult: ReceiveValue<(i32, i32), SendValue<i32, End>>,
}

fn calculator_provider() -> Session<ExternalChoice<CalculatorOps>> {
  todo!("Implement a calculator provider here");
  // offer_choice! {
  //   Add =>
  //     receive_value(move |(x, y)| {
  //       send_value(x + y, terminate())
  //     }),
  //   Mult =>
  //     receive_value(move |(x, y)| {
  //       send_value(x * y, terminate())
  //     }),
  // }
}

// Challenge: Implement a main program that includes two instances
// of calculator_provider, calculate the result of 3 + 4 * 5,
// and prints the result.
fn main_session() -> Session<End> {
  include_session(calculator_provider(), move |c1| {
    include_session(calculator_provider(), move |c2| {
      todo!("Implement a calculator client here");
      // choose!(
      //   c1,
      //   Mult,
      //   send_value_to(
      //     c1,
      //     (4, 5),
      //     receive_value_from(c1, move |res1| {
      //       choose!(
      //         c2,
      //         Add,
      //         send_value_to(
      //           c2,
      //           (res1, 3),
      //           receive_value_from(c2, move |res2| {
      //             println!("result of 3 + 4 x 5: {}", res2);
      //             wait_all!([c1, c2], terminate())
      //           })
      //         )
      //       )
      //     })
      //   )
      // )
    })
  })
}

#[tokio::main]
pub async fn main() {
  env_logger::init();

  run_session(main_session()).await
}
