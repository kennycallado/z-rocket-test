use std::net::IpAddr;
use escalon_jobs::EscalonJob;
use rocket_sync_db_pools::ConnectionPool;

use crate::app::modules::cron::model::{AppJob, AppJobComplete};
use crate::app::modules::escalon::model::EJob;

#[cfg(feature = "cron")]
use crate::app::providers::services::cron::CronManager;
#[cfg(feature = "db")]
use crate::database::connection;
use diesel::PgConnection;
use escalon_jobs::manager::{EscalonJobsManager, ContextTrait};
use rocket::{Rocket, Build, State, Orbit};
#[cfg(feature = "db")]
use rocket::fairing::AdHoc;

use crate::app::providers::cors;
#[cfg(feature = "fetch")]
use crate::app::providers::services::fetch::Fetch;

use super::modules::routing as modules_routing;
use super::routing as service_routing;

use crate::database::connection::Db;

#[launch]
pub async fn rocket() -> _ {
    #[allow(unused_mut)]
    let mut rocket_build = rocket::build();

    #[cfg(feature = "db")]
    {
        rocket_build = rocket_build
            .attach(connection::Db::fairing())
            .attach(AdHoc::on_ignite(
                "Diesel Migrations",
                connection::run_migrations,
            ));
    }

    #[cfg(feature = "fetch")]
    {
        rocket_build = rocket_build.manage(Fetch::new());
    }

    #[cfg(feature = "cron")]
    {
        let mut db_url = None;
        if cfg!(feature = "db") {
            db_url = Some(
                rocket::Config::figment()
                    .extract_inner::<String>("databases.questions.url")
                    .unwrap(),
            );
        }

        rocket_build = rocket_build.manage(CronManager::new(db_url).await);

        if cfg!(feature = "db") {
            let cron_manager = rocket_build
                .state::<CronManager>()
                .expect("ERROR: rocket(); cron manager");

            cron_manager.init_from_db().await;
        }
    }

    // Manage new escalonjobs
    rocket_build = rocket_build.attach(AdHoc::on_ignite("Adding escalonjobs", test));
    // rocket_build = rocket_build.attach(AdHoc::on_liftoff("Adding escalonjobs", |rocket| {
    //     let pool = match Db::get_one(&rocket) {
    //         Some(pool) => pool.0.clone(), // clone the wrapped pool
    //         None => return Err(rocket),
    //     };


    //     Box::pin(async move {
    //         println!("Adding escalonjobs");
    //     })
    // }));

    rocket_build
        .attach(cors::Cors)
        .attach(service_routing::router())
        .attach(modules_routing::router())
}

#[derive(Debug, Clone)]
pub struct Context<T>(pub T);

type ContextDb = Context<ConnectionPool<Db, PgConnection>>;

#[rocket::async_trait]
impl ContextTrait<ContextDb> for ContextDb {
    async fn update_job(&self, job: EscalonJob, Context(ctx): &ContextDb) {
        use diesel::prelude::*;
        use crate::database::schema::{appjobs, escalonjobs};

        let blah = ctx.get().await.unwrap().run(move |conn| {
            let app_job: AppJob = appjobs::table
                .filter(appjobs::job_id.eq(job.job_id))
                .first::<AppJob>(conn).unwrap();

            let escalon_job = escalonjobs::table
                .find(job.job_id)
                .first::<EJob>(conn).unwrap();

            AppJobComplete {
                id: app_job.id,
                service: app_job.service,
                route: app_job.route,
                job: escalon_job,
            }
        }).await;
        
        println!("update_job: {:?}", blah);

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
    }

    async fn take_jobs(&self, from: String, start_at: usize, n_jobs: usize) {
        println!("taking jobs: {} {} {}", from, start_at, n_jobs)
    }
}

async fn test(rocket: Rocket<Build>) -> Rocket<Build> {
    let pool = match Db::pool(&rocket) {
        Some(pool) => pool.clone(), // clone the wrapped pool
        None => return rocket,
    };

    let jm = EscalonJobsManager::new(Context(pool));
    let jm = jm
        .set_id("Blah".to_string())
        .set_addr("0.0.0.0".parse::<IpAddr>().unwrap())
        .set_port(65065)
        .build()
        .await;

    jm.init().await;

    rocket.manage(jm)
}
