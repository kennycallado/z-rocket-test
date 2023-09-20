use chrono::NaiveDateTime;
use rocket::serde::uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::database::schema::appjobs;
use crate::app::modules::escalon::model::{EJob, NewEJob};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable)]
#[diesel(table_name = appjobs)]
#[serde(crate = "rocket::serde")]
pub struct AppJob {
    pub id: i32,
    pub service: String,
    pub route: String,
    pub job_id: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, AsChangeset)]
#[diesel(table_name = appjobs)]
#[serde(crate = "rocket::serde")]
pub struct NewAppJob {
    pub service: String,
    pub route: String,
    pub job_id: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostNewAppJob {
    pub service: String,
    pub route: String,
    pub job: NewEJob,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct AppJobComplete {
    pub id: i32,
    pub service: String,
    pub route: String,
    pub job: EJob,
}
