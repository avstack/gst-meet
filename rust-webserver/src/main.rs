use actix_web::{get, web, App, HttpServer, Responder, HttpRequest, HttpResponse};
mod repositories;
use redis::AsyncCommands;
use redis::ControlFlow;
use redis::PubSubCommands;
pub use repositories::AppState;
pub use repositories::RedisActor;
pub use repositories::InfoCommandGet;
pub use repositories::InfoCommandSet;
pub use repositories::InfoCommandDel;
pub use repositories::InfoCommandPublish;
pub use repositories::SetRoomInfo;
use std::env;
use std::thread;
use std::{collections::HashMap, pin::Pin, sync::RwLock};
use redis::{Client, aio::MultiplexedConnection};
use actix::prelude::*;
use actix::Message;
use libc::{kill, SIGTERM};
use serde_json::Error;
use nix::unistd::Pid;
use nix::sys::signal::{self, Signal};

impl RedisActor {
    pub async fn new(redis_url: String) -> Self {
        let client = Client::open(redis_url).unwrap();
        let (conn, call) = client.get_multiplexed_async_connection().await.unwrap();
        thread::spawn(move || {
            let mut con = client.get_connection().unwrap();
            let _ :() =  con.subscribe(&["sariska_channel_gstreamer"], |msg| {
                let ch = msg.get_channel_name();
                let payload: String = msg.get_payload().unwrap();
                let decoded: SetRoomInfo  = serde_json::from_str(&payload).unwrap();
                let hostname = env::var("MY_POD_NAME").unwrap_or("none".to_string());

                println!("{} hostname", hostname);
                if decoded.hostname != "" {
                    println!("{:?} subscribed", decoded);
                    if  hostname == decoded.hostname {
                        let my_int = decoded.process_id.parse::<i32>().unwrap();
                        unsafe {
                            println!(" killed process id {} ", my_int);
                            signal::kill(Pid::from_raw(my_int), Signal::SIGTERM).unwrap();
                        }
                    }
                }
                return ControlFlow::Continue;
            }).unwrap();
        });
        actix_rt::spawn(call);
        RedisActor { conn }
    }
}

impl Handler<InfoCommandGet> for RedisActor {
    type Result = ResponseFuture<Result<Option<String>, redis::RedisError>>;
    fn handle(&mut self, _msg: InfoCommandGet, _: &mut Self::Context) -> Self::Result {
        let mut con = self.conn.clone();
        let mut cmd = redis::cmd(&_msg.command);

        let fut = async move {
            cmd
                .arg(&_msg.arg)
                .query_async(&mut con)
                .await
        };
        Box::pin(fut)
    }
}

impl Handler<InfoCommandSet> for RedisActor {

    type Result = ResponseFuture<Result<Option<String>, redis::RedisError>>;

    fn handle(&mut self, _msg: InfoCommandSet, _: &mut Self::Context) ->  Self::Result  {
        let mut con = self.conn.clone();
        let mut cmd = redis::cmd(&_msg.command);
        let mut pipe = redis::pipe();
        let fut = async move {
            pipe.cmd("SET")
            .arg(_msg.arg.clone())
            .arg(_msg.arg2)
            .ignore()
            .cmd("EXPIRE")
            .arg(_msg.arg.clone())
            .arg(86400)
            .ignore();

            return pipe.query_async(&mut con).await; 
        };
        Box::pin(fut)
    }
}


impl Handler<InfoCommandDel> for RedisActor {
    
    type Result = ResponseFuture<Result<Option<String>, redis::RedisError>>;

    fn handle(&mut self, _msg: InfoCommandDel, _: &mut Self::Context) ->Self::Result {
        let mut con = self.conn.clone();
        let mut cmd = redis::cmd(&_msg.command);
        let mut pipe = redis::pipe();
        let fut = async move {
            pipe.cmd("DEL")
            .arg(_msg.arg)
            .ignore();

            return pipe.query_async(&mut con).await; 
        };
        Box::pin(fut)
    }
}



impl Handler<InfoCommandPublish> for RedisActor {
    
    type Result = ResponseFuture<Result<Option<String>, redis::RedisError>>;

    fn handle(&mut self, _msg: InfoCommandPublish, _: &mut Self::Context) ->Self::Result {
        println!("publish");
        let mut con = self.conn.clone();
        let mut cmd = redis::cmd(&_msg.command);
        let mut pipe = redis::pipe();
        let fut = async move {
            pipe.cmd("publish")
            .arg(_msg.channel)
            .arg(_msg.message)
            .ignore();
            return pipe.query_async(&mut con).await; 
        };
        Box::pin(fut)
    }
}

impl Actor for RedisActor {
    type Context = Context<Self>;
}

pub async fn get_health_status() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .body("Healthy!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let redis_url: String = env::var("REDIS_URL_GSTREAMER_PIPELINE").unwrap_or("none".to_string());
    let actor = RedisActor::new(redis_url).await;
    let addr = actor.start();
    HttpServer::new(move || {
        App::new()
            .app_data( 
                web::Data::new(RwLock::new(AppState {
                    map: HashMap::new(),
                    conn: addr.clone()
            })).clone())
            .service(web::resource("/user/startRecording").route(web::post().to(repositories::user_repository::start_recording)))
            .service(web::resource("/user/stopRecording").route(web::post().to(repositories::user_repository::stop_recording)))
            .service(web::resource("/healthz").route(web::get().to(get_health_status)))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
