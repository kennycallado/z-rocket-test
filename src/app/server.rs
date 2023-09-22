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
use escalon_jobs::manager::{EscalonJobsManager, ContextTrait};
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
                .get_result::<EJob>(conn)
            
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

    async fn take_jobs(&self, ctx: &Context<ContextDb>, from: String, start_at: usize, n_jobs: usize) {
        use diesel::prelude::*;
        use crate::database::schema::{appjobs, escalonjobs};

        println!("taking jobs: {} {} {}", from, start_at, n_jobs);
        let jobs: Vec<AppJob> = ctx.0.get().await.unwrap().run(move |conn| {
            appjobs::table
                .filter(appjobs::owner.eq(from))
                .limit(n_jobs as i64)
                .offset(start_at as i64)
                .load::<AppJob>(conn).unwrap()
        }).await;

        for job in jobs {
            let mut new_job: NewAppJob = job.into();
            new_job.owner = ConfigGetter::get_identity();

            let new_job = ctx.0.get().await.unwrap().run(move |conn| {
                diesel::insert_into(appjobs::table)
                    .values(&new_job)
                    .get_result::<AppJob>(conn)
            }).await;
        }
    }
}

async fn test(rocket: Rocket<Build>) -> Rocket<Build> {
    let pool = match Db::pool(&rocket) {
        Some(pool) => pool.clone(), // clone the wrapped pool
        None => return rocket,
    };

    let jm = EscalonJobsManager::new(Context(pool));
    let jm = jm
        .set_id(ConfigGetter::get_identity())
        .set_addr("0.0.0.0".parse::<IpAddr>().unwrap())
        .set_port(ConfigGetter::get_port().unwrap_or(65056))
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
