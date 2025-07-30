#[allow(dead_code)]

use std::{io::Write, thread, time::{Duration, Instant}, u128};
use reqwest::blocking::{Response, Client};
use clap::{command, Parser};
use plotters::prelude::*;

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
    let client = Client::builder().timeout(Duration::from_secs(300)).build().unwrap_or(Client::new());

    let args = Args::parse();
    let mut failed_requests = 0u32;
    let mut response_times: Vec<Duration> = Vec::new();
    let mut wait_time = 0u128;

    wait_time += args.millis.unwrap_or(0);
    wait_time += args.seconds.unwrap_or(0) * 1000;
    wait_time += args.mins.unwrap_or(0) * 60 * 1000;
    wait_time += args.hours.unwrap_or(0) * 60 * 60 * 1000;

    for rep in 0..args.rep {
        delay_countdown(wait_time, rep + 1, args.rep);
                
        let start = Instant::now();
        let response = client.get(&args.url).send();
        let duration = start.elapsed();
        let status = match response {
            Ok(response) => {
                let status = response.status().as_u16();
                parse_response(response, status, &mut failed_requests, rep + 1);
                status
            }
            _ => {
                println!("Error fetching response from {}", args.url);
                failed_requests += 1;
                0u16
            },
        };
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
    plot(&response_times, min, max);
    (min, max, sum / response_times.len() as u128)
}

fn print_result(result: (u128, u128, u128)) -> String {
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

fn parse_response(response: Response, status: u16, failed_requests: &mut u32, step: u128) {
    if status < 200 || status >= 300 {
        *failed_requests += 1;
        println!("\r(Request {}) {}: {}", step, status, response.text().unwrap_or(String::from("Request did not send a response")));
    }
}

fn delay_countdown(wait_time: u128, step: u128, total: u128) {
    let seconds = (wait_time / 1000) as u64;
    print!("\rRequest {}/{}     ", step, total);
    std::io::stdout().flush().unwrap();
    if step == 1 {
        return;
    }
    for i in (1..=seconds).rev() {
        print!("\rRequest {}/{} in {} s     ", step, total, i);
        std::io::stdout().flush().unwrap();
        thread::sleep(Duration::from_secs(1));
    }
    thread::sleep(Duration::from_millis((wait_time % 1000) as u64));           
}

fn plot(values: &Vec<Duration>, min: u128, max: u128) {

    let mut data: Vec<f64> = values.iter().map(|d| d.as_millis() as f64).collect();
    data.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let quartiles = Quartiles::new(&data);

    let root = BitMapBackend::new("boxplot.png", (1000, 200)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .margin(30)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d((quartiles.values()[0] * 0.98) as f32..(quartiles.values()[4] * 1.02) as f32, -5.0f32..5.0f32).unwrap();

    chart.configure_mesh().x_desc("Millisekunden").y_labels(0).draw().unwrap();
   
    println!("{:?}", quartiles);
    let boxplot = Boxplot::new_horizontal(0f32, &quartiles);

    chart.draw_series(std::iter::once(boxplot)).unwrap();
}