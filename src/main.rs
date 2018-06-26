extern crate actix_web;
use actix_web::{http, server, App, Path, Responder,FromRequest,HttpRequest};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::{Mutex, Arc};



struct AppState {
    cmd: Arc<Mutex<String>>
}


fn cmd(req: HttpRequest<AppState>) -> impl Responder {
	let info = Path::<(String)>::extract(&req).unwrap();
	let cmd = info.into_inner();
	println!("Incoming cmd is {}!", &cmd);
	let mut curcmd = req.state().cmd.lock().unwrap();
	*curcmd = cmd;

    return format!("Incoming cmd is {}!", &curcmd);
}

fn main() {
	let mutex = Arc::new(Mutex::new("".to_string()));

	let clone = mutex.clone();
	thread::spawn(move || {
		let mut cmd = "stop".to_string();
        let mut lastcmd = Instant::now();
        loop {
        	{
        		let mut mutex = clone.lock().unwrap();
        		if(*mutex != "") {
        			cmd = mutex.to_string();
                    *mutex = "".to_string();
                    lastcmd = Instant::now();

                    println!("new cmd: {}", cmd);
        		}
        	}

            if(lastcmd.elapsed().as_secs() > 5) {
                cmd = "stop".to_string();
            }

            println!("Current bot job is {}!", cmd);
            thread::sleep(Duration::from_secs(1));
        }
    });


    server::new(move || {
		let mutex = mutex.clone();
		return App::with_state(AppState{cmd: mutex}) 
            .route("/cmd/{cmd}", http::Method::GET, cmd)
        })
        .bind("127.0.0.1:1337").unwrap()
        .run();


    println!("Prob never gets here");
}