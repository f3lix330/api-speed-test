use std::{io::Write, thread, time::{Duration, Instant}, u128};

use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    url: String,
    #[arg(short, long, default_value_t = 1)]
    rep: u128,
    #[arg(long)]
    millis: Option<u128>,
    #[arg(short, long)]
    seconds: Option<u128>,
    #[arg(long)]
    mins: Option<u128>,
    #[arg(long)]
    hours: Option<u128>
}

fn main() {
    let args = Args::parse();
    let mut failed_requests = 0;
    let mut response_times: Vec<Duration> = Vec::new();
    let mut wait_time = 0u128;

    wait_time += args.millis.unwrap_or(0);
    wait_time += args.seconds.unwrap_or(0) * 1000;
    wait_time += args.mins.unwrap_or(0) * 60 * 1000;
    wait_time += args.hours.unwrap_or(0) * 60 * 60 * 1000;

    for rep in 0..args.rep {
        if rep > 0 {
            if wait_time > 1000 {
                let seconds = (wait_time / 1000) as u64;
                for i in (0..=seconds).rev() {
                    print!("\r({}/{}) {} ", rep + 1, args.rep, i);
                    std::io::stdout().flush().unwrap();
                    thread::sleep(Duration::from_secs(1));
                }
                thread::sleep(Duration::from_millis((wait_time % 1000) as u64));
            } else if wait_time > 0 {
                thread::sleep(Duration::from_millis(wait_time as u64));
            }
        } else {
            print!("\r({}/{}) {} ", rep + 1, args.rep, 0);
            std::io::stdout().flush().unwrap();
        }
        
        let start = Instant::now();
        let status = match reqwest::blocking::get(&args.url) {
            Ok(response) => {
                let status = response.status().as_u16();
                if status < 200 || status >= 300 {
                    println!("\r(Request {}){}: {}",rep + 1, status, response.text().unwrap_or(String::from("Request unsuccessful")));
                }
                status
            }
            _ => {
                println!("Error fetching response from {}", args.url);
                failed_requests += 1;
                0u16
            },
        };
        let duration = start.elapsed();
        if status >= 200 && status < 300 {
            response_times.push(duration);
        }
    }
    if response_times.len() < 1 {
        println!("\rThere was no successful request...");
    } else {
        println!("\rFailed requests: {failed_requests} ");
        println!("{}", print_result(calc_stats(response_times)));
    }
    
}

fn calc_stats(response_times: Vec<Duration>) -> (u128, u128, u128) {
    let mut max = 0u128;
    let mut min = u128::MAX;
    let mut sum = 0u128;
    for time in response_times.iter() {
        let millis = time.as_millis();
        if millis < min {
            min = millis;
        }
        if millis > max {
            max = millis;
        }
        sum += millis;
    }
    (min, max, sum / response_times.len() as u128)
}

fn print_result(result: (u128, u128, u128)) -> String{
    let (min, max, avg) = result;
    let mut  text = String::new();
    if min < 1000 {
        text.push_str(format!("Min: {min} ms,").as_str());
    } else {
        text.push_str(format!("Min: {} s,", min as f32 / 1000.0).as_str());
    }

    if max < 1000 {
        text.push_str(format!(" Max: {max} ms,").as_str());
    } else {
        text.push_str(format!(" Max: {} s,", max as f32 / 1000.0).as_str());
    }

    if avg < 1000 {
        text.push_str(format!(" Avg: {avg} ms").as_str());
    } else {
        text.push_str(format!(" Avg: {} s", avg as f32 / 1000.0).as_str());
    }

    text
}