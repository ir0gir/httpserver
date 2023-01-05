use reqwest::blocking;

pub fn get_ext_ip() -> String {
    let uri = "https://myexternalip.com/raw";
    match blocking::get(uri) {
        Ok(response) => { response.text().unwrap() }
        Err(_) => { String::from("0.0.0.0") }
    }
}