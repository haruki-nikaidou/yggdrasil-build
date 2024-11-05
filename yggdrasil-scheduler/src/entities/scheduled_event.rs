use chrono::NaiveDateTime;
use sea_orm::prelude::*;
use sea_orm::ActiveValue::Set;
use yggdrasil_scheduler::{AddScheduleRequest, AddScheduleResponse, DeleteScheduleRequest};

#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "scheduler_queue", schema_name = "yggdrasil")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(indexed)]
    pub time: NaiveDateTime,
    #[sea_orm(column_type = "Json")]
    pub content: String,
    pub future_subject: String,
    #[sea_orm(default_value = false)]
    pub consumed: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub async fn add_event(
        db: &impl ConnectionTrait,
        req: &AddScheduleRequest,
    ) -> Result<AddScheduleResponse, DbErr> {
        let content = serde_json::to_string(req).unwrap();
        let mut active: ActiveModel = ActiveModel::new();
        active.time = Set(req.time);
        active.content = Set(content);
        active.future_subject = Set(req.future_subject.clone());
        let res = active.insert(db).await?;
        Ok(AddScheduleResponse { id: res.id })
    }
    pub async fn delete_event(
        db: &impl ConnectionTrait,
        req: &DeleteScheduleRequest,
    ) -> Result<(), DbErr> {
        let id = req.id;
        Entity::delete_by_id(id).exec(db).await?;
        Ok(())
    }
    pub async fn get_on_time(
        db: &impl ConnectionTrait,
        time: NaiveDateTime,
    ) -> Result<Vec<Self>, DbErr> {
        Entity::find()
            .filter(Column::Time.lte(time))
            .filter(Column::Consumed.eq(false))
            .all(db)
            .await
    }
    pub async fn push_batch_into_queue(
        batch: Vec<Self>,
        db: &impl ConnectionTrait,
        nats: &async_nats::Client,
    ) -> anyhow::Result<()> {
        let mut success_ids = Vec::new();
        for event in batch {
            let subject = event.future_subject;
            let message = event.content;
            let success = nats.publish(subject, message.into()).await.is_ok();
            if success {
                success_ids.push(event.id);
            }
        }
        let mut active = ActiveModel::new();
        active.consumed = Set(true);
        Entity::update_many()
            .filter(Column::Id.is_in(success_ids))
            .set(active)
            .exec(db)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Entity;
    use sea_orm::{DbBackend, Schema};

    #[test]
    fn create_table_sql() {
        let db_postgres = DbBackend::Postgres;
        let schema = Schema::new(db_postgres);
        let _1 = db_postgres.build(&schema.create_table_from_entity(Entity));
    }
}
