use chrono::NaiveDateTime;
use diesel::PgConnection;
use escalon_jobs::manager::Context;
use rocket::serde::uuid::Uuid;
use serde::{Deserialize, Serialize};
use escalon_jobs::{NewEscalonJob, EscalonJobTrait, EscalonJob};
use rocket_sync_db_pools::ConnectionPool;

use crate::database::connection::Db;
use crate::database::schema::escalonjobs;

use crate::app::modules::cron::model::{NewAppJob, AppJob};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = escalonjobs)]
#[serde(crate = "rocket::serde")]
pub struct EJob {
    pub id: Uuid,
    pub schedule: String,
    pub since: Option<NaiveDateTime>,
    pub until: Option<NaiveDateTime>,
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
impl EscalonJobTrait<ConnectionPool<Db, PgConnection>> for NewEJob {
    async fn run_job(&self, job: EscalonJob, ctx: Context<ConnectionPool<Db, PgConnection>>) -> EscalonJob {
        use diesel::prelude::*;
        use crate::database::schema::{appjobs, escalonjobs};

        let blah: AppJob = ctx.0.get().await.unwrap().run(move |conn| {
            appjobs::table
                .filter(appjobs::job_id.eq(job.job_id))
                .first::<AppJob>(conn).unwrap()

        }).await;

        println!("blah: {:?}", blah);

        job
    }
    async fn update_job(&self, job: &EscalonJob) {}

}
