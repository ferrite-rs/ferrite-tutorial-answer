use ferrite_session::prelude::*;

struct Soda;
struct NickelVal;
struct DimeVal;

type Nickel = SendValue<NickelVal, End>;
type Dime = SendValue<DimeVal, End>;

fn forge_nickel() -> Session<Nickel> {
  send_value(NickelVal, terminate())
}

fn forge_dime() -> Session<Dime> {
  send_value(DimeVal, terminate())
}

fn vending_machine(
) -> Session<ReceiveChannel<Dime, SendValue<Soda, End>>> {
  receive_channel(move |dime| {
    receive_value_from(dime, move |_| {
      send_value(Soda, wait(dime, terminate()))
    })
  })
}

fn exchange() -> Session<
  ReceiveChannel<
    Nickel,
    ReceiveChannel<Nickel, SendChannel<Dime, End>>,
  >,
> {
  receive_channel(move |nickel1| {
    receive_channel(move |nickel2| {
      receive_value_from(nickel1, move |_| {
        receive_value_from(nickel2, move |_| {
          include_session(forge_dime(), move |dime| {
            send_channel_from(
              dime,
              wait_all!([nickel1, nickel2], terminate()),
            )
          })
        })
      })
    })
  })
}

fn main_session() -> Session<End> {
  include_session(exchange(), move |exchange| {
    include_session(vending_machine(), move |machine| {
      include_session(forge_nickel(), move |nickel1| {
        include_session(forge_nickel(), move |nickel2| {
          // todo!("Exchange nickel for dime, adn then get soft drink from vending machine");
          send_channel_to(
            exchange,
            nickel1,
            send_channel_to(
              exchange,
              nickel2,
              receive_channel_from(exchange, |dime| {
                send_channel_to(
                  machine,
                  dime,
                  receive_value_from(machine, move |_soda| {
                    println!("gotten soda drink");
                    wait_all!([machine, exchange], terminate())
                  }),
                )
              }),
            ),
          )
        })
      })
    })
  })
}

#[tokio::main]
async fn main() {
  env_logger::init();

  run_session(main_session()).await
}
