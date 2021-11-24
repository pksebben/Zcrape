/*
Here, we want to reach out to urls and receive status codes IOT cull based on certain
code classes (400 and 500, possibly others)
*/

#[derive(Debug)]
enum LinkStatus {
    Bad(RFail),
    Good,
}
type StatusMap = HashMap<String, LinkStatus>;

// testing wrapper to pass to test fn

fn print_pmfm(mb: MessageBuffer) {
    // cast to link buffer
    let lb = LinkBuffer::from_msgbuffer(mb);
    // pull urls from link buffer

    let results = procmult_fmap(lb);
    println!("Processing failure map...");
    for r in results {
        println!("{:?}\n", r);
    }
}

fn procmult_fmap(lb: LinkBuffer) -> StatusMap {
    let urls = lb.url_list();

    let (tx, rx) = mpsc::channel();

    let mut statmap = StatusMap::new();

    for u in urls {
        let btx = mpsc::Sender::clone(&tx);
        thread::spawn(move || {
            let resolved = (u.to_string(), get_code(u.to_string(), 0));
            btx.send(resolved).unwrap();
        });
    }

    for received in rx {
        statmap.insert(received.0.to_string(), received.1);
    }
    statmap
}

// this is a wrapper to bundle particular state with failure types
#[derive(Debug)]
enum RFail {
    Req(reqwest::Error),
    Timeout,
    Status(reqwest::StatusCode),
    Conn,
    Panic(reqwest::Error),
    Ignored(reqwest::Error),
    TimedOut,
}

// this layer is only here because I couldn't figure out why Rust complained about
// reqwest::Error not having the .is_request() fn when used elsewhere.
// that said, I changed versions since....
fn rqe_handler(e: reqwest::Error) -> RFail {
    if e.is_request() {
        println!("bad request: {}", e); // DNS failures occur here.  Can we get more specific?
        RFail::Req(e)
    } else if e.is_status() {
        println!("got status: {}", e); // here we want to bubble the status code up
        RFail::Status(
            e.status()
                .expect("unknown failure in reqwest::Error::Status"),
        )
    } else if e.is_connect() {
        println!("connection err: {}", e); // here we want to fail quietly and shelve the link
        RFail::Conn
    } else if e.is_timeout() {
        println!("request timeout"); // here we want to pause execution and wait.
        RFail::Timeout
    } else if e.is_builder() {
        println!("WE'RE ON A LIIIIINK TO NOOOWHEEERE");
        RFail::Ignored(e)
    } else {
        panic!("unknown err {} on {:?}\n\n{:#?}", e, e.url(), e)
    }
}

fn get_code(url: String, timeout: u32) -> LinkStatus {
    match check_status_code(url.to_string()) {
        Ok(data) => LinkStatus::Good,
        Err(e) => {
            if e.is_request() {
                println!("bad request: {}", e); // DNS failures occur here.  Can we get more specific?
                LinkStatus::Bad(RFail::Req(e))
            } else if e.is_status() {
                println!("got status: {}", e); // here we want to bubble the status code up
                LinkStatus::Bad(RFail::Status(
                    e.status()
                        .expect("unknown failure in reqwest::Error::Status"),
                ))
            } else if e.is_connect() {
                println!("connection err: {}", e); // here we want to fail quietly and shelve the link
                LinkStatus::Bad(RFail::Conn)
            } else if e.is_timeout() {
                if timeout < 10 {
                    println!("timeout retry: {}/10", timeout);
                    get_code(url, timeout + 1)
                } else {
                    println!("request timeout"); // here we want to pause execution and wait.
                    LinkStatus::Bad(RFail::Timeout)
                }
            } else if e.is_builder() {
                println!("WE'RE ON A LIIIIINK TO NOOOWHEEERE");
                LinkStatus::Bad(RFail::Ignored(e))
            } else {
                panic!("unknown err {} on {:?}\n\n{:#?}", e, e.url(), e)
            }
        }
    }
}

// load in a message buffer and check the urls for status codes
fn test_bulk_message_status_check() {
    let pattern: Vec<String> = std::env::args().collect();
    for file in &pattern[1..] {}
}

