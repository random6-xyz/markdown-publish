const FILENAME: &str = "./markdown/sample_1.md";

#[cfg(test)]
mod test {
    use reqwest::blocking::Client;

    #[test]
    fn upload_file() {
        let client = Client::new();
        let file = File::open(FILENAME).expect("Can't open file");

        let response = client.post(format!("127.0.0.1:8000/upload/{}", FILENAME))
            .body(file)
            .send()
            .expect("Can't send request");

        assert!(response.status(), Status::new(200));
    }
}
