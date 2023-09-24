use std::net::IpAddr;
use escalon_jobs::EscalonJob;
use rocket::route::BoxFuture;
use rocket::tokio::net::UdpSocket;
use rocket_sync_db_pools::ConnectionPool;

use crate::app::modules::cron::model::{AppJob, NewAppJob, AppJobComplete};
use crate::app::modules::escalon::model::EJob;

#[cfg(feature = "cron")]
use crate::app::providers::services::cron::CronManager;
#[cfg(feature = "db")]
use crate::database::connection;
use crate::database::schema::appjobs;
use diesel::{PgConnection, ExpressionMethods};
use escalon_jobs::manager::{EscalonJobsManager, ContextTrait, EscalonJobsManagerTrait};
use rocket::{Rocket, Build, State, Orbit};
#[cfg(feature = "db")]
use rocket::fairing::AdHoc;

use crate::app::providers::cors;
#[cfg(feature = "fetch")]
use crate::app::providers::services::fetch::Fetch;

use super::modules::routing as modules_routing;
use super::providers::config_getter::ConfigGetter;
use super::routing as service_routing;

use crate::database::connection::Db;

#[launch]
pub async fn launch() -> _ {
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
    //
    // rocket_build = rocket_build.attach(AdHoc::on_liftoff("blah blah", test));
    // rocket_build = rocket_build.attach(AdHoc::on_liftoff("blah blah", |rocket| {
    //     let rocket = rocket.clone();
    //     let pool = Db::get_one(&rocket);

    //     Box::pin(async move {
    //         let _pool = match pool.await {
    //             Some(pool) => {pool},
    //             None => panic!("No pool"),
    //         };

    //         let _jm = EscalonJobsManager::new(Context(rocket));
    //     })
    // }));
    //
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

type ContextDb = ConnectionPool<Db, PgConnection>;

#[derive(Clone)]
pub struct Context<T>(pub T);

#[rocket::async_trait]
impl ContextTrait<Context<ContextDb>> for Context<ContextDb> {
    // TODO
    // if returns Ok, then escalon-jobs will be added to the list
    // YEAH
    async fn update_job(&self, ctx: &Context<ContextDb>, job: EscalonJob) {
        use diesel::prelude::*;
        use crate::database::schema::{appjobs, escalonjobs};

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
        
        let ejob: EJob = job.into();
        let blah = ctx.0.get().await.unwrap().run(move |conn| {
            diesel::update(escalonjobs::table)
                .filter(escalonjobs::id.eq(ejob.id))
                .set(&ejob)
                .get_result::<EJob>(conn).unwrap()
            
        }).await;

        println!("update_job: {:?} - {}", blah.id, blah.status);

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

    // async fn take_jobs(&self, ctx: &Context<ContextDb>, from: String, start_at: usize, n_jobs: usize) {
    //     use diesel::prelude::*;
    //     use crate::database::schema::{appjobs, escalonjobs};

    //     println!("taking jobs: {} {} {}", from, start_at, n_jobs);
    //     let jobs: Vec<AppJob> = ctx.0.get().await.unwrap().run(move |conn| {
    //         appjobs::table
    //             .filter(appjobs::owner.eq(from))
    //             .limit(n_jobs as i64)
    //             .offset(start_at as i64)
    //             .load::<AppJob>(conn).unwrap()
    //     }).await;

    //     for job in jobs {
    //         let mut new_job: NewAppJob = job.into();
    //         new_job.owner = ConfigGetter::get_identity();

    //         let new_job = ctx.0.get().await.unwrap().run(move |conn| {
    //             diesel::insert_into(appjobs::table)
    //                 .values(&new_job)
    //                 .get_result::<AppJob>(conn)
    //         }).await;
    //     }
    // }
}

struct Functions;

#[async_trait]
impl EscalonJobsManagerTrait<Context<ContextDb>> for Functions {
    async fn take_jobs(&self, manager: &EscalonJobsManager<Context<ContextDb>>, from_client: String, start_at: usize, n_jobs: usize) -> Result<Vec<String>, ()> {
        use diesel::prelude::*;

        use crate::app::modules::escalon::repository as escalon_repository;
        use crate::app::modules::escalon::model::{NewEJob, EJob};
        use crate::database::connection::Db;
        use crate::database::schema::{appjobs, escalonjobs};

        let id = ConfigGetter::get_identity();

        let mut jobs_added = Vec::new();

        // TODO: Should be resolved in escalon
        if id == from_client { return Ok(jobs_added) }

        let jobs: Vec<AppJob> = manager.context.0.0.get().await.unwrap().run(move |conn| {
            appjobs::table
                .filter(appjobs::owner.eq(from_client))
                .limit(n_jobs as i64)
                .offset(start_at as i64)
                .load::<AppJob>(conn).unwrap()
        }).await;

        for job in jobs {
            // let mut new_job: NewAppJob = job.into();
            // new_job.owner = ConfigGetter::get_identity();

            let new_ejob: EJob = manager.context.0.0.get().await.unwrap().run(move |conn| {
                escalonjobs::table
                    .filter(escalonjobs::id.eq(job.job_id.clone()))
                    .first::<EJob>(conn).unwrap()
            }).await;

            let old_uuid = new_ejob.id.clone();

            let new_ejob: NewEJob = new_ejob.into();
            let escalon_job = manager.add_job(new_ejob).await;
            let new_job = NewAppJob {
                owner: ConfigGetter::get_identity(),
                service: job.service,
                route: job.route,
                job_id: escalon_job.job_id.clone(),
            };

            let ejob: EJob = escalon_job.clone().into();

            // escalon_repository::insert(&db, escalon_job.into()).await.unwrap();
            manager.context.0.0.get().await.unwrap().run(move |conn| {
                diesel::insert_into(escalonjobs::table)
                    .values(ejob)
                    .get_result::<EJob>(conn).unwrap()
            }).await;

            // let job = cron_repository::create(&db, new_job).await.unwrap();
            let job: AppJob = manager.context.0.0.get().await.unwrap().run(move |conn| {
                diesel::insert_into(appjobs::table)
                    .values(new_job)
                    .get_result::<AppJob>(conn).unwrap()
            }).await;
            // let job = cron_repository::get_complete(&db, job.id).await.unwrap();

            jobs_added.push(old_uuid.to_string());
        }

        Ok(jobs_added)
    }

    async fn drop_jobs(&self, manager: &EscalonJobsManager<Context<ContextDb>>, jobs: Vec<String>) -> Result<(), ()> {
        use diesel::prelude::*;
        use crate::database::schema::{appjobs, escalonjobs};
        use rocket::serde::uuid::Uuid;

        println!("Jobs comming: {}", jobs.len());
        println!("Current jobs: {:?}", manager.jobs.lock().unwrap().len());

        let mut affected_rows: usize = 0;

        for job in jobs {
            let job_id: Uuid = Uuid::parse_str(job.as_str()).unwrap();

            // let job = manager.get_job(job_id);
            if let None = manager.get_job(job_id).await {
                continue;
            };

            let id: i32 = manager.context.0.0.get().await.unwrap().run(move |conn| {
                appjobs::table
                    .filter(appjobs::job_id.eq(job_id.clone()))
                    .select(appjobs::id)
                    .first::<i32>(conn).unwrap()
            }).await;


            manager.context.0.0.get().await.unwrap().run(move |conn| {
                diesel::delete(appjobs::table.find(&id))
                    .execute(conn).unwrap();

                diesel::delete(escalonjobs::table.find(&job_id))
                    .execute(conn).unwrap();
            }).await;

            affected_rows += 1;

            println!("{}", job_id);
            manager.remove_job(job_id).await;
        }

        println!("Current jobs: {:?}", manager.jobs.lock().unwrap().len());
        println!("Jobs dropped: {}", affected_rows);
        Ok(())
    }
}

async fn test(rocket: Rocket<Build>) -> Rocket<Build> {
    let pool = match Db::pool(&rocket) {
        Some(pool) => pool.clone(), // clone the wrapped pool
        None => return rocket,
    };

    let functions = Functions;
    let jm = EscalonJobsManager::new(Context(pool));
    let jm = jm
        .set_id(ConfigGetter::get_identity())
        .set_addr("0.0.0.0".parse::<IpAddr>().unwrap())
        .set_port(ConfigGetter::get_port().unwrap_or(65056))
        .set_functions(functions)
        .build()
        .await;

    jm.init().await;

    // rocket::tokio::spawn(async move {
    //     let socket = UdpSocket::bind("0.0.0.0:8000").await.unwrap();
    //     let mut buf = [u8::MAX; 1024];

    //     while let Ok((size, _)) = socket.recv_from(&mut buf).await {
    //         let msg = &buf[..size];
    //         println!("Recived: {:?}", String::from_utf8(msg.to_vec()));
    //     };
    // });


    rocket.manage(jm)
}
