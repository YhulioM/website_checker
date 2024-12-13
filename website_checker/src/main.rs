use ureq;
use std::thread;
use std::sync::mpsc;
use std::fs::File;
use chrono::{Duration, DateTime, Utc};
use std::io::{BufRead, BufReader};

// File path of txt file containing website list
const FILE_PATH: &str = "data/websites.txt";
// Configurable timeout duration for each request
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

struct WebsiteStatus {
    url: String,
    status: Result<u16, String>,
    response_time: Duration,
    timestamp: DateTime<Utc>,
}

impl WebsiteStatus {
    fn new(url: String, status: Result<u16, String>, response_time: i64) -> WebsiteStatus {
        WebsiteStatus {
            url: url,
            status: status,
            response_time: Duration::milliseconds(response_time),
            timestamp: Utc::now(),
        }
    }

    fn display(&self) {
        match &self.status {
            Ok(status) => {
                println!(
                "For {} = status: {}, response time: {} ms, timestamp: {}",
                self.url, status, self.response_time.num_milliseconds(), self.timestamp
                );
            }
            Err(error) => {
                println!("Error: {}", error);
            }
        }
        
    }   
}

 fn collect_status(url: &str) -> Result<(u16, i64, Option<String>), ureq::Error>
    {
        let agent = ureq::AgentBuilder::new().timeout_connect(TIMEOUT).timeout_read(TIMEOUT).build();

        let start_time = std::time::Instant::now();
        let response = agent.get(url).call();
        let duration = start_time.elapsed().as_millis();

        match response {
            Ok(resp) => Ok((resp.status(), duration.try_into().unwrap(), None)),
            Err(ureq::Error::Status(code, _)) => Ok((code, duration.try_into().unwrap(), Some("HTTP error".to_string()))),
            Err(e) => Ok((0, duration.try_into().unwrap(), Some(format!("Other error: {}", e)))),
        }

    }

fn main() {
    // Open the text file that contains the list of website URLs
    let file = File::open(FILE_PATH).expect("Failed to open file");
    let reader = BufReader::new(file);

    // Store the URLs into a vector
    let urls: Vec<String> = reader.lines().filter_map(Result::ok).collect();

    // Create channel for thread data access
    let (sender,receiver) = mpsc::channel();

    // Vector to store thread handles
    let mut handles = Vec::new();

    // Create threads for each website URL
    for url in urls {
        let sender = sender.clone();
        let handle = thread::spawn(move || {
                // Access the URL   
                if let Ok((status, response_time, error)) = collect_status(&url) {
                    let website = WebsiteStatus::new(url,Ok(status),response_time); // create website struct
                    website.display();
                   
                    sender.send(error).unwrap();   
                }
            });
        handles.push(handle);
    }
    drop(sender);

    for result in receiver {
        println!("Errors: {:?}", result);
    }

     for handle in handles {
        handle.join().unwrap();
    } 
}
