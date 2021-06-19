use ferrite_session::prelude::*;

define_choice! { QueueOps;
  Enqueue: ReceiveValue<String, Release>,
  Dequeue: SendValue<Option<String>, Release>
}

type Queue = LinearToShared<ExternalChoice<QueueOps>>;

fn shared_queue(mut queue: Vec<String>) -> SharedSession<Queue> {
  accept_shared_session(move || {
    // todo!("Implement shared queue");
    offer_choice! {
      Enqueue => {
        receive_value(move |val: String| {
          queue.push(val);
          detach_shared_session(shared_queue(queue))
        })
      }
      Dequeue => {
        send_value(queue.pop(),
          detach_shared_session(shared_queue(queue)))
      }
    }
  })
}

fn create_shared_queue() -> SharedChannel<Queue> {
  run_shared_session(shared_queue(vec![]))
}

async fn enqueue(queue: &SharedChannel<Queue>, val: String) {
  run_session(acquire_shared_session(queue.clone(), move |chan| {
    // todo!("Implement enqueue client");
    choose!(
      chan,
      Enqueue,
      send_value_to(
        chan,
        val,
        release_shared_session(chan, terminate())
      )
    )
  }))
  .await;
}

async fn dequeue_and_print(queue: &SharedChannel<Queue>) {
  run_session(acquire_shared_session(queue.clone(), move |chan| {
    // todo!("Implement dequeue client");
    choose!(
      chan,
      Dequeue,
      receive_value_from(chan, move |val| {
        match val {
          Some(val) => {
            println!("Gotten dequeue value: {}", val);
          }
          None => {
            println!("Dequeue returns None");
          }
        }

        release_shared_session(chan, terminate())
      })
    )
  }))
  .await
}

#[tokio::main]
pub async fn main() {
  env_logger::init();

  let queue = create_shared_queue();

  enqueue(&queue, "Hello".to_string()).await;
  enqueue(&queue, "World".to_string()).await;

  dequeue_and_print(&queue).await;
  dequeue_and_print(&queue).await;
  dequeue_and_print(&queue).await;
}
