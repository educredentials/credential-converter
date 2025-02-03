use codes_iso_3166::part_1::CountryCode;
use serde_json::Value;
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
          "type": "LearningAchievementSpecification",
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
          }
          else{
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
    Value::String(_) => {parsed_json["point"] = Value::String(credits.to_string());},
    Value::Number(_) => {parsed_json["point"] = Value::String(credits.to_string());},
    _ => {return  Value::Null;}
  }
  //println!("{:#?}", parsed_json);
  parsed_json
}




