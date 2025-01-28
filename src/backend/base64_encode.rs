use base64::{engine::general_purpose::STANDARD as Base64Engine, Engine};
use regex::Regex;
use serde_json::Value;
use std::error::Error;
use std::io::Read;
use std::path::Path;
use ureq::get;

/// Fetches an image from the given URL, encodes it in Base64, and returns the encoded string.
///
/// # Arguments
/// - `url`: The URL of the image to encode.
///
/// # Returns
/// - `Ok(String)`: The Base64-encoded string of the image if successful.
/// - `Err(Box<dyn Error>)`: An error if the fetch or encoding fails.
fn encode_image_from_url(url: &str) -> Result<String, Box<dyn Error>> {
    let resp = get(url).call().expect("Failed to download image");

    assert!(resp.has("Content-Length"));
    let len: usize = resp.header("Content-Length").unwrap().parse()?;

    let mut bytes: Vec<u8> = Vec::with_capacity(len);
    resp.into_reader().take(10_000_000).read_to_end(&mut bytes)?;

    // Encode the image bytes as a Base64 string
    let base64_string = Base64Engine.encode(&bytes);

    Ok(base64_string)
}

/// Creates contentType object based on input type in string
///
/// # Arguments
/// - `content_type`: The a type that is than rewritten into a serde Value object.
///
/// # Returns
/// - `Ok(Value)`: The content value Object in ELM format if successful.
/// - `Err(Box<dyn Error>)`: An error if the fetch or encoding fails.
fn set_content_type(content_type: &str) -> Result<Value, Box<dyn Error>> {
    let content_json = r#"
    {
        "id": "http://publications.europa.eu/resource/authority/file-type/PNG",
        "type": "Concept",
        "inScheme": {
            "id": "http://publications.europa.eu/resource/authority/file-type",
            "type": "ConceptScheme"
        },
        "prefLabel": {
            "en": ["PNG"]
        },
        "notation": "file-type"
    }
    "#;

    let mut parsed_content_type_json: Value = serde_json::from_str(content_json).unwrap();

    match content_type {
        "PNG" => {
            parsed_content_type_json["id"] =
                Value::String("http://publications.europa.eu/resource/authority/file-type/PNG".to_string());
            parsed_content_type_json["prefLabel"]["en"][0] = Value::String("PNG".to_string());
        }
        "JPG" | "JPEG" => {
            parsed_content_type_json["id"] =
                Value::String("http://publications.europa.eu/resource/authority/file-type/JPEG".to_string());
            parsed_content_type_json["prefLabel"]["en"][0] = Value::String("JPG".to_string());
        }
        "SVG" => {
            parsed_content_type_json["id"] =
                Value::String("http://publications.europa.eu/resource/authority/file-type/SVG".to_string());
            parsed_content_type_json["prefLabel"]["en"][0] = Value::String("SVG".to_string());
        }
        _ => {
            // Handle other cases
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "This file Type is not exporteswent wrong!",
            )));
        }
    }
    Ok(parsed_content_type_json)
}

/// Creates contentEncodingType object based on input type in string
///
/// # Arguments
/// - `content_encoding_type`: The a type that is than rewritten into a serde Value object.
///
/// # Returns
/// - `Ok(Value)`: The content value Object in ELM format if successful.
/// - `Err(Box<dyn Error>)`: An error if the fetch or encoding fails.
fn set_content_enconding_type(content_encoding_type: &str) -> Result<Value, Box<dyn Error>> {
    let template_json = r#"
  {
    "id": "http://data.europa.eu/snb/encoding/6146cde7dd",
    "type": "Concept",
    "inScheme": {
      "id": "http://data.europa.eu/snb/encoding/25831c2",
      "type": "ConceptScheme"
    },
    "prefLabel": {
      "en": ["base64"]
    }
  }
  "#;

    let mut parsed_content_encoding_type_json: Value = serde_json::from_str(template_json).unwrap();

    match content_encoding_type {
        "base64" => {
            parsed_content_encoding_type_json["id"] =
                Value::String("http://data.europa.eu/snb/encoding/6146cde7dd".to_string());
            parsed_content_encoding_type_json["prefLabel"]["en"][0] = Value::String("base64".to_string());
        }
        _ => {
            // Handle other cases
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "This content encoding Type is not supported!",
            )));
        }
    }
    Ok(parsed_content_encoding_type_json)
}

