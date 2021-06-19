use ferrite_session::prelude::*;

// Excercise 1.1
// Implement a greeter program that receives the name as
// a string value and then print out the line
//   "Hello, {name}!"
fn greeter() -> Session<ReceiveValue<String, End>> {
  todo!("implement greeter here");
  // receive_value(move |name| {
  //   println!("Hello, {}!", name);
  //   terminate()
  // })
}

fn main_session() -> Session<End> {
  include_session(greeter(), move |a| {
    send_value_to(a, "Alice".to_string(), wait(a, terminate()))
  })
}

#[tokio::main]
pub async fn main() {
  env_logger::init();

  run_session(main_session()).await
}
