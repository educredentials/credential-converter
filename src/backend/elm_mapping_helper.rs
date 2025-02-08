use codes_iso_3166::part_1::CountryCode;
use serde_json::{json, Value};
use std::str::FromStr;

/// Creates country code based on input type in string found in addressCountryCode
///
/// # Arguments
/// - `country_code`: the code found in .
///
/// # Returns
/// - Value: The content value Object in ELM format if successful.
pub fn address_to_location(address_value: Value) -> Value {
    //inspect the address object (address as used in issuer for now) and re write it so it can be reused in ELM
    //we need to achieve the following structure into the indivudualDisplay array:
    let json_data = r#"
      {
        "id": "urn:epass:certificateLocation:1",
        "type": "Location",
        "address": {
          "id": "urn:epass:certificateAddress:1",
          "type": "Address",
          "countryCode": {
            "id": "http://publications.europa.eu/resource/authority/country/ESP",
            "type": "Concept",
            "inScheme": {
              "id": "http://publications.europa.eu/resource/authority/country",
              "type": "ConceptScheme"
            },
            "notation": "country",
            "prefLabel": { "en": "Spain" }
          }
        }
      }
    "#;

    let mut parsed_json: Value = serde_json::from_str(json_data).unwrap();

    // set country code default to NL
    let mut country = CountryCode::NL;

    // Directly mutate the `Location` value
    // Access the "addressCountryCode" field
    if let Some(country_code) = address_value.get("addressCountryCode") {
        if let Some(country_code_str) = country_code.as_str() {
            if country_code_str.is_empty() {
                //println!("The addressCountryCode is empty.");
                country = CountryCode::from_str("NLD").unwrap();
            } else {
                country = CountryCode::from_str(country_code_str).unwrap();
                //println!("The addressCountryCode is: {}", country_code_str);
            }
        } else {
            //println!("The addressCountryCode is not a string.");
        }
    } else {
        //println!("The addressCountryCode field does not exist.");
    }

    parsed_json["address"]["countryCode"]["id"] = Value::String(format!(
        "http://publications.europa.eu/resource/authority/language/{}",
        country.alpha_3_code().unwrap()
    ));
    parsed_json["address"]["countryCode"]["prefLabel"]["en"] = Value::String(country.full_name().unwrap().to_string());

    //println!("{:#?}", parsed_json);
    parsed_json
}

/// Creates specifiedBy based on input type in string found title
///
/// # Arguments
/// - `title`: the code found in .
///
/// # Returns
/// - Value: The content value Object in ELM format if successful.
pub fn title_to_specifiedby(title: Value) -> Value {
    //inspect the title object and re write it so it can be reused in ELM for building a Specification
    //we need to achieve the following structure for a specification:
    let json_data = r#"
  {
          "id": "urn:epass:learningAchievementSpec:1",
          "type": "Qualification",
          "title": {
            "en": ["Data and soferetware business"]
          }
  }
  "#;

    let mut parsed_json: Value = serde_json::from_str(json_data).unwrap();

    // Directly mutate the `Location` value
    // Access the "addressCountryCode" field
    if let Some(title_str) = title.as_str() {
        if title_str.is_empty() {
            return Value::Null;
        } else {
            parsed_json["title"]["en"][0] = Value::String(title_str.to_string());
        }
    } else {
        return Value::Null;
    }

    //println!("{:#?}", parsed_json);
    parsed_json
}

/// Creates specifiedBy based on input type in string found title
///
/// # Arguments
/// - `credits`: the ammount of credits for a credential
///
/// # Returns
/// - Value: The creditpoint value Object in ELM format if successful.
pub fn credentialpoint_values_to_object(credits: Value) -> Value {
    //inspect the title object and re write it so it can be reused in ELM for building a creditpoint that cn be used in Specification
    //we need to achieve the following structure for a creditpoint:
    let json_data = r#"
  {
    "id": "urn:epass:creditPoint:1",
    "type": "CreditPoint",
    "framework": {
      "id": "http://data.europa.eu/snb/education-credit/6fcec5c5af",
      "type": "Concept",
      "inScheme": {
        "id": "http://data.europa.eu/snb/education-credit/25831c2",
        "type": "ConceptScheme"
      },
      "prefLabel": {
        "en": ["European Credit Transfer System"]
      }
    },
    "point": "5"
  }
  "#;

    let mut parsed_json: Value = serde_json::from_str(json_data).unwrap();

    // Directly mutate the `Location` value
    // Access the "addressCountryCode" field
    match credits {
        Value::String(_) => {
            parsed_json["point"] = Value::String(credits.to_string());
        }
        Value::Number(_) => {
            parsed_json["point"] = Value::String(credits.to_string());
        }
        _ => {
            return Value::Null;
        }
    }
    //println!("{:#?}", parsed_json);
    parsed_json
}

