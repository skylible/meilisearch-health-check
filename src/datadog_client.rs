use dogstatsd::{Client, Options};

pub fn send_histogram(stat: &str, val: &str) {
    // Define the options for the Dogstatsd client
    let options = Options {
        to_addr: get_address(),  // Assuming you have a function to get the address
        ..Default::default()  // Use default options for other parameters
    };

    // Create a new Dogstatsd client with the specified options
    let client = Client::new(options).unwrap();

    // Tags for the metric (assuming env:production as a tag)
    let tags = &["env:production"];

    // Send a histogram metric to the Dogstatsd server
    client.histogram(stat, val, tags).unwrap();
}

pub fn get_address() -> String {
    return String::from("localhost:7700"); // Change to real address
}