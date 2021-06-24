use ferrite_session::prelude::*;

/*
  # Excercise 8: Linear Queue

  - Implement a queue provider consist of chains of linear processes,
    providing the session type Queue with following operations:

    - Enqueue: Receive a string value, enqueue it to the back of of the queue
      and then release.

    - Dequeue:
      - If the queue is not empty:
        - Offer the branch `Elem`
        - Pop the front of the queue and send the value
        - Recurse back to the Queue session type

      - If the queue is empty:
        - Offer the branch `Empty`
        - terminate

  - Implement an empty queue provider.

  - Implement a elem queue provider which:
      - Takes a string value
      - Receives a tail channel offering `Queue`,
      - Offers a new `Queue` channel with the given string value as head,
        and tail operations delegated to the tail process.

  - Implement an enqueue client which enqueues "Hello" and then
    enqueues "World" to the queue, then sends back the new queue.

  - Implement a dequeue client that dequeues all elements in the queue
    and then terminates.

    - If the current queue is non empty, prints
      "Dequeued value: {val}"
      and then continue dequeue the remaining elements

    - If the current queue is empty, prints
      "Queue is now empty" and then terminate

  The provided main function spawns the queues providers and clients,
  use the enqueue the strings, and then.

  After completing your solution, you should get the following result
  running the program:

  ```
  $ cargo run --bin 08_queue
  Dequeued value: World
  Dequeued value: Hello
  Queue is now empty
  ```
*/


type Queue = Rec<ExternalChoice<QueueOps>>;

define_choice! { QueueOps;
  Enqueue: ReceiveValue<String, Z>,
  Dequeue: InternalChoice<DequeueOps>
}

define_choice! { DequeueOps;
  Elem: SendValue<String, Z>,
  Empty: End,
}

fn empty() -> Session<Queue>
{
  fix_session(
    offer_choice! {
      Enqueue => {
        receive_value(move |val| {
          include_session(empty(), move |tail| {
            include_session(elem(val), move |head| {
              send_channel_to(head, tail,
                forward(head))
            })
          })
        })
      }
      Dequeue => {
        offer_case!(Empty,
          terminate())
      }
    })
}

fn elem(val: String) -> Session<ReceiveChannel<Queue, Queue>>
{
  receive_channel(move |tail| {
    fix_session(
      offer_choice! {
        Enqueue => {
          receive_value(move |new_val| {
            include_session(elem(val), move |new_tail| {
              send_channel_to(new_tail, tail,
                include_session(elem(new_val), move |head| {
                  send_channel_to(head, new_tail,
                    forward(head))
                }))
            })
          })
        }
        Dequeue => {
          offer_case!(Elem,
            send_value(val,
              forward(tail)))
        }
      })
  })
}

fn dequeue_all() -> Session<ReceiveChannel<Queue, End>>
{
  receive_channel(move |queue| {
    unfix_session(queue,
      choose!(queue, Dequeue,
        case!{ queue;
          Elem =>
            receive_value_from(queue, move |val| {
              println!("Dequeued value: {}", val);
              include_session(dequeue_all(), move |next| {
                send_channel_to(next, queue,
                  forward(next))
              })
            })
          Empty => {
            println!("Queue is now empty");
            wait(queue, terminate())
          }
        }))
    })
}

fn enqueue_hello_world() -> Session<ReceiveChannel<Queue, Queue>>
{
  receive_channel(move |queue| {
    unfix_session(queue,
      choose!(queue, Enqueue,
        send_value_to(queue, "Hello".to_string(),
          unfix_session(queue,
            choose!(queue, Enqueue,
              send_value_to(queue, "World".to_string(),
                forward(queue)))))))
  })
}

fn main_session() -> Session<End>
{
  include_session(empty(), move |queue| {
    include_session(enqueue_hello_world(), move |c1| {
      include_session(dequeue_all(), move |c2| {
        send_channel_to(c1, queue,
          send_channel_to(c2, c1,
            wait(c2, terminate())))
      })
    })
  })
}

#[tokio::main]
pub async fn main()
{
  env_logger::init();

  run_session(main_session()).await;
}