/// Creates specifiedBy based on input type in string found title
///
/// # Arguments
/// - `alignment`: an array that could be found in OBv3 but needs to be translated to fit the new structure of ELM.
///
/// # Returns
/// - Value: The content value Object in ELM format if successful.
pub fn eqf_to_specifiedby_qualification(alignment: Value) -> Value {
    //inspect the title object and re write it so it can be reused in ELM for building a creditpoint that cn be used in Specification
    //we need to achieve the following structure for a creditpoint:
    let json_data = r#"
      {
        "id": "http://data.europa.eu/snb/eqf/5",
        "type": "Concept",
        "inScheme": {
          "id": "http://data.europa.eu/snb/eqf/25831c2",
          "type": "ConceptScheme"
        },
        "prefLabel": {
          "en": ["Level 5"]
        }
      }
  "#;

    let mut parsed_json: Value = serde_json::from_str(json_data).unwrap();
    //println!("{:#?}", alignment);
    // Extract the array from the Value
    if let Some(array) = alignment.as_array() {
        // Find the targetCode where targetType == "ext:EQF"
        if let Some(target_code) = array
            .iter()
            .find(|item| item.get("targetType").and_then(|v| v.as_str()) == Some("ext:EQF"))
            .and_then(|item| item.get("targetCode").and_then(|v| v.as_str()))
        {
            parsed_json["id"] = Value::String(format!(
                "http://publications.europa.eu/resource/authority/language/{}",
                target_code
            ));
            parsed_json["prefLabel"]["en"] = Value::String(format!("Level {}", target_code));
        } else {
            //println!("targetCode not found.");
            return Value::Null;
        }
    } else {
        //println!("Error: Data is not an array.");
        return Value::Null;
    }

    //println!("{:#?}", parsed_json);
    parsed_json
}

/// Creates specifiedBy based on input type in string found title
///
/// # Arguments
/// - `assessment_type`: an array that could be found in OBv3 but needs to be translated to fit the new structure of ELM.
///
/// # Returns
/// - Value: The specifiedBy object with dummies in ELM format if successful.
pub fn assessment_type_to_specifiedby_assesment(assessement_type: Value) -> Value {
    //inspect the title object and re write it so it can be reused in ELM for building a creditpoint that cn be used in Specification
    //we need to achieve the following structure for a creditpoint:
    let json_data = r#"
{
  "id": "urn:epass:learningAssessment:1",
  "type": "LearningAssessment",
  "awardedBy": {
    "id": "urn:epass:awardingProcess:1",
    "type": "AwardingProcess",
    "awardingBody": [
      {
        "id": "urn:epass:org:1",
        "type": "Organisation",
        "location": [ {"address":"placeholder"}
        ],
        "legalName": {
          "en": ["University of Life"]
        }
      }
    ]
  },
  "title": {
    "en": ["AssessmentTypeValue"]
  },
  "grade": {
    "id": "urn:epass:note:1",
    "type": "Note",
    "noteLiteral": {
      "en": ["0"]
    }
  }
}

"#;

    let mut parsed_json: Value = serde_json::from_str(json_data).unwrap();

    // Directly mutate the title value of the assessment
    if let Some(assessement_type_str) = assessement_type.as_str() {
        if assessement_type_str.is_empty() {
            return Value::Null;
        } else {
            parsed_json["title"]["en"][0] = Value::String(assessement_type_str.to_string());
        }
    } else {
        return Value::Null;
    }

    //println!("{:#?}", parsed_json);
    parsed_json
}

