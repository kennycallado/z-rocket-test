use chrono::NaiveDateTime;
use diesel::PgConnection;
use escalon_jobs::EscalonJobStatus;
use rocket::serde::uuid::Uuid;
use serde::{Deserialize, Serialize};
use escalon_jobs::{NewEscalonJob, EscalonJobTrait, EscalonJob};
use rocket_sync_db_pools::ConnectionPool;

use crate::database::connection::Db;
use crate::database::schema::escalonjobs;

use crate::app::server::Context;
use crate::app::modules::cron::model::{NewAppJob, AppJob, AppJobComplete};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = escalonjobs)]
#[serde(crate = "rocket::serde")]
pub struct EJob {
    pub id: Uuid,
    pub status: String,
    pub schedule: String,
    pub since: Option<NaiveDateTime>,
    pub until: Option<NaiveDateTime>,
}

impl From<EscalonJob> for EJob {
    fn from(escalon: EscalonJob) -> Self {
        Self {
            id: escalon.job_id,
            status: match escalon.status {
                EscalonJobStatus::Scheduled => "scheduled".to_string(),
                EscalonJobStatus::Running => "running".to_string(),
                EscalonJobStatus::Done => "done".to_string(),
                EscalonJobStatus::Failed => "failed".to_string(),
            },
            schedule: escalon.schedule,
            since: escalon.since,
            until: escalon.until,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewEJob {
    pub schedule: String,
    pub since: Option<NaiveDateTime>,
    pub until: Option<NaiveDateTime>,
}

impl From<NewEJob> for NewEscalonJob {
    fn from(ejob: NewEJob) -> Self {
        NewEscalonJob {
            schedule: ejob.schedule,
            since: ejob.since,
            until: ejob.until,
        }
    }
}

#[async_trait]
impl EscalonJobTrait<Context<ConnectionPool<Db, PgConnection>>> for NewEJob {
    async fn run_job(&self, job: EscalonJob, _ctx: Context<ConnectionPool<Db, PgConnection>>) -> EscalonJob {
        println!("running job: {}", job.job_id);
        // use diesel::prelude::*;
        // use crate::database::schema::{appjobs, escalonjobs};

        // let blah = ctx.0.get().await.unwrap().run(move |conn| {
        //     let app_job: AppJob = appjobs::table
        //         .filter(appjobs::job_id.eq(job.job_id))
        //         .first::<AppJob>(conn).unwrap();

        //     let escalon_job = escalonjobs::table
        //         .find(job.job_id)
        //         .first::<EJob>(conn).unwrap();

        //     AppJobComplete {
        //         id: app_job.id,
        //         service: app_job.service,
        //         route: app_job.route,
        //         job: escalon_job,
        //     }

        // }).await;

        job
    }
}
