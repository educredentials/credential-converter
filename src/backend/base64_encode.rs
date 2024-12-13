use num_traits::ops::bytes;
//use reqwest::Client;
//use reqwest::blocking::get;
use base64::{engine::general_purpose::STANDARD as Base64Engine, Engine};
use std::error::Error;
use std::io::{BufReader, Read, copy};
use std::fs::File;
use ureq::get;

/// Fetches an image from the given URL, encodes it in Base64, and returns the encoded string.
///
/// # Arguments
/// - `url`: The URL of the image to encode.
///
/// # Returns
/// - `Ok(String)`: The Base64-encoded string of the image if successful.
/// - `Err(Box<dyn Error>)`: An error if the fetch or encoding fails.
pub fn encode_image_from_url(url: &str) -> Result<String, Box<dyn Error>> {

    let filename = "downloaded_image.jpg";
    let resp = get(url).call().expect("Failed to download image");

    assert!(resp.has("Content-Length"));
    let len: usize = resp.header("Content-Length")
        .unwrap()
        .parse()?;

    let mut bytes: Vec<u8> = Vec::with_capacity(len);
    resp.into_reader()
        .take(10_000_000)
        .read_to_end(&mut bytes)?;

    // let mut file = File::create(filename).expect("Failed to create file");
    // copy(&mut resp.into_reader(), &mut file).expect("Failed to save image");

    // let mut reader = BufReader::new(file);
    // let mut image_bytes : Vec<u8> = Vec::new();
    // reader.read_to_end(&mut image_bytes).unwrap();
    // println!("{:#?}", image_bytes);


    // Encode the image bytes as a Base64 string
    let base64_string = Base64Engine.encode(&bytes);

    Ok(base64_string)
}


