use ferrite_session::prelude::*;

type Adder =
  ReceiveValue<i32, ReceiveValue<i32, SendValue<i32, End>>>;

fn adder_provider() -> Session<Adder> {
  receive_value(move |x| {
    receive_value(move |y| send_value(x + y, terminate()))
  })
}

fn adder_consumer() -> Session<ReceiveChannel<Adder, End>> {
  // todo!("implement adder consumer here");
  receive_channel(move |adder| {
    send_value_to(
      adder,
      1,
      send_value_to(
        adder,
        2,
        receive_value_from(adder, move |result| {
          println!("Result of 1 + 2: {}", result);
          wait(adder, terminate())
        }),
      ),
    )
  })
}

fn main_session() -> Session<End> {
  // todo!("link adder_provider with adder_consumer here")
  apply_channel(adder_consumer(), adder_provider())
}

#[tokio::main]
pub async fn main() {
  env_logger::init();

  run_session(main_session()).await
}
