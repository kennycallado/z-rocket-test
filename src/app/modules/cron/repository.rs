use diesel::prelude::*;
use rocket::serde::uuid::Uuid;

use crate::app::modules::cron::model::{AppJob, NewAppJob, AppJobComplete};
use crate::app::modules::escalon::model::EJob;
use crate::database::connection::Db;
use crate::database::schema::{appjobs, escalonjobs};

pub async fn get_all(db: &Db) -> Result<Vec<AppJob>, diesel::result::Error> {
    db.run(move |conn| {
        appjobs::table
            .load::<AppJob>(conn)
    }).await
}

pub async fn get_by_id(db: &Db, id: i32) -> Result<AppJob, diesel::result::Error> {
    db.run(move |conn| {
        appjobs::table
            .find(id)
            .first::<AppJob>(conn)
    }).await
}

pub async fn get_by_job_id(db: &Db, job_id: Uuid) -> Result<AppJob, diesel::result::Error> {
    db.run(move |conn| {
        appjobs::table
            .filter(appjobs::job_id.eq(job_id))
            .first::<AppJob>(conn)
    }).await
}

pub async fn get_complete(db: &Db, id: i32) -> Result<AppJobComplete, diesel::result::Error> {
    db.run(move |conn| {
        let app_job = appjobs::table
            .find(id)
            .first::<AppJob>(conn)?;

        let escalon_job = escalonjobs::table
            .find(app_job.job_id)
            .first::<EJob>(conn)?;

        Ok(AppJobComplete {
            id: app_job.id,
            owner: app_job.owner,
            service: app_job.service,
            route: app_job.route,
            job: escalon_job,
        })
    }).await
}

pub async fn create(db: &Db, app_job: NewAppJob) -> Result<AppJob, diesel::result::Error> {
    db.run(move |conn| {
        diesel::insert_into(appjobs::table)
            .values(app_job)
            .get_result::<AppJob>(conn)
    }).await
}