/// Creates language object based on input type in string
///
/// # Arguments
/// - `content_encoding_type`: The a type that is than rewritten into a serde Value object.
///
/// # Returns
/// - `Ok(Value)`: The content value Object in ELM format if successful.
/// - `Err(Box<dyn Error>)`: An error if the fetch or encoding fails.
fn set_language(language: &str) -> Result<Value, Box<dyn Error>> {
    let template_json = r#"
  {
    "id": "http://publications.europa.eu/resource/authority/language/ENG",
    "type": "Concept",
    "inScheme": {
      "id": "http://publications.europa.eu/resource/authority/language",
      "type": "ConceptScheme"
    },
    "prefLabel": {
      "en": ["English"]
    },
    "notation": "language"
  }
  "#;

    let mut parsed_language_json: Value = serde_json::from_str(template_json).unwrap();

    match language {
        "ENG" => {
            parsed_language_json["id"] =
                Value::String("http://publications.europa.eu/resource/authority/language/ENG".to_string());
            parsed_language_json["prefLabel"]["en"][0] = Value::String("English".to_string());
        }
        "NLD" => {
            parsed_language_json["id"] =
                Value::String("http://publications.europa.eu/resource/authority/language/NLD".to_string());
            parsed_language_json["prefLabel"]["en"][0] = Value::String("dutch".to_string());
        }
        _ => {
            // Handle other cases
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "This language Type is not supported!",
            )));
        }
    }
    Ok(parsed_language_json)
}

pub fn image_to_individual_display(image_value: Value) -> Value {
    //inspect the image object and re write it so it can be reused in ELM

    //we need to achieve the following structure into the indivudualDisplay array:
    let json_data = r#"
    {
        "id": "urn:epass:individualDisplay:c05743e7-9f9d-4e0b-899b-7ae6514c7a02",
        "type": "IndividualDisplay",
        "language": {
          "id": "http://publications.europa.eu/resource/authority/language/ENG",
          "type": "Concept",
          "inScheme": {
            "id": "http://publications.europa.eu/resource/authority/language",
            "type": "ConceptScheme"
          },
          "prefLabel": {
            "en": ["English"]
          },
          "notation": "language"
        },
        "displayDetail": [
          {
            "id": "urn:epass:displayDetail:123",
            "type": "DisplayDetail",
            "image": {
              "id": "urn:epass:mediaObject:https://avatars.githubusercontent.com/u/22613412?v=4",
              "type": "MediaObject",
              "content": "bas64content",
              "contentEncoding": {
                "id": "http://data.europa.eu/snb/encoding/6146cde7dd",
                "type": "Concept",
                "inScheme": {
                  "id": "http://data.europa.eu/snb/encoding/25831c2",
                  "type": "ConceptScheme"
                },
                "prefLabel": {
                  "en": ["base64"]
                }
              },
              "page": 1,
              "contentType": {
                "id": "http://publications.europa.eu/resource/authority/file-type/JPEG",
                "type": "Concept",
                "inScheme": {
                  "id": "http://publications.europa.eu/resource/authority/file-type",
                  "type": "ConceptScheme"
                },
                "prefLabel": {
                  "en": ["JPG"]
                },
                "notation": "file-type"
              }
            }
          }
        ]
      }
    "#;

    let mut parsed_json: Value = serde_json::from_str(json_data).unwrap();

    // OB usess the id field to point to an image or have the image encoded.
    // Content type is also based on either URL or encoding in the id.
    // Extract the `id` field
    let mut encoded_string = String::new();
    let mut file_type_sting = String::new();
    if let Some(id_value) = image_value.get("id") {
        if let Some(ob3_image_id) = id_value.as_str() {
            // Test if `id` is a URL
            // println!("ob3_image_id value: {}", ob3_image_id);
            let url_regex = Regex::new(r"^(https?://[^\s]+)$").unwrap();
            if url_regex.is_match(ob3_image_id) {
                // println!("The `id` is a valid URL: {}", ob3_image_id);
                // Directly mutate the `content` value
                // first try to encode the image in the URL:
                match encode_image_from_url(ob3_image_id) {
                    Ok(_encoded_image_string) => {
                        // println!("Successfully encoded the image.");
                        encoded_string = _encoded_image_string; // Assign the encoded string to the variable

                        if let Some(extension) = Path::new(ob3_image_id).extension() {
                            // Convert the extension to a string
                            if let Some(ext) = extension.to_str() {
                                // println!("File extension: {}", ext);
                                file_type_sting = ext.to_ascii_uppercase().to_string();
                            } else {
                                // println!("Could not convert extension to string.");
                            }
                        } else {
                            // println!("No file extension found.");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        encoded_string = String::new(); // Assign an empty string or a default value in case of an error
                    }
                };
            // } else if Base64Engine.decode(ob3_image_id).is_ok() {
            } else if ob3_image_id.contains("data") {
                // Test if `id` is Base64 encoded
                // println!("The `id` is a Base64-encoded binary string.");
                if let Some((mime_part, content_part)) = ob3_image_id.split_once(',') {
                    if let Some((_, type_and_enc)) = mime_part.split_once('/') {
                        if let Some((subtype, _)) = type_and_enc.split_once(';') {
                            file_type_sting = subtype.to_ascii_uppercase().to_string();
                        }
                    }
                    encoded_string = content_part.to_string();
                } else {
                    // println!("Invalid data URI format.");
                }
            } else {
                // println!("The `id` is neither a URL nor a Base64-encoded string.");
            }
        } else {
            // println!("The 'id' field is not a string.");
        }
    } else {
        // println!("The 'id' field does not exist.");
    }

    if let Some(_image_content) = parsed_json["displayDetail"][0]["image"]["content"].as_str() {
        parsed_json["displayDetail"][0]["image"]["content"] = Value::String(encoded_string);
    } else {
        // println!("Key 'content' in 'image' not found.");
    }

    // Directly mutate the `contentType` value
    // Set the contentType to a choosen value (currently default to PNG)
    if file_type_sting.is_empty() {
        file_type_sting = "PNG".to_string();
    }
    // println!("fileTypoe string: {}", file_type_sting);
    let encoded_content_type = match set_content_type(file_type_sting.as_str()) {
        Ok(encoded_content_type) => {
            // println!("Successfully added the contentType.");
            encoded_content_type // Assign the encoded string to the variable
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Value::Null // Assign an empty string or a default value in case of an error
        }
    };

    if let Some(_content_type) = parsed_json["displayDetail"][0]["image"].as_object() {
        parsed_json["displayDetail"][0]["image"]["contentType"] = encoded_content_type;
    } else {
        // println!("Key 'contentType' in 'image' not found.");
    }

    // Directly mutate the `encoding` value
    // first try to encode the image in the URL:
    let encoding_value = match set_content_enconding_type("base64") {
        Ok(encoding_value) => {
            // println!("Successfully added encoding type to the image.");
            encoding_value // Assign the encoded string to the variable
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Value::Null // Assign an empty string or a default value in case of an error
        }
    };

    if let Some(_image_encoding) = parsed_json["displayDetail"][0]["image"]["contentEncoding"].as_object() {
        parsed_json["displayDetail"][0]["image"]["contentEncoding"] = encoding_value;
    } else {
        // println!("Key 'contentEncoding' in 'image' not found.");
    }

    // Directly mutate the `language` value
    // first try to encode the image in the URL:
    let language_value = match set_language("ENG") {
        Ok(language_value) => {
            // println!("Successfully added language to the individual display properties.");
            language_value // Assign the encoded string to the variable
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Value::Null // Assign an empty string or a default value in case of an error
        }
    };

    if let Some(_language) = parsed_json["language"].as_object() {
        parsed_json["language"] = language_value;
    } else {
        // println!("Key 'language' in 'individualDisplay' not found.");
    }

    //println!("{:#?}", parsed_json);
    parsed_json
}

pub fn create_display_parameter(image_value: Value) -> Value {
    //inspect the image object and re write it so it can be reused in ELM

    //we need to achieve the following structure into the indivudualDisplay array:
    let json_data = r#"
  {
    "id": "urn:epass:displayParameter:1",
    "type": "DisplayParameter",
    "language": [
      {
        "id": "http://publications.europa.eu/resource/authority/language/ENG",
        "type": "Concept",
        "inScheme": {
          "id": "http://publications.europa.eu/resource/authority/language",
          "type": "ConceptScheme"
        },
        "prefLabel": {
          "en": ["English"]
        },
        "notation": "language"
      }
    ],
    "description": {
      "en": [
        "EBSI Example https://github.com/Knowledge-Innovation-Centre/ESBI-JSON-schemas/blob/main/examples%20of%20credentials/DigiComp%20Generic.json"
      ]
    },
    "individualDisplay": [],
    "primaryLanguage": {
      "id": "http://publications.europa.eu/resource/authority/language/ENG",
      "type": "Concept",
      "inScheme": {
        "id": "http://publications.europa.eu/resource/authority/language",
        "type": "ConceptScheme"
      },
      "prefLabel": {
        "en": ["English"]
      },
      "notation": "language"
    },
    "title": {
      "en": ["DigiComp Generic"]
    }
  }  "#;

    let mut parsed_dp_json: Value = serde_json::from_str(json_data).unwrap();

    // Add individual display value
    // Set the contentType to a choosen value (currently default to PNG)
    parsed_dp_json["individualDisplay"] = Value::Array(vec![image_to_individual_display(image_value)]);

    parsed_dp_json

    // if let Some(id_value) = identity_value.get("identityHash") {
    //     if identity_type.eq(&"Student ID".to_string()) {
    //         let mut new_object = Map::new();
    //         new_object.insert("id".to_string(), Value::String("urn:epass:identifier:2".to_string()));
    //         new_object.insert("type".to_string(), Value::String("Identifier".to_string()));
    //         new_object.insert("notation".to_string(), id_value.clone());
    //         new_object.insert("schemeName".to_string(), Value::String(identity_type.to_string()));
    //         let _current_value = Value::Object(new_object);
    //         _current_value
    //     } else {
    //         id_value.clone()
    //     }
    // } else {
    //     Value::String("".to_string())
    // }
}
