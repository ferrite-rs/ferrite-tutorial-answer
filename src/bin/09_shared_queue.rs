use ferrite_session::prelude::*;

/*
  # Excercise 8: Shared Queue

  - Implement a shared queue provider consist of chains of shared processes,
    providing the shared session type Queue with following operations:

    - Enqueue:
      - Receive a string value,

      - Enqueue it to the back of of the queue and then release.

    - Dequeue:

      - If the queue is not empty:

        - Pop the front of the queue
        - Offer the branch `Elem`
        - Send the string value

      - If the queue is empty, sends `Empty`.

  - Implement an empty queue shared provider.

  - Implement a head queue shared provider, which takes a string value and a
    tail shared channel `SharedChannel<Queue>`, and offers a new shared channel
    with the head being the given string value, and tail operations delegated to
    the other shared process.

  - Implement an enqueue client which is given a string value and a shared
    channel, and runs a linear session that enqueue the given value to the
    shared queue.

  - Implement a dequeue client which is given a shared channel, and
    runs a linear session that tries to dequeue from the shared queue
    and sends back an `Option<String>` value.

  The provided main function spawns the shared providers and clients,
  and attempt to enqueue and dequeue from the shared queue.

  After completing your solution, you should get the following result
  running the program:

  ```
  $ cargo run --bin 09_shared_queue
  Gotten dequeue value: Foo
  Gotten dequeue value: Bar
  Dequeue returns None
  Gotten dequeue value: Baz
  Dequeue returns None
  ```
*/

type Queue = LinearToShared<ExternalChoice<QueueOps>>;

define_choice! { QueueOps;
  Enqueue: ReceiveValue<String, Release>,
  Dequeue: InternalChoice<DequeueOps>
}

define_choice! { DequeueOps;
  HeadVal: SendValue<String, Release>,
  QueueEmpty: Release,
}

fn empty_queue() -> SharedSession<Queue>
{
  // todo!("Implement empty queue here");
  accept_shared_session(offer_choice! {
    Enqueue =>
      receive_value(move |val| {
        detach_shared_session(
          head_queue(val, run_shared_session(empty_queue())))
      }),
    Dequeue =>
      offer_case!(QueueEmpty,
        detach_shared_session(empty_queue()))
  })
}

fn head_queue(
  val1: String,
  tail: SharedChannel<Queue>,
) -> SharedSession<Queue>
{
  // todo!("Implement head queue here");
  accept_shared_session(offer_choice! {
    Enqueue =>
      receive_value(move |val2| {
        acquire_shared_session(tail.clone(), move |c| {
          choose!(c, Enqueue,
            send_value_to(c, val2,
              release_shared_session(c,
                detach_shared_session(
                  head_queue(val1, tail)))))
        })
      }),
    Dequeue =>
      offer_case!(HeadVal,
        send_value(val1,
          shared_forward(tail)))
  })
}

fn enqueue_client(
  queue: SharedChannel<Queue>,
  val: String
) -> Session<End>
{
  // todo!("Implement enqueue client here");
  acquire_shared_session(queue, move |c| {
    choose!(
      c,
      Enqueue,
      send_value_to(c, val, release_shared_session(c, terminate()))
    )
  })
}

fn dequeue_client(
  queue: SharedChannel<Queue>,
) -> Session<SendValue<Option<String>, End>>
{
  // todo!("Implement dequeue client here");
  acquire_shared_session(queue, move |c| {
    choose!(
      c,
      Dequeue,
      case! { c;
        HeadVal =>
          receive_value_from(c, move |val| {
            release_shared_session(c,
              send_value(Some(val), terminate()))
          }),
        QueueEmpty => {
          release_shared_session(c,
            send_value(None, terminate()))
        }
      }
    )
  })
}

async fn enqueue(
  queue: SharedChannel<Queue>,
  val: String,
)
{
  run_session(enqueue_client(queue, val)).await;
}

async fn dequeue_and_print(queue: SharedChannel<Queue>)
{
  let res = run_session_with_result(dequeue_client(queue)).await;
  match res {
    Some(val) => {
      println!("Gotten dequeue value: {}", val);
    }
    None => {
      println!("Dequeue returns None");
    }
  }
}

#[tokio::main]
pub async fn main()
{
  env_logger::init();

  let queue = run_shared_session(empty_queue());

  enqueue(queue.clone(), "Foo".to_string()).await;
  enqueue(queue.clone(), "Bar".to_string()).await;

  dequeue_and_print(queue.clone()).await;
  dequeue_and_print(queue.clone()).await;
  dequeue_and_print(queue.clone()).await;

  enqueue(queue.clone(), "Baz".to_string()).await;
  dequeue_and_print(queue.clone()).await;
  dequeue_and_print(queue.clone()).await;
}