/// Creates specifiedBy based on input type in string found title
///
/// # Arguments
/// - `assessment_type`: an array that could be found in OBv3 but needs to be translated to fit the new structure of ELM.
///
/// # Returns
/// - Value: The specifiedBy object with dummies in ELM format if successful.
pub fn object_to_note_literal(any_object: Value) -> Value {
    //inspect the title object and re write it so it can be reused in ELM for building a creditpoint that cn be used in Specification
    //we need to achieve the following structure for a creditpoint:

    let str_array = handle_json_input(&any_object);
    Value::String(str_array)
}

/// Creates learningOutcomes based on input type in outcome array
///
/// # Arguments
/// - `learning_outcomes`: an array that could be found in OBv3 but needs to be translated to fit the new structure of ELM.
///
/// # Returns
/// - Value: The specifiedBy object with dummies in ELM format if successful.
pub fn transform_learning_outcomes(json_obj: Value) -> Value {
    if let Some(learning_outcomes) = json_obj.as_array() {
        let transformed_outcomes: Vec<Value> = learning_outcomes
            .iter()
            .map(|outcome| {
                let new_related_skill = outcome.get("relatedSkill").map(|related| {
                    json!({
                        "id": related.get("id").unwrap_or(&json!("")),
                        "type": "Concept",
                        "inScheme": {
                            "id": "https://publications.europa.eu/resource/authority/snb/dcf/25831c2",
                            "type": "ConceptScheme"
                        },
                        "prefLabel": {
                            "en": [related.get("title").unwrap_or(&json!("")).as_str().unwrap_or("")]
                        }
                    })
                });

                let mut new_outcome = outcome.clone();
                if let Some(obj) = new_outcome.as_object_mut() {
                    if let Some(new_related_skill) = new_related_skill {
                        obj.insert("relatedSkill".to_string(), new_related_skill);
                    }
                    obj.insert("title".to_string(), json!({"en": [outcome.get("title")]}));
                }
                // Modify fields inside the object
                new_outcome
            })
            .collect();

        Value::Array(transformed_outcomes)
    } else {
        Value::Null
    }
}

/// Creates a learning outcomes summary structure
///
/// # Arguments
/// - `criteria`: Tekst as found in criteria in OBv3.
///
/// # Returns
/// - Value: The learningOutcomeSummary object with dummies in ELM format if successful.
pub fn create_learning_outcome_summary(json_obj: Value) -> Value {
    if let Some(outcome_sum_str) = json_obj.as_str() {
        if outcome_sum_str.is_empty() {
            Value::Null
        } else {
            let json_result = json!({
              "id": "urn:epass:note:3",
              "type": "Note",
              "noteLiteral": {
                  "en": [outcome_sum_str]
              }
            });
            json_result
        }
    } else {
        Value::Null
    }
}

// additional private helpers
// Function to handle both single object and array of objects
fn handle_json_input(json_obj: &Value) -> String {
    if json_obj.is_array() {
        // If it's an array, map each object to a string and join them
        json_obj
            .as_array()
            .unwrap()
            .iter()
            .map(|obj| object_to_string(obj))
            .collect::<Vec<String>>()
            .join(" | ") // Separate objects with " | "
    } else if json_obj.is_object() {
        // If it's a single object, convert it directly
        object_to_string(json_obj)
    } else {
        "Invalid JSON format".to_string()
    }
}

// Function to convert JSON object (with 1-level nesting) to a "key:value" string
fn object_to_string(json_obj: &Value) -> String {
    if let Some(obj) = json_obj.as_object() {
        obj.iter()
            .flat_map(|(key, value)| {
                match value {
                    Value::Object(nested_obj) => {
                        // Flatten nested objects: "key.nested_key:value"
                        nested_obj
                            .iter()
                            .map(|(nested_key, nested_value)| {
                                format!("{}.{}:{}", key, nested_key, value_to_string(nested_value))
                            })
                            .collect::<Vec<String>>()
                    }
                    _ => vec![format!("{}:{}", key, value_to_string(value))], // Normal key:value
                }
            })
            .collect::<Vec<String>>()
            .join(", ")
    } else {
        "Invalid JSON object".to_string()
    }
}

// Converts JSON values to strings
fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Array(arr) => format!("[{}]", arr.iter().map(value_to_string).collect::<Vec<_>>().join(", ")),
        Value::Object(_) => "{...}".to_string(), // Shouldn't reach here due to flattening
        Value::Null => "null".to_string(),
    }
}
