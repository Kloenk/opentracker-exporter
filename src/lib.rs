use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::process::exit;
use std::vec::Vec;
use std::collections::HashMap;

/// thread library containing a thread pool
pub mod threads;

/// error library for error handling
pub mod error;

pub struct Config {
    /// verbosity level
    pub verbose: u8,

    /// address without http and without /stats?mode=everything
    pub url: String,

    /// port to listen on
    pub port: u16,

    /// interface to listen on
    pub interface: String,

    /// prefix for metrics
    pub prefix: String,

    /// number of thread in threadpool
    pub threads: usize,
}

impl Config {
    /// new creates a new instance with default values
    pub fn new() -> Self {
        Self {
            verbose: 0,
            url: String::from("localhost"),
            port: 9999,
            interface: String::from("0.0.0.0"),
            prefix: String::from("opentracker"),
            threads: 8,
        }
    }

    /// execute application
    pub fn run(mut self) -> Result<(), String> {
        if self.verbose >= 1 {
            println!("Debug1: using opentracker stats on {}", self.url);
            println!(
                "Debug1: listening for prometheus on {}:{} with {} threads",
                self.interface, self.port, self.threads
            );
            println!("Debug1: metrics are calle {}_*", self.prefix);
        }

        // create threadPool
        let mut thread_pool = threads::ThreadPool::new(self.threads).unwrap_or_else(|err| {
            eprintln!("Error creating threadPool: {}", err.to_string());
            exit(-2);
        });

        if self.verbose >= 2 {
            println!("Debug2: enabling threadPool verbose mode");
            thread_pool.set_verbose_mode(true);
        }

        // open port
        let listener = TcpListener::bind(format!("{}:{}", self.interface, self.port))
            .unwrap_or_else(|err| {
                eprintln!("could not bind to port: {}", err);
                exit(-3);
            });

        // handle connection
        for stream in listener.incoming() {
            let stream = match stream {
                Ok(stream) => stream,
                Err(err) => {
                    eprintln!("error createing stream: {}", err);
                    continue;
                }
            };

            let verbose = self.verbose;
            let url = self.url.clone();
            let prefix = self.prefix.clone();

            // move stream to thread
            thread_pool.execute(move || {
                handle(stream, verbose, url, &prefix);
            });
        }
        Ok(())
    }
}

/// function for processing of prometheus client
fn handle(mut stream: TcpStream, verbose: u8, url: String, prefix: &str) {
    if verbose >= 3 {
        println!("Debug3: Connection established!");
    }
    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();

    //println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    let content = get_content(url, prefix);

    stream.write(format!(
        "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: {}\r\nContent-Type: text/plain; version=0.0.4\r\nDate: {}\r\n\r\n{}",
        content.len(),
        httpdate::fmt_http_date(std::time::SystemTime::now()),
        content
    ).as_bytes()).unwrap(); //fixme;
    stream.flush();
}

// HTTP/1.1 200 OK
// Connection: Keep-Alive
// Content-Length: {}
// Content-Type: text/plain; version=0.0.4
// Date: Mon, 24 Jun 2019 17:19:  GMT

#[derive(Debug)]
struct Torrents {
    mutex: usize,
    iterator: usize,
}

impl Torrents {
    pub fn new() -> Self {
        Self {
            mutex: 0,
            iterator: 0,
        }
    }
}

#[derive(Debug)]
struct Connections {
    tcp_accept: usize,
    tcp_announce: usize,
    tcp_scrape: usize,
    udp_overall: usize,
    udp_connect: usize,
    udp_announce: usize,
    udp_scrape: usize,
    udp_missmatch: usize,
    livesync: isize,
}

impl Connections {
    pub fn new() -> Self {
        Self {
            tcp_accept: 0,
            tcp_announce: 0,
            tcp_scrape: 0,
            udp_overall: 0,
            udp_connect: 0,
            udp_announce: 0,
            udp_scrape: 0,
            udp_missmatch: 0,
            livesync: 0,
        }
    }
}

#[derive(Debug)]
struct Everything {
    tracker_id: usize,
    uptime: usize,
    torrents: Torrents,
    peers: usize,
    seeds: usize,
    completed: usize,
    connections: Connections,
    http_error: HashMap<String, usize>,
    mutex_stall: usize,
}

