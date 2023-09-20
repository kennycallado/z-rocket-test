use diesel::PgConnection;
use escalon_jobs::manager::EscalonJobsManager;
use rocket_sync_db_pools::ConnectionPool;
use escalon_jobs::NewEscalonJob;
use rocket::State;
use rocket::serde::json::Json;

use crate::app::modules::cron::model::{AppJob, AppJobComplete, NewAppJob};

use crate::app::modules::cron::repository as cron_repository;
use crate::app::modules::escalon::model::{NewEJob, EJob};
use crate::app::modules::escalon::repository as escalon_repository;
use crate::app::server::Context;
use crate::database::connection::Db;

use super::model::PostNewAppJob;

pub fn routes() -> Vec<rocket::Route> {
    routes![ index, show, create ]
}

#[get("/")]
pub async fn index(db: Db) -> Json<Vec<AppJob>> {
    let jobs = cron_repository::get_all(&db).await.unwrap();

    Json(jobs)
}

#[get("/<id>")]
pub async fn show(db: Db, id: i32) -> Json<AppJobComplete> {
    let job = cron_repository::get_complete(&db, id).await.unwrap();

    Json(job)
}

#[post("/", data = "<new_job>")]
pub async fn create(db: Db, jm: &State<EscalonJobsManager<Context<ConnectionPool<Db, PgConnection>>>>, new_job: Json<PostNewAppJob>) -> Json<AppJobComplete> {
    let new_job = new_job.into_inner();

    // let escalon_job = jm.inner().0.escalon.add_job(new_job.job.clone()).await;
    let escalon_job = jm.inner().add_job(new_job.job.clone()).await;

    let new_job = NewAppJob {
        service: new_job.service,
        route: new_job.route,
        job_id: escalon_job.job_id.clone(),
    };

    escalon_repository::insert(&db, escalon_job.into()).await.unwrap();

    let job = cron_repository::create(&db, new_job).await.unwrap();
    let job = cron_repository::get_complete(&db, job.id).await.unwrap();


    Json(job)
}
