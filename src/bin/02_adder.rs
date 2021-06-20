use ferrite_session::prelude::*;

/**
  # Excercise 2: Adder Client

  - Implement `adder_client` that uses the defined adder_provider
    to calculate 1 + 2, and print the result.

  - Implement `main_session` to link `adder_provider` with
    `adder_client`.

  After completing your solution, you should get the following result
  running the program:

  ```
  $ cargo run --bin 02_adder
  Result of 1 + 2 = 3
  ```
**/

type Adder =
  ReceiveValue<i32, ReceiveValue<i32, SendValue<i32, End>>>;

fn adder_provider() -> Session<Adder> {
  receive_value(move |x| {
    receive_value(move |y| send_value(x + y, terminate()))
  })
}

fn adder_client() -> Session<ReceiveChannel<Adder, End>> {
  // todo!("implement adder consumer here");
  receive_channel(move |adder| {
    send_value_to(
      adder,
      1,
      send_value_to(
        adder,
        2,
        receive_value_from(adder, move |result| {
          println!("Result of 1 + 2 = {}", result);
          wait(adder, terminate())
        }),
      ),
    )
  })
}

fn main_session() -> Session<End> {
  // todo!("link adder_provider with adder_consumer here")
  apply_channel(adder_client(), adder_provider())
}

#[tokio::main]
pub async fn main() {
  env_logger::init();

  run_session(main_session()).await
}
