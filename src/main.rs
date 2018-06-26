extern crate actix_web;
extern crate rppal;
use actix_web::{http, server, App, Path, Responder,FromRequest,HttpRequest,fs};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::{Mutex, Arc};
use rppal::gpio::{Gpio, Mode, Level};
use rppal::system::DeviceInfo;



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
    let gpio = init_gpio();
    stop(&gpio);

	let mutex = Arc::new(Mutex::new("".to_string()));

	let clone = mutex.clone();
	thread::spawn(move || {
		let mut cmd = "stop".to_string();
        let mut lastcmd = Instant::now();
        loop {
        	{
        		let mut mutex = clone.lock().unwrap();
        		if *mutex != "" {
        			cmd = mutex.to_string();
                    *mutex = "".to_string();
                    lastcmd = Instant::now();

                    println!("new cmd: {}", cmd);
        		}
        	}

            if lastcmd.elapsed().subsec_millis() > 500 {
                cmd = "stop".to_string();
            }

            let mut job : Option<Box<Fn(&Gpio)>> = None;
            //let mut job = Fn(&Gpio);

            if cmd == "stop" {
                job = Some(Box::new(|gp| stop(&gp)));
            }

            if cmd == "forward" {
                job = Some(Box::new(|gp| {r_f(&gp); l_f(&gp);}));
            }

            if cmd == "backward" {
                job = Some(Box::new(|gp| {r_b(&gp); l_b(&gp);}));
            }

            if cmd == "left" {
               job = Some(Box::new(|gp| {r_b(&gp); l_f(&gp);}));
            }

            if cmd == "right" {
                job = Some(Box::new(|gp| {r_f(&gp); l_b(&gp);}));
            }

            if let Some(job) = job {
                job(&gpio);
            }

            //println!("Current bot job is {}!", cmd);
            thread::sleep(Duration::from_millis(10));
        }
    });


    server::new(move || {
		let mutex = mutex.clone();
		return App::with_state(AppState{cmd: mutex}) 
            .route("/cmd/{cmd}", http::Method::GET, cmd)
            .handler(
                "/",
                fs::StaticFiles::new("./static/").index_file("index.html"))
        })
        .bind("0.0.0.0:1337").unwrap()
        .run();


    println!("Prob never gets here");
}




fn init_gpio () -> rppal::gpio::Gpio {
    let mut gpio = Gpio::new().unwrap();
    gpio.set_mode(6, Mode::Output);
    gpio.set_mode(13, Mode::Output);
    gpio.set_mode(19, Mode::Output);
    gpio.set_mode(26, Mode::Output);
    return gpio
}

fn r_b(gpio: &Gpio) {
    gpio.write(6, Level::Low);
    gpio.write(13, Level::High);
}

fn l_f(gpio: &Gpio) {
    gpio.write(19, Level::High);
    gpio.write(26, Level::Low);
}

fn r_f(gpio: &Gpio) {
    gpio.write(6, Level::High);
    gpio.write(13, Level::Low);
}

fn l_b(gpio: &Gpio) {
    gpio.write(19, Level::Low);
    gpio.write(26, Level::High);
}

fn stop(gpio: &Gpio) {
    gpio.write(6, Level::Low);
    gpio.write(13, Level::Low);
    gpio.write(19, Level::Low);
    gpio.write(26, Level::Low);
}
