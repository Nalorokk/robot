extern crate actix_web;
extern crate rppal;
use actix_web::{fs, http, server, App, FromRequest, HttpRequest, Path, Responder};
use rppal::gpio::{Gpio, Level, Mode};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

struct AppState {
    cmd: Arc<Mutex<String>>,
    power: Arc<AtomicUsize>,
}

fn on_cmd(req: HttpRequest<AppState>) -> impl Responder {
    let info = Path::<(String)>::extract(&req).unwrap();
    let cmd = info.into_inner();
    println!("Incoming cmd is {}!", &cmd);
    let mut curcmd = req.state().cmd.lock().unwrap();
    *curcmd = cmd;

    return format!("Incoming cmd is {}!", &curcmd);
}

fn on_power(req: HttpRequest<AppState>) -> impl Responder {
    let info = Path::<(usize)>::extract(&req).unwrap();
    let power = info.into_inner();
    println!("Incoming power is {}!", &power);

    let curpower = &req.state().power;
    curpower.store(power, Ordering::Relaxed);

    return format!("Incoming power is {}!", &power);
}

fn main() {
    println!("Bot startup");
    let gpio = init_gpio();
    stop(&gpio);

    let cmd_mutex = Arc::new(Mutex::new("".to_string()));
    let power_atomic = Arc::new(AtomicUsize::new(100));

    {
        let cmd_mutex = cmd_mutex.clone();
        let power_atomic = power_atomic.clone();

        thread::spawn(move || {
            let mut cmd = "stop".to_string();
            let mut lastcmd = Instant::now();
            let mut cpower: usize;
            loop {
                {
                    let mut cmd_mutex = cmd_mutex.lock().unwrap();
                    if *cmd_mutex != "" {
                        cmd = cmd_mutex.to_string();
                        *cmd_mutex = "".to_string();
                        lastcmd = Instant::now();

                        println!("new cmd: {}", cmd);
                    }

                    cpower = power_atomic.load(Ordering::Relaxed);
                }

                if lastcmd.elapsed().subsec_millis() > 500 {
                    cmd = "stop".to_string();
                }

                let mut job: Option<Box<Fn(&Gpio)>> = None;
                //let mut job = Fn(&Gpio);

                if cmd == "stop" {
                    job = Some(Box::new(|gp| stop(&gp)));
                }

                if cmd == "forward" {
                    job = Some(Box::new(|gp| {
                        r_f(&gp);
                        l_f(&gp);
                    }));
                }

                if cmd == "backward" {
                    job = Some(Box::new(|gp| {
                        r_b(&gp);
                        l_b(&gp);
                    }));
                }

                if cmd == "left" {
                    job = Some(Box::new(|gp| {
                        r_b(&gp);
                        l_f(&gp);
                    }));
                }

                if cmd == "right" {
                    job = Some(Box::new(|gp| {
                        r_f(&gp);
                        l_b(&gp);
                    }));
                }

                if let Some(job) = job {
                    if cpower >= 100 || cmd == "stop" {
                        job(&gpio);
                        thread::sleep(Duration::from_millis(10));
                    } else {
                        pwm(&gpio, 10, cpower, job);
                    }
                }
            }
        });
    }

    server::new(move || {
        let cmd_mutex = cmd_mutex.clone();
        let power_atomic = power_atomic.clone();
        return App::with_state(AppState {
            cmd: cmd_mutex,
            power: power_atomic,
        }).route("/cmd/power/{power}", http::Method::GET, on_power)
            .route("/cmd/{cmd}", http::Method::GET, on_cmd)
            .handler(
                "/",
                fs::StaticFiles::new("./static/").index_file("index.html"),
            );
    }).bind("0.0.0.0:1337")
        .unwrap()
        .run();

    println!("Prob never gets here");
}

fn init_gpio() -> rppal::gpio::Gpio {
    let mut gpio = Gpio::new().unwrap();
    gpio.set_mode(6, Mode::Output);
    gpio.set_mode(13, Mode::Output);
    gpio.set_mode(19, Mode::Output);
    gpio.set_mode(26, Mode::Output);
    return gpio;
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

fn pwm(gpio: &Gpio, millis: u32, prc: usize, f: Box<Fn(&Gpio)>) {
    let cycle = 1000.; // 1 KHZ PWM frequency
    let duty = prc as f32 * 0.01;

    let wcycle = cycle * duty;
    let scycle = cycle - wcycle;

    println!(
        "Cycle: {} us, duty: {}, wcycle: {}, scycle: {}",
        cycle, duty, wcycle, scycle
    );

    let start = Instant::now();
    while start.elapsed().subsec_millis() < millis {
        f(&gpio);
        thread::sleep(Duration::from_micros(wcycle as u64));
        stop(&gpio);
        thread::sleep(Duration::from_micros(scycle as u64));
    }
}
