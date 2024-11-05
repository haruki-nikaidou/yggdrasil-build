use async_nats::Client;
use clap::Parser;
use futures_util::stream::StreamExt;
use sea_orm::{Database, DatabaseConnection};
use std::sync::{Arc};
use tokio::sync::{Mutex};
use tracing::{info};
use yggdrasil_bus::NatsJsonMessage;
use yggdrasil_scheduler::{AddScheduleRequest, DeleteScheduleRequest};

mod config;
mod entities;

async fn handle_add_schedule_request(
    db: Arc<DatabaseConnection>,
    nats: Arc<Client>,
    is_error: Arc<Mutex<bool>>,
) -> anyhow::Result<()> {
    let mut adding_subscriber = nats.subscribe(AddScheduleRequest::subject()).await?;
    while let Some(msg) = adding_subscriber.next().await {
        let rep = msg.reply;
        let msg = msg.payload;

        let deserialized = AddScheduleRequest::from_json_bytes(&msg)?;
        let adding_result = entities::scheduled_event::Model::add_event(db.as_ref(), &deserialized).await?;
        if let Some(reply_to) = rep {
            nats.publish(reply_to, adding_result.to_json_bytes()?.into())
                .await?;
        }
    }
    adding_subscriber.unsubscribe().await?;
    let mut error = is_error.lock().await;
    *error = true;
    Ok(())
}

async fn handle_delete_schedule_request(
    db: Arc<DatabaseConnection>,
    nats: Arc<Client>,
    is_error: Arc<Mutex<bool>>,
) -> anyhow::Result<()> {
    let mut deleting_subscriber = nats.subscribe(DeleteScheduleRequest::subject()).await?;
    while let Some(msg) = deleting_subscriber.next().await {
        let msg = msg.payload;
        let deserialized = DeleteScheduleRequest::from_json_bytes(&msg)?;
        entities::scheduled_event::Model::delete_event(db.as_ref(), &deserialized).await?;
    }
    deleting_subscriber.unsubscribe().await?;
    let mut error = is_error.lock().await;
    *error = true;
    Ok(())
}

async fn push_on_time_events(
    db: Arc<DatabaseConnection>,
    nats: Arc<Client>,
) -> anyhow::Result<()> {
    let now = chrono::Utc::now().naive_utc();
    let events = entities::scheduled_event::Model::get_on_time(db.as_ref(), now).await?;
    entities::scheduled_event::Model::push_batch_into_queue(events, db.as_ref(), nats.as_ref()).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = config::Args::parse();
    let config = config::read_config_file(&args.config_file).await?;
    config::set_tracing_subscriber(config.log_level);
    let db = Database::connect(&config.postgres).await?;
    let db = Arc::new(db);
    let nats = async_nats::connect(&config.nats).await?;
    let nats = Arc::new(nats);
    let ms = config.ms;
    info!("Scheduler connected to NATS and database");
    let is_error = Arc::new(Mutex::new(false));

    let is_error_clone1 = is_error.clone();
    let db_clone_1 = db.clone();
    let nats_clone_1 = nats.clone();
    tokio::spawn(async move {
        if handle_add_schedule_request(db_clone_1, nats_clone_1, is_error_clone1.clone())
            .await
            .is_err()
        {
            let mut error = is_error_clone1.lock().await;
            *error = true;
        }
    });

    let is_error_clone2 = is_error.clone();
    let db_clone_2 = db.clone();
    let nats_clone_2 = nats.clone();
    tokio::spawn(async move {
        if handle_delete_schedule_request(db_clone_2, nats_clone_2, is_error_clone2.clone())
            .await
            .is_err()
        {
            let mut error = is_error_clone2.lock().await;
            *error = true;
        }
    });

    tokio::spawn(async move {
        loop {
            if *is_error.lock().await {
                break;
            }
            if push_on_time_events(db.clone(), nats.clone()).await.is_err() {
                let mut error = is_error.lock().await;
                *error = true;
            }
            tokio::time::sleep(std::time::Duration::from_millis(ms.into())).await;
        }
    });
    Ok(())
}
