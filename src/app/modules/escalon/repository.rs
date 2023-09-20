use diesel::prelude::*;
use rocket::serde::uuid::Uuid;


use crate::app::modules::escalon::model::EJob;
use crate::database::connection::Db;
use crate::database::schema::escalonjobs;

pub async fn get_all(db: &Db) -> Result<Vec<EJob>, diesel::result::Error> {
    db.run(move |conn| {
        escalonjobs::table
            .load::<EJob>(conn)
    }).await
}

pub async fn get_by_id(db: &Db, id: Uuid) -> Result<EJob, diesel::result::Error> {
    db.run(move |conn| {
        escalonjobs::table
            .find(id)
            .first::<EJob>(conn)
    }).await
}

pub async fn insert(db: &Db, escalon_job: EJob) -> Result<EJob, diesel::result::Error> {
    db.run(move |conn| {
        diesel::insert_into(escalonjobs::table)
            .values(escalon_job)
            .get_result::<EJob>(conn)
    }).await
}

pub async fn update(db: &Db, escalon_job: EJob) -> Result<EJob, diesel::result::Error> {
    db.run(move |conn| {
        diesel::update(escalonjobs::table)
            .filter(escalonjobs::id.eq(escalon_job.id))
            .set(escalon_job)
            .get_result::<EJob>(conn)
    }).await
}
