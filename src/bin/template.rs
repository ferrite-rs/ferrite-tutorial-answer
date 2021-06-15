use ferrite_session::prelude::*;

fn main_session() -> Session<End> {
  todo!("fill in the blank")
}

#[tokio::main]
pub async fn main() {
  env_logger::init();

  run_session(main_session()).await
}
