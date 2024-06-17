use dotenv::dotenv;
use meilisearch_sdk::{client::*, SucceededTask, Task, TasksSearchQuery};
use serde::{Deserialize, Serialize};
use std::env;

mod datadog_client;

use aws_lambda_events::event::eventbridge::EventBridgeEvent;
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(_event: LambdaEvent<EventBridgeEvent>) -> Result<(), Error> {
    // Extract some useful information from the request

    dotenv().ok();
    let hosts = get_hosts();

    let mut clients: Vec<Client> = Vec::new();
    for host in hosts {
        let client = Client::new(host, Some(""));
        clients.push(client);
    }

    println!("Meilisearch Health Check Started!");

    if clients.len() <= 0 {
        println!("{:?}", String::from("No meilisearch host to check"));
    }

    check_failed_tasks(&clients).await;

    check_slow_processing_task(&clients).await;

    println!("Meilisearch Health Check Ended!");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}

#[derive(Serialize, Deserialize)]
struct TaskResponse {
    tasks: Vec<String>,
}

async fn check_failed_tasks(clients: &Vec<Client>) {
    println!("Checking Failed Tasks Start...");

    for client in clients {
        let mut query = TasksSearchQuery::new(&client);
        query.with_limit(1);

        let task_results = client.get_tasks_with(&query).await.unwrap();
        let first_task = &task_results.results[0];

        match first_task {
            Task::Succeeded { content: _ } => panic!("Wrong filter"),
            Task::Enqueued { content: _ } => panic!("Wrong filter"),
            Task::Processing { content: _ } => panic!("Wrong filter"),
            Task::Failed { content } => datadog_client::send_histogram(
                "task_failed",
                &content.error.error_message.to_string(),
            ),
        }
    }

    println!("Checking Failed Tasks Ended...");
}

async fn check_slow_processing_task(clients: &Vec<Client>) {
    for client in clients {
        let mut query = TasksSearchQuery::new(&client);
        query.with_limit(1);

        let task_results = client.get_tasks_with(&query).await.unwrap();
        let first_task = &task_results.results[0];

        match first_task {
            Task::Succeeded { content } => {
                if check_is_old(&content) {
                    datadog_client::send_histogram("task_old", "")
                }
            }
            Task::Enqueued { content: _ } => panic!("Wrong filter"),
            Task::Processing { content: _ } => panic!("Wrong filter"),
            Task::Failed { content: _ } => panic!("Wrong filter"),
        }
    }
}

fn get_hosts() -> Vec<String> {
    let hosts_str = env::var("HOSTS").unwrap();
    let hosts = serde_json::from_str(&hosts_str).unwrap_or_else(|err| {
        println!("Error when parsing hosts from env: {:?}", err);
        vec![]
    });
    hosts
}

fn check_is_old(task: &SucceededTask) -> bool {
    let time_now = time::OffsetDateTime::now_utc();
    let time_enqueued = task.enqueued_at;
    let old_threshold_in_ms_str =
        env::var("OLD_THRESHOLD_IN_MS").unwrap_or((60 * 60 * 1000).to_string());
    let old_threshold_in_ms: i64 = old_threshold_in_ms_str.parse().unwrap();
    let time_diff_ms = time_now.unix_timestamp() - time_enqueued.unix_timestamp();

    if time_diff_ms > old_threshold_in_ms {
        return true;
    }

    false
}
