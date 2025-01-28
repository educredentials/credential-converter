use codes_iso_3166::part_1::CountryCode;
use std::str::FromStr;
use serde_json::Value;


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

    parsed_json["address"]["countryCode"]["id"] = Value::String(format!("http://publications.europa.eu/resource/authority/language/{}",country.alpha_3_code().unwrap()));
    parsed_json["address"]["countryCode"]["prefLabel"]["en"] = Value::String(country.full_name().unwrap().to_string());
    

    //println!("{:#?}", parsed_json);
    parsed_json


}

