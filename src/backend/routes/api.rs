use axum::{
    // routing::post,
    // Router,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
//use config::Value;
use serde_json::Value;

use crate::backend::base64_encode::decode_json;
use crate::backend::headless_cli::load_files_apply_transformations;
use crate::state::{AppState, Mapping};
use std::{fs::File, io::Write, path::Path};
use tokio::fs;

pub async fn api(Json(input_json): Json<Value>) -> Result<Response, (StatusCode, String)> {
    //test the input types for this API
    // with JSON body: {
    //     "From": {"Name": "OB", "Version": "3.0"},
    //     "To": {"Name": "elm", "Version": "3.2"},
    //     "Parameters": { "PreferredLanguages": ["en", "sv"]},
    //     "Content": "Base 64 encoded content in From format"
    // }

    // Handle the file upload
    let mut _input_file_path = String::new();
    let mut _mapping_file_name = String::new();
    let mut _mapping_type = Mapping::default();

    // Create directories for uploads and outputs if they don't exist
    let upload_dir = "uploads";
    let output_dir = "outputs";
    let file_name = "export_file";
    fs::create_dir_all(upload_dir).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create upload directory".to_string(),
        )
    })?;
    fs::create_dir_all(output_dir).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create output directory".to_string(),
        )
    })?;

    match input_json
        .get("From")
        .and_then(|v| v.get("Name"))
        .and_then(|v| v.as_str())
    {
        Some("OB") => {
            _mapping_file_name = "json/mapping/custom_mapping_OBv3_ELM_latest.json".to_string();
            _mapping_type = Mapping::OBv3ToELM;
        }
        Some("ELM") => {
            _mapping_file_name = "json/mapping/custom_mapping_ELM_OBv3_latest.json".to_string();
            _mapping_type = Mapping::ELMToOBv3;
        }
        Some(value) => return Err((StatusCode::BAD_REQUEST, format!("Invalid translation value: {}", value))),
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid translation value: no key found".to_string(),
            ))
        }
    }

    match input_json.get("Content").and_then(|v| v.as_str()) {
        Some(value) => {
            _input_file_path = format!("{}/{}", upload_dir, file_name);
            let mut file = File::create(&_input_file_path)
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create file".to_string()))?;
            let data = decode_json(value).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to read file data".to_string(),
                )
            })?;
            file.write_all(&data)
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to write to file".to_string()))?;
        }
        None => return Err((StatusCode::BAD_REQUEST, "Invalid data value: no key found".to_string())),
    }

    // Define the output file path
    let output_file_name = format!(
        "translated_{}",
        Path::new(&_input_file_path).file_name().unwrap().to_str().unwrap()
    );
    let output_file_path = format!("{}/{}", output_dir, output_file_name);

    // start mapping based on the input form the API
    // 1 create a state needed for the mapping tool
    // 2 load all hte state elements needed for mapping

    let mut state = AppState::default();
    state.input_path = _input_file_path;
    state.output_path = output_file_path.clone();
    state.mapping_path = _mapping_file_name;
    state.mapping = _mapping_type;

    load_files_apply_transformations(&mut state);

    // Return the translated file as a response
    // 1 load file from fs into mem
    // 2 remove file from fs
    // 3 push mem to http output

    let output_file = fs::read(&output_file_path).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read output file".to_string(),
        )
    })?;
    fs::remove_file(state.input_path).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to remove output file".to_string(),
        )
    })?;
    fs::remove_file(state.output_path).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to remove output file".to_string(),
        )
    })?;

    // Set the headers, including content disposition for download
    let mut headers = HeaderMap::new();
    // For better integration into EDCI change the output_file_name from *.json to *.jsonld
    let mut long_output_file_name = output_file_name;
    long_output_file_name.push_str("ld");
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", long_output_file_name)).unwrap(),
    );

    // Return the file content along with the appropriate headers
    Ok((headers, output_file).into_response())

    // Ok(output_file.into_response())
}
