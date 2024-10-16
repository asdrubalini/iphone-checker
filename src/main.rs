use std::{
    collections::HashMap,
    io::{BufReader, Cursor},
    thread::sleep,
    time::Duration,
};

use reqwest::blocking::Client;
use rodio::{Decoder, OutputStream, Sink, Source};

const MODELS: [&str; 4] = ["MYWV3QL/A", "MYWW3QL/A", "MYWY3QL/A", "MYWX3QL/A"];
const STORE: &str = "R667"; // Piazza Liberty
const API_ENDPOINT: &str = "https://www.apple.com/it/shop/fulfillment-messages";

const AUDIO_DATA: &[u8] = include_bytes!("../sound.mp3");

fn apple_check_availability() -> anyhow::Result<HashMap<String, bool>> {
    let mut params = vec![
        ("searchNearby".to_string(), "false".to_string()),
        ("little".to_string(), "false".to_string()),
        ("purchaseOption".to_string(), "fullPrice".to_string()),
        ("fts".to_string(), "true".to_string()),
        ("store".to_string(), STORE.to_string()),
    ];

    params.extend(
        MODELS
            .iter()
            .enumerate()
            .map(|(i, model)| (format!("parts.{i}"), model.to_string())),
    );

    let client = Client::new();

    let response = client.get(API_ENDPOINT).query(&params).send()?.text()?;
    let j: serde_json::Value = serde_json::from_str(&response)?;

    let store = &j["body"]["content"]["pickupMessage"]["stores"][0];
    let parts = &store["partsAvailability"].as_object().unwrap();

    let m = parts
        .iter()
        .map(|(part, info)| {
            let buyability = &info["buyability"];

            let is_buyable = buyability["isBuyable"].as_bool().unwrap();
            let reason = buyability["reason"].as_str().unwrap();

            let status = is_buyable || reason != "NOT_AVAILABLE_FOR_PICKUP";

            (part.to_string(), status)
        })
        .collect();

    Ok(m)
}

fn play_warning() {
    // Get an output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    // shenanigans needed for BufReader since we're not reading as the sound bytes are already
    // in the executable
    let cursor = Cursor::new(AUDIO_DATA);

    // Load a sound from a file, using a path relative to Cargo.toml
    let file = BufReader::new(cursor);
    // Decode that sound file into a source
    let source = Decoder::new(file).unwrap();

    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(source);

    // Play the sound directly on the device
    sink.sleep_until_end();
}

fn main() -> anyhow::Result<()> {
    play_warning();

    // loop {
    //     // dont crash
    //     let a = match apple_check_availability() {
    //         Ok(a) => a,
    //         Err(err) => {
    //             println!("Got an error: {err:?}");

    //             sleep(Duration::from_secs(15));
    //             continue;
    //         }
    //     };

    //     for (model, availability) in a {
    //         if availability {
    //             println!("Model {model} is available!!");

    //             for _ in 0..25 {
    //                 play_warning();
    //                 sleep(Duration::from_secs(3));
    //             }
    //         }
    //     }

    //     sleep(Duration::from_secs(15));
    // }

    Ok(())
}