impl Everything {
    pub fn new() -> Self {
        Self {
            tracker_id: 0,
            uptime: 0,
            torrents: Torrents::new(),
            peers: 0,
            seeds: 0,
            completed: 0,
            connections: Connections::new(),
            http_error: HashMap::new(),
            mutex_stall: 0,
        }
    }
    pub fn get_string(&self, prefix: &str) -> String {
        let mut ret = String::new();
        ret.push_str(&format!(r#"# HELP {}_uptime uptime of the tracker
# TYPE {}_uptime gauge
{}_uptime{{tracker="{}"}} {}"#, prefix, prefix, prefix, self.tracker_id, self.uptime));
        ret.push_str(&format!(r#"
# HELP {}_torrents counts torrents on server
# TYPE {}_torrents gauge
{}_torrents{{tracker="{}",type="mutex"}} {}
{}_torrents{{tracker="{}",type="iterator"}} {}"#, prefix, prefix, prefix, self.tracker_id,
        self.torrents.mutex, prefix, self.tracker_id, self.torrents.iterator));
        ret.push_str(&format!(r#"
# HELP {}_count count for varios things
# TYPE {}_count gauge
{}_count{{tracker="{}",type="peers"}} {}
{}_count{{tracker="{}",type="seeds"}} {}
{}_count{{tracker="{}",type="completed"}} {}
{}_count{{tracker="{}",type="mutex_stall"}} {}"#, prefix, prefix, prefix, self.tracker_id, self.peers,
        prefix, self.tracker_id, self.seeds,
        prefix, self.tracker_id, self.completed,
        prefix, self.tracker_id, self.mutex_stall));
        ret.push_str(&format!(r#"
# HELP {}_connections to the tracker
# TYPE {}_connections gauge
{}_connections{{tracker="{}",protocol="tcp",type="accept"}} {}
{}_connections{{tracker="{}",protocol="tcp",type="announce"}} {}
{}_connections{{tracker="{}",protocol="tcp",type="scrape"}} {}
{}_connections{{tracker="{}",protocol="udp",type="overall"}} {}
{}_connections{{tracker="{}",protocol="udp",type="connect"}} {}
{}_connections{{tracker="{}",protocol="udp",type="announce"}} {}
{}_connections{{tracker="{}",protocol="udp",type="scrape"}} {}
{}_connections{{tracker="{}",protocol="udp",type="missmatch"}} {}
{}_connections{{tracker="{}",type="livesync"}} {}"#, prefix, prefix,
        prefix, self.tracker_id, self.connections.tcp_accept,
        prefix, self.tracker_id, self.connections.tcp_announce,
        prefix, self.tracker_id, self.connections.tcp_scrape,
        prefix, self.tracker_id, self.connections.udp_overall,
        prefix, self.tracker_id, self.connections.udp_connect,
        prefix, self.tracker_id, self.connections.udp_announce,
        prefix, self.tracker_id, self.connections.udp_scrape,
        prefix, self.tracker_id, self.connections.udp_missmatch,
        prefix, self.tracker_id, self.connections.livesync));

        // http codes
        ret.push_str(&format!(r#"
# HELP {}_http_codes http error code count
# TYPE {}_http_codes gauge
"#, prefix, prefix));

        for (key, value) in &self.http_error {
            ret.push_str(&format!(r#"{}_http_codes{{tracker="{}",code="{}"}} {}
"#, prefix, self.tracker_id, key, value));
        }

        ret.push_str(&format!(r#"
# opentracker/export_prometheus {}
"#, env!("CARGO_PKG_VERSION") ));
        ret
    }
}

fn get_content(url: String, prefix: &str) -> String {
    let mut tracker_data = Everything::new();
    // get mode=everything
    if let Ok(mut stream) = TcpStream::connect(&url) {
        stream.write(format!(
            "GET /stats?mode=everything HTTP/1.1\r\nHost: {}\r\nUser-Agent: opentracker-exporter/{}\r\nAccept: text/plain\r\n\r\n",
            url,
            env!("CARGO_PKG_VERSION")
        ).as_bytes()).unwrap(); //FIXME: unwrap
        stream.flush();

        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).unwrap();

        let buffer = String::from_utf8_lossy(buffer.as_slice());
        let (_, buffer) = buffer.split_at(buffer.find("\r\n\r\n").unwrap());
        let buffer = buffer.trim().to_string();

        use xml::reader::{EventReader, XmlEvent};

        let parser = xml::reader::EventReader::from_str(&buffer);
        let mut outer_name = String::new();
        let mut inner_name = String::new();
        let mut http_code = String::new();

        let mut depth = 0;
        for e in parser {
            match e {
                Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                    //println!("{}+{}", indent(depth), name);
                    depth += 1;

                    let name = name.to_string();

                    if name == "count" && outer_name == "http_error" {
                        if attributes.len() == 1 {
                            http_code = attributes[0].value.to_string();
                        }
                    }

                    if name == "count" || name == "accept" || name == "announce" || name == "scrape" || name == "overall" || name == "connect" || name == "missmatch" {
                        inner_name = name;
                    } else {
                        outer_name = name;
                    }
                },
                Ok(XmlEvent::EndElement { name }) => {
                    depth -= 1;
                    //println!("{}-{}", indent(depth), name);
                },
                Ok(XmlEvent::Characters(data)) => {
                    //println!("{}chars: {}", indent(depth), data);
                    if outer_name == "tracker_id" {
                        tracker_data.tracker_id = data.parse().unwrap_or_else(|_| 0);
                    } else if outer_name == "uptime" {
                        tracker_data.uptime = data.parse().unwrap_or_else(|_| 0);
                    } else if outer_name == "count_mutex" {
                        tracker_data.torrents.mutex = data.parse().unwrap_or_else(|_| 0);
                    } else if outer_name == "count_iterator" {
                        tracker_data.torrents.iterator = data.parse().unwrap_or_else(|_| 0);
                    } else if outer_name == "peers" && inner_name == "count" {
                        tracker_data.peers = data.parse().unwrap_or_else(|_| 0);
                    } else if outer_name == "seeds" && inner_name == "count" {
                        tracker_data.seeds = data.parse().unwrap_or_else(|_| 0);
                    } else if outer_name == "completed" && inner_name == "count" {
                        tracker_data.completed = data.parse().unwrap_or_else(|_| 0);
                    } else if outer_name == "mutex_stall" && inner_name == "count" {
                        tracker_data.mutex_stall = data.parse().unwrap_or_else(|_| 0);
                    } else if outer_name == "tcp" && inner_name == "accept" {
                        tracker_data.connections.tcp_accept = data.parse().unwrap_or_else(|_| 0);
                    } else if outer_name == "tcp" && inner_name == "announce" {
                        tracker_data.connections.tcp_announce = data.parse().unwrap_or_else(|_| 0);
                    } else if outer_name == "tcp" && inner_name == "scrape" {
                        tracker_data.connections.tcp_scrape = data.parse().unwrap_or_else(|_| 0);
                    } else if outer_name == "udp" && inner_name == "overall" {
                        tracker_data.connections.udp_overall = data.parse().unwrap_or_else(|_| 0);
                    } else if outer_name == "udp" && inner_name == "connect" {
                        tracker_data.connections.udp_connect = data.parse().unwrap_or_else(|_| 0);
                    } else if outer_name == "udp" && inner_name == "announce" {
                        tracker_data.connections.udp_announce = data.parse().unwrap_or_else(|_| 0);
                    } else if outer_name == "udp" && inner_name == "scrape" {
                        tracker_data.connections.udp_scrape = data.parse().unwrap_or_else(|_| 0);
                    } else if outer_name == "udp" && inner_name == "missmatch" {
                        tracker_data.connections.udp_missmatch = data.parse().unwrap_or_else(|_| 0);
                    } else if outer_name == "http_error" && inner_name == "count" {
                        tracker_data.http_error.insert(http_code.clone(), data.parse().unwrap_or_else(|_| 0));
                    }
                },
                _ => {},
            }
        }

    }
    tracker_data.get_string(prefix)
}

fn indent(size: usize) -> String {
    const INDENT: &'static str = "    ";
    (0..size).map(|_| INDENT)
             .fold(String::with_capacity(size*INDENT.len()), |r, s| r + s)
}

// GET /stats?mode=everything HTTP/1.1
// Host: tracker.yoshi210.com:1337
// User-Agent: curl/7.64.0
// Accept: */*


