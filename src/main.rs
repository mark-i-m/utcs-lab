//! A program to find the UTCS lab machine with the lowest load
//!
//! Inspired by https://github.com/claysmalley/cshosts

#![feature(lookup_host, ip_addr)]

extern crate regex;

use std::io::prelude::*;
use std::net::{TcpStream, lookup_host, SocketAddrV4, IpAddr};
use std::collections::BinaryHeap;
use std::cmp::Ordering;

use regex::Regex;

/// The address of the webapp
const URL: &'static str = "apps.cs.utexas.edu";

/// A struct to represent a single host. Orderable by load.
#[allow(dead_code)]
struct Host {
    host: String,
    up: bool,
    uptime: String,
    users: usize,
    load: f64,
}

impl PartialEq for Host {
    fn eq(&self, other: &Host) -> bool {
        self.load == other.load
    }
}

impl Eq for Host { }

impl PartialOrd for Host {
    fn partial_cmp(&self, other: &Host) -> Option<Ordering> {
        if self.eq(other) {
            Some(Ordering::Equal)
        } else if self.load > other.load { // lower load => higher priority
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }
}

impl Ord for Host {
    fn cmp(&self, other: &Host) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// Make a GET request to get the raw HTML of the Unix Lab page.
fn get_utcs_html() -> String {
    // DNS query
    let addr = lookup_host(URL)
        .ok()
        .expect("Unable to complete DNS query")
        .last()
        .unwrap()
        .ok()
        .expect("Unable to complete DNS query")
        .ip();

    let sock_addr = match addr {
        IpAddr::V4(v4) => SocketAddrV4::new(v4, 80),
        _ => panic!("Could not complete DNS query"),
    };

    // connect
    // println!("Connecting to server at {}", sock_addr);
    let mut cn = TcpStream::connect(sock_addr).unwrap();

    // send GET request
    let req = format!(concat!("GET /unixlabstatus/ HTTP/1.1\r\n",
                              "Host: {}\r\n",
                              "\r\n\r\n"), URL);
    //println!("Sending request:\n{}", req);
    cn.write(req.as_bytes())
        .ok()
        .expect("Unable to make GET request");

    // receive response
    let mut html = String::new();
    cn.read_to_string(&mut html)
        .ok()
        .expect("Unable to read response to GET");
    
    html
}

/// Parse HTML into a PriorityQueue
fn get_queue(html: String) -> Box<BinaryHeap<Host>> {
    let mut heap = Box::new(BinaryHeap::new());

    let re = Regex::new(concat!(
        r"(?ms)<tr>.",
        r"<td style=.background-color: (yellow|white); text-align:  ",
        r"(left|center|right);.>(?P<host>[\w-]+)</td>.",
        r"<td style=.background-color: (yellow|white); text-align:  ",
        r"(left|center|right);.>(?P<ud>[\w-]+)</td>.",
        r"<td style=.background-color: (yellow|white); text-align:  ",
        r"(left|center|right);.>(?P<uptime>[\d\+:,]+)</td>.",
        r"<td style=.background-color: (yellow|white); text-align:  ",
        r"(left|center|right);.>(?P<users>\d+)</td>.",
        r"<td style=.background-color: (yellow|white); text-align:  ",
        r"(left|center|right);.>(?P<load>[\d\.]+)</td>.",
        r"</tr>"
        ))
        .unwrap();

    for caps in re.captures_iter(&*html) {
        let host = caps.name("host").unwrap().to_string();
        let up = caps.name("ud").unwrap() == "up";
        let uptime = caps.name("uptime").unwrap().to_string();
        let users: usize = caps.name("users")
            .unwrap()
            .parse()
            .ok()
            .expect("Could not parse user count");

        let load: f64 = caps.name("load")
            .unwrap()
            .parse()
            .ok()
            .expect("could not parse load");

        //println!("host: {:?}", host);
        heap.push(Host {
            host: host,
            up: up,
            uptime: uptime,
            users: users,
            load: load,
        }
        );
    }

    heap
}

fn main() {
    let mut heap = get_queue(get_utcs_html());

    // Get the least used element
    print!("{}", heap.pop().unwrap().host);
}
