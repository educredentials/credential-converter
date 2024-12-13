use crate::{
    backend::{
        jsonpointer::{JsonPath, JsonPointer},
        leaf_nodes::construct_leaf_node,
        transformations::{DataLocation, DataTypeLocation, StringArrayValue, StringValue, Transformation},
        base64_encode::encode_image_from_url,
    },
    state::{AppState, Mapping},
    trace_dbg,
};
use jsonpath_rust::JsonPathFinder;
use serde_json::{json, Map, Value};
//use tracing_subscriber::fmt::format;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Default, Clone)]
pub struct Repository(HashMap<String, Value>);

impl DerefMut for Repository {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for Repository {
    type Target = HashMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<HashMap<String, Value>> for Repository {
    fn from(map: HashMap<String, Value>) -> Self {
        Self(map)
    }
}

impl Repository {
    pub fn apply_transformation(
        &mut self,
        transformation: Transformation,
        mapping: Mapping,
    ) -> Option<(String, String)> {
        match transformation {
            Transformation::OneToOne {
                type_: transformation,
                source:
                    DataLocation {
                        format: source_format,
                        path: mut source_path,
                    },
                destination:
                    DataLocation {
                        format: destination_format,
                        path: destination_path,
                    },
            } => {
                if source_format != mapping.input_format() || destination_format != mapping.output_format() {
                    return None;
                }

                let source_credential = self.get(&source_format).unwrap();

                // custom code to handle the special character '@' in the source_path
                if source_path == "$.@context" {
                    source_path = r#"$["@context"]"#.to_string();
                };

                let finder = JsonPathFinder::from_str(&source_credential.to_string(), &source_path).unwrap();

                let source_value = match finder.find().as_array() {
                    // todo: still need to investigate other find() return types
                    Some(array) => array.first().unwrap().clone(),
                    None => {
                        return None;
                    }
                };

                let destination_credential = self.entry(destination_format).or_insert(json!({})); // or_insert should never happen, since repository is initialized with all formats, incl empty json value when not present.

                let pointer = JsonPointer::try_from(JsonPath(destination_path.clone())).unwrap();

                let mut leaf_node = construct_leaf_node(&pointer);

                if let Some(value) = leaf_node.pointer_mut(&pointer) {
                    *value = transformation.apply(source_value);
                }

                merge(destination_credential, leaf_node);

                trace_dbg!("Successfully completed transformation");
                Some((destination_path, source_path))
            }
            Transformation::ManyToOne {
                type_: transformation,
                sources,
                destination,
            } => {
                if sources.iter().any(|source| source.format != mapping.input_format())
                    || destination.format != mapping.output_format()
                {
                    return None;
                }

                let source_values = sources
                    .iter()
                    .map(|source| {
                        let source_credential = self.get(&source.format).unwrap();

                        let finder = JsonPathFinder::from_str(&source_credential.to_string(), &source.path).unwrap();
                        finder.find().as_array().unwrap().first().unwrap().clone()
                    })
                    .collect::<Vec<_>>();

                let destination_credential = self.entry(destination.format).or_insert(json!({}));
                let pointer = JsonPointer::try_from(JsonPath(destination.path.clone())).unwrap();

                let mut leaf_node = construct_leaf_node(&pointer);

                if let Some(value) = leaf_node.pointer_mut(&pointer) {
                    *value = transformation.apply(source_values);
                }

                merge(destination_credential, leaf_node);

                trace_dbg!("Successfully completed transformation");
                None // Todo: this is not implemented yet, so returns None for now
            }

            Transformation::StringToOne {
                type_: transformation,
                source: StringValue { value: source_value },
                destination:
                    DataLocation {
                        format: destination_format,
                        path: destination_path,
                    },
            } => {
                if destination_format != mapping.output_format() {
                    return None;
                }

                let destination_credential = self.entry(destination_format).or_insert(json!({})); // or_insert should never happen, since repository is initialized with all formats, incl empty json value when not present.
                let pointer = JsonPointer::try_from(JsonPath(destination_path)).unwrap();

                let mut leaf_node = construct_leaf_node(&pointer);
                if let Some(value) = leaf_node.pointer_mut(&pointer) {
                    *value = transformation.apply(source_value);
                }

                merge(destination_credential, leaf_node);
                None
            }

            Transformation::StringArrayToOne {
                type_: transformation,
                source: StringArrayValue { value: source_value },
                destination:
                    DataLocation {
                        format: destination_format,
                        path: destination_path,
                    },
            } => {
                if destination_format != mapping.output_format() {
                    return None;
                }

                let destination_credential = self.entry(destination_format).or_insert(json!({})); // or_insert should never happen, since repository is initialized with all formats, incl empty json value when not present.
                let pointer = JsonPointer::try_from(JsonPath(destination_path)).unwrap();
                let mut leaf_node = construct_leaf_node(&pointer);
                if let Some(value) = leaf_node.pointer_mut(&pointer) {
                    let json_value: Value = Value::Array(
                        source_value
                            .into_iter()
                            .map(Value::String) // Convert each String into serde_json::Value::String
                            .collect(),
                    );
                    *value = transformation.apply(json_value);
                }

                merge(destination_credential, leaf_node);
                None
            }

            Transformation::JsonToMarkdown {
                type_: transformation,
                source:
                    DataLocation {
                        format: source_format,
                        path: source_path,
                    },
                destination:
                    DataLocation {
                        format: destination_format,
                        path: destination_path,
                    },
            } => {
                if source_format != mapping.input_format() || destination_format != mapping.output_format() {
                    return None;
                }

                let source_credential = self.get(&source_format).unwrap();

                let finder = JsonPathFinder::from_str(&source_credential.to_string(), &source_path).unwrap();

                let source_value = match finder.find().as_array() {
                    // todo: still need to investigate other find() return types
                    Some(array) => array.first().unwrap().clone(),
                    None => {
                        return None;
                    }
                };

                let destination_credential = self.entry(destination_format).or_insert(json!({})); // or_insert should never happen, since repository is initialized with all formats, incl empty json value when not present.
                let pointer = JsonPointer::try_from(JsonPath(destination_path.clone())).unwrap();

                let mut leaf_node = construct_leaf_node(&pointer);

                // run the source value through a markdown converter to fit the nested objects into a markdown string
                let markdown_source_value = json!(json_to_markdown(&source_value, 0));

                if let Some(value) = leaf_node.pointer_mut(&pointer) {
                    *value = transformation.apply(markdown_source_value);
                }

                merge(destination_credential, leaf_node);

                trace_dbg!("Successfully completed transformation");
                Some((destination_path, source_path))
            }

            Transformation::MarkdownToJson {
                type_: transformation,
                source:
                    DataLocation {
                        format: source_format,
                        path: source_path,
                    },
                destination:
                    DataLocation {
                        format: destination_format,
                        path: destination_path,
                    },
            } => {
                if source_format != mapping.input_format() || destination_format != mapping.output_format() {
                    return None;
                }

                let source_credential = self.get(&source_format).unwrap();

                let finder = JsonPathFinder::from_str(&source_credential.to_string(), &source_path).unwrap();

                let source_value = match finder.find().as_array() {
                    // todo: still need to investigate other find() return types
                    Some(array) => array.first().unwrap().clone(),
                    None => {
                        return None;
                    }
                };

                let destination_credential = self.entry(destination_format).or_insert(json!({})); // or_insert should never happen, since repository is initialized with all formats, incl empty json value when not present.
                let pointer = JsonPointer::try_from(JsonPath(destination_path.clone())).unwrap();

                let mut leaf_node = construct_leaf_node(&pointer);

                if let Some(inner_string) = &source_value.as_str() {
                    let mut lines: Vec<&str> = inner_string.lines().collect();

                    lines.insert(0, "");

                    // Split the string by newlines and collect into Vec<&str>
                    let markdown_function_result = markdown_to_json(&lines);

                    if let Some(value) = leaf_node.pointer_mut(&pointer) {
                        *value = transformation.apply(markdown_function_result);
                    }
                }

                merge(destination_credential, leaf_node);

                trace_dbg!("Successfully completed transformation");
                Some((destination_path, source_path))
            }

            Transformation::AddIdentifier {
                type_: transformation,
                source:
                    DataTypeLocation {
                        format: source_format,
                        datatype: source_type,
                        path: source_path,
                    },
                destination:
                    DataLocation {
                        format: destination_format,
                        path: destination_path,
                    },
            } => {
                if source_format != mapping.input_format() || destination_format != mapping.output_format() {
                    return None;
                }

                let source_credential = self.get(&source_format).unwrap();

                let finder = JsonPathFinder::from_str(&source_credential.to_string(), &source_path).unwrap();

                let source_value = match finder.find().as_array() {
                    // todo: still need to investigate other find() return types
                    Some(array) => array.first().unwrap().clone(),
                    None => {
                        return None;
                    }
                };

                let destination_credential = self.entry(destination_format).or_insert(json!({})); // or_insert should never happen, since repository is initialized with all formats, incl empty json value when not present.
                let pointer = JsonPointer::try_from(JsonPath(destination_path.clone())).unwrap();

                let mut leaf_node = construct_leaf_node(&pointer);
                let identifier_function_result = values_to_identity(&source_type, source_value);

                if let Some(value) = leaf_node.pointer_mut(&pointer) {
                    *value = transformation.apply(identifier_function_result);
                }

                merge(destination_credential, leaf_node);

                trace_dbg!("Successfully completed transformation");
                Some((destination_path, source_path))
            }

            Transformation::IdentifierToObject {
                type_: transformation,
                source:
                    DataTypeLocation {
                        format: source_format,
                        datatype: source_type,
                        path: source_path,
                    },
                destination:
                    DataLocation {
                        format: destination_format,
                        path: destination_path,
                    },
            } => {
                if source_format != mapping.input_format() || destination_format != mapping.output_format() {
                    return None;
                }

                let source_credential = self.get(&source_format).unwrap();

                let finder = JsonPathFinder::from_str(&source_credential.to_string(), &source_path).unwrap();

                let source_value = match finder.find().as_array() {
                    // todo: still need to investigate other find() return types
                    Some(array) => array.first().unwrap().clone(),
                    None => {
                        return None;
                    }
                };

                let destination_credential = self.entry(destination_format).or_insert(json!({})); // or_insert should never happen, since repository is initialized with all formats, incl empty json value when not present.
                let pointer = JsonPointer::try_from(JsonPath(destination_path.clone())).unwrap();

                let mut leaf_node = construct_leaf_node(&pointer);
                let identifier_function_result = identity_to_object(&source_type, source_value);

                if let Some(value) = leaf_node.pointer_mut(&pointer) {
                    *value = transformation.apply(identifier_function_result);
                }

                merge(destination_credential, leaf_node);

                trace_dbg!("Successfully completed transformation");
                Some((destination_path, source_path))
            }

            Transformation::ImageToIndividualDisplay {
                type_: transformation,
                source:
                    DataLocation {
                        format: source_format,
                        path: source_path,
                    },
                destination:
                    DataLocation {
                        format: destination_format,
                        path: destination_path,
                    },
            } => {
                if source_format != mapping.input_format() || destination_format != mapping.output_format() {
                    return None;
                }

                let source_credential = self.get(&source_format).unwrap();

                let finder = JsonPathFinder::from_str(&source_credential.to_string(), &source_path).unwrap();

                let source_value = match finder.find().as_array() {
                    // todo: still need to investigate other find() return types
                    Some(array) => array.first().unwrap().clone(),
                    None => {
                        return None;
                    }
                };

                let destination_credential = self.entry(destination_format).or_insert(json!({})); // or_insert should never happen, since repository is initialized with all formats, incl empty json value when not present.
                let pointer = JsonPointer::try_from(JsonPath(destination_path.clone())).unwrap();

                let mut leaf_node = construct_leaf_node(&pointer);

                // run the source value through a markdown converter to fit the nested objects into a markdown string
                let markdown_source_value = json!(image_to_individual_display(source_value));

                if let Some(value) = leaf_node.pointer_mut(&pointer) {
                    *value = transformation.apply(markdown_source_value);
                }

                merge(destination_credential, leaf_node);

                trace_dbg!("Successfully completed transformation");
                Some((destination_path, source_path))
            }



            _ => todo!(),
        }
    }

    pub fn apply_transformations(
        &mut self,
        transformations: Vec<Transformation>,
        mapping: Mapping,
    ) -> Vec<(String, String)> {
        let mut completed_fields: Vec<(String, String)> = Vec::new();

        for transformation in transformations {
            if let Some(completed_field) = self.apply_transformation(transformation, mapping) {
                trace_dbg!(&completed_field);
                completed_fields.push(completed_field);
            }
        }

        completed_fields
    }

    pub fn clear_mapping(&mut self, mut output_pointer: String, mapping: Mapping) {
        let output_json = self.get_mut(&mapping.output_format()).unwrap();

        output_pointer = output_pointer.trim_start_matches("/").to_string();
        let keys: Vec<String> = output_pointer.split('/').map(|s| s.to_string()).collect();

        remove_key_recursive(output_json, &keys);
    }
}

pub fn merge(a: &mut Value, b: Value) {
    match (a, b) {
        (a @ &mut Value::Object(_), Value::Object(b)) => {
            let a = a.as_object_mut().unwrap();
            for (k, v) in b {
                merge(a.entry(k).or_insert(Value::Null), v);
            }
        }
        (a @ &mut Value::Array(_), Value::Array(b_arr)) => {
            let a_arr = a.as_array_mut().unwrap();
            let a_len = a_arr.len();
            let b_iter = b_arr.into_iter();

            for (i, b_val) in b_iter.enumerate() {
                if i < a_len {
                    merge(&mut a_arr[i], b_val);
                } else {
                    a_arr.push(b_val);
                }
            }
        }
        (_, Value::Null) => {
            // If the incoming merge Json Value is `Value::Null`, do nothing to the existing Json Value,the current repository, `a`
        }
        (a, b) => *a = b,
    }
}

pub fn update_repository(state: &mut AppState) {
    let output_pointer = state.output_pointer.clone();
    let output_format = state.mapping.output_format();

    let source_value = state.candidate_output_value.clone();
    let output_json = state.repository.get_mut(&output_format).unwrap();

    let mut leaf_node = construct_leaf_node(&output_pointer);

    if let Some(value) = leaf_node.pointer_mut(&output_pointer) {
        *value = serde_json::from_str(&source_value).unwrap();
    }

    merge(output_json, leaf_node);
}

fn remove_key_recursive(current_json: &mut Value, keys: &[String]) -> bool {
    if let Some((first, rest)) = keys.split_first() {
        if let Some(obj) = current_json.as_object_mut() {
            if rest.is_empty() {
                obj.remove(first);
            } else if let Some(child) = obj.get_mut(first) {
                // Recursively traverse deeper layers
                if remove_key_recursive(child, rest) {
                    // If the child is now empty, remove it
                    obj.remove(first);
                }
            }

            // Return true if the current object is empty
            return obj.is_empty();
        }
    }

    false
}

fn values_to_identity(identity_type: &str, identity_value: Value) -> Value {
    //Create a new identity object that is fit for puprose in OBv3 (so not to lose information)

    let mut new_object = Map::new();
    new_object.insert("type".to_string(), Value::String("IdentityObject".to_string()));
    new_object.insert("identityHash".to_string(), identity_value);
    new_object.insert("identityType".to_string(), Value::String(identity_type.to_string()));
    new_object.insert("hashed".to_string(), Value::Bool(false));
    new_object.insert("salt".to_string(), Value::String("not-used".to_string()));
    let _current_value = Value::Object(new_object);
    _current_value
}

fn identity_to_object(identity_type: &str, identity_value: Value) -> Value {
    //inspect the identity object and re write it so it can be reused in ELM

    //we need to achieve the followin structures:
    // "identifier": [
    //     {
    //       "id": "urn:epass:identifier:2",
    //       "type": "Identifier",
    //       "notation": "75541452",
    //       "schemeName": "Student ID"
    //     }
    //   ],

    // and for example
    // "givenName": {
    //     "en": ["David"]
    //   },

    if let Some(id_value) = identity_value.get("identityHash") {
        if identity_type.eq(&"Student ID".to_string()) {
            let mut new_object = Map::new();
            new_object.insert("id".to_string(), Value::String("urn:epass:identifier:2".to_string()));
            new_object.insert("type".to_string(), Value::String("Identifier".to_string()));
            new_object.insert("notation".to_string(), id_value.clone());
            new_object.insert("schemeName".to_string(), Value::String(identity_type.to_string()));
            let _current_value = Value::Object(new_object);
            _current_value
        } else {
            id_value.clone()
        }
    } else {
        Value::String("".to_string())
    }
}


fn image_to_individual_display(image_value: Value) -> Value {
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
              "content": "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",
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
              "contentType": {
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
            }
          }
        ]
      }
    "#;
 

    let mut parsed_json: Value = serde_json::from_str(json_data).unwrap();

    // Directly mutate the `content` value
    // first try to encode the image in the URL:
    let encoded_string = match encode_image_from_url("https://avatars.githubusercontent.com/u/22613412?v=4") {
        Ok(encoded_string) => {
            println!("Successfully encoded the image.");
            encoded_string // Assign the encoded string to the variable
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            String::new() // Assign an empty string or a default value in case of an error
        }
    };
    if let Some(image_content) = parsed_json["displayDetail"][0]["image"]["content"].as_str() {
        parsed_json["displayDetail"][0]["image"]["content"] = Value::String(encoded_string);
    } else {
        println!("Key 'id' in 'language' not found.");
    }
    


    println!("{:#?}", parsed_json);
    parsed_json

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

fn json_to_markdown(json: &Value, indent_level: usize) -> String {
    let mut markdown = String::new();
    let indent = "  ".repeat(indent_level);

    match json {
        // Handle JSON objects (key-value pairs)
        Value::Object(map) => {
            for (key, value) in map {
                // Add the key as a bold label
                markdown.push_str(&format!("{}**{}**:\n", indent, key));
                // Recursively handle the value
                markdown.push_str(&json_to_markdown(value, indent_level + 1));
            }
        }

        // Handle JSON arrays
        Value::Array(array) => {
            for item in array {
                // Add each array item as a list item
                markdown.push_str(&format!("{}- ", indent));
                markdown.push_str(&json_to_markdown(item, indent_level + 1));
            }
        }

        // Handle primitive types: strings, numbers, booleans, and null
        Value::String(s) => markdown.push_str(&format!("{}{}\n", indent, s)),
        Value::Number(n) => markdown.push_str(&format!("{}{}\n", indent, n)),
        Value::Bool(b) => markdown.push_str(&format!("{}{}\n", indent, b)),
        Value::Null => markdown.push_str(&format!("{}null\n", indent)),
    }

    markdown
}

fn markdown_to_json(lines: &[&str]) -> Value {

// Recursively converts indented lines of Markdown into a JSON structure.
// 1.	Parsing Markdown:
// •	Headings (#): These are treated as keys in the resulting JSON object.
// •	Bold Text (**): This is also treated as a key in the JSON object.
// •	List Items (-): These are treated as elements in a JSON array.
// •	Plain Text: If it’s not part of a list or a key, it’s treated as a value associated with the last key in the current JSON object.
// 2.	Indentation Handling:
// •	The code tracks the current indentation level of the Markdown. If the indentation increases, it means a new nested structure (object or array) is starting. If it decreases, the last completed structure is attached to the parent object or array.
// 3.	Stack Management:
// •	A stack is used to manage the nested structure. Each time a new nested object or array is detected, it’s pushed onto the stack. Once the nesting ends (indentation decreases), the structure is popped from the stack and integrated into the parent structure.
// 4.	Regex Patterns:
// •	heading_regex: Matches Markdown headings (e.g., # Title).
// •	bold_regex: Matches bolded keys (e.g., **Key**:).
// •	list_item_regex: Matches list items (e.g., - item).


    let mut i = 0;
    let mut position: Vec<String> = Vec::new();

    //lets create a string to which we will concatenate new lines based on the markdown lines
    position.insert(0, "".to_string());
    let mut json_string = String::from("");
    // evaluate if the input contains markdown or not
    // if markdown is detected we will try to create json otherwise the value of the "markdown" will be put straight into the attribute
    if lines.contains(&"**") == false {
        while i < lines.len() {
            let line = lines[i];
            json_string.push_str(line);
            i += 1;
        }
        let parsed_json: Value = serde_json::to_value(&json_string).unwrap();
        parsed_json        

    } else {



        while i < lines.len() {
            let line = lines[i];

            // Handle key-value pairs (e.g., **key**: value)
            let (obj_type, depth) = evaluate_line(line);
            if obj_type == "E" && i == 0 {
                // open a json object
                json_string.push_str("{\n");
            }

            if obj_type == "O" {
                while depth < position.len() - 1 {
                    // we need to close the positions
                    if let Some(last_value) = position.last() {
                        if last_value == "O" {
                            // The last value is O we can now close this object
                            if let Some(_last_value) = position.pop() {
                                json_string.pop();
                                json_string = json_string.trim_end_matches(',').to_string();
                                json_string.push_str("},\n");
                            }
                        } else if last_value == "A" {
                            // close value A
                            if let Some(_last_value) = position.pop() {
                                json_string.push_str("]\n");
                            }
                        } else if last_value == "OA" && depth < position.len() - 1 {
                            //The last value is OA
                            if let Some(_last_value) = position.pop() {
                                // first close the array
                                json_string.pop();
                                json_string = json_string.trim_end_matches(',').to_string();
                                json_string.push_str("],\n");
                                if depth > 0 {
                                    if let Some(_last_value) = position.pop() {
                                        // then close the object
                                        json_string.push_str("},\n");
                                    }
                                } else {
                                    //"The vector was empty, nothing to remove.");
                                }
                            } else {
                                // ("The vector was empty, nothing to remove.");
                            }
                        } else {
                            // we have a different last value we will remove it from tha vector
                            if let Some(_last_value) = position.pop() {
                                // json_string.push_str("]\n");
                            }
                        }
                    } else {
                        // println!("The vector is empty.");
                    }
                }

                // setup the vector array for the right value and position
                if depth >= position.len() - 1 && i > 0 {
                    //test previous line to see is we might have a nested object
                    let (last_obj_type, _new_depth) = evaluate_line(lines[i - 1]);
                    if last_obj_type == "O" {
                        json_string.push('{');
                        json_string.push_str(&cleanup_string(line));
                        json_string.push(':');
                        position.insert(depth, "O".to_string());
                    } else if let Some(last_value) = position.last() {
                        if last_value == "OA" && depth == position.len() - 1 {
                            json_string.push_str(&cleanup_string(line));
                            json_string.push(':');
                        } else if depth > position.len() - 1 {
                            json_string.push('{');
                            json_string.push_str(&cleanup_string(line));
                            json_string.push(':');
                            position.insert(depth, "O".to_string());
                        } else {
                            json_string.push_str(&cleanup_string(line));
                            json_string.push(':');
                            position[depth] = "O".to_string();
                        }
                    }
                }
            } else if obj_type == "A" {
                while depth < position.len() - 1 {
                    // we need to close the positions
                    if let Some(last_value) = position.last() {
                        if last_value == "O" {
                            if let Some(_last_value) = position.pop() {
                                json_string.push_str("},\n");
                            }
                        } else if last_value == "A" {
                            if let Some(_last_value) = position.pop() {
                                json_string.push_str("]\n");
                            }
                        } else {
                            // The last value is something leave it
                        }
                    } else {
                        // The vector is empty.
                    }
                }

                // setup the vector array for the right value and position
                if depth >= position.len() - 1 {
                    if let Some(last_value) = position.last() {
                        if last_value == "OA" {
                            json_string.push('[');
                        } else if last_value == "A" && depth == position.len() - 1 {
                            position.insert(depth, "A".to_string());
                        } else {
                            position.insert(depth, "A".to_string());
                            json_string.push('[');
                        }
                    }
                    json_string.push_str(&cleanup_string(line));
                }
            } else if obj_type == "OA" {
                while depth < position.len() - 1 {
                    // we need to close the positions
                    if let Some(last_value) = position.last() {
                        if last_value == "O" {
                            if let Some(_last_value) = position.pop() {
                                json_string.pop();
                                json_string.pop();
                                json_string.push_str("},\n");
                            }
                        } else if last_value == "A" {
                            if let Some(_last_value) = position.pop() {
                                json_string.push_str("]\n");
                            } else {
                                // The vector was empty, nothing to remove.
                            }
                        } else {
                            // The last value is something else leave it
                        }
                    } else {
                        // The vector is empty
                    }
                }

                // we are creating a new array that will contain objects of the same type
                json_string.push_str("[ \n {");
                json_string.push_str(&cleanup_string(line));
                json_string.push(':');
                // test if extra handling is needed for closing
                position.insert(depth - 1, "OA".to_string());
                position.insert(depth, "O".to_string());
            } else if obj_type == "V" {
                json_string.push_str(&cleanup_string(line));
                json_string.push_str(",\n");
            }

            i += 1;
        }

        // Finalize the string to which we will concatenate new lines based on the markdown lines
        json_string.pop();
        json_string = json_string.trim_end_matches(',').to_string();
        json_string.push_str("\n}");
        let parsed_json: Value = serde_json::from_str(&json_string).unwrap();
        parsed_json
    }
}

fn evaluate_line(line_to_test: &str) -> (String, usize) {
    //test depth
    let mut depth = line_to_test.chars().take_while(|c| c.is_whitespace()).count() / 2;
    //test type
    let line_type;
    let trimmed = line_to_test.trim();
    if line_to_test.is_empty() {
        // Handle list as object items of previous depth
        line_type = "E";
    } else if trimmed.starts_with("-") && trimmed.ends_with("**:") {
        // Handle list as object items of previous depth
        line_type = "OA";
        depth += 1;
    } else if trimmed.starts_with("**") {
        // Handle list as object items of previous depth
        line_type = "O";
    } else if trimmed.starts_with("-") {
        // Handle as array items of previous depth
        line_type = "A";
    } else {
        // Handle value of previous depth
        line_type = "V";
    }

    (line_type.to_string(), depth)
}

fn cleanup_string(string_to_clean: &str) -> String {
    //trim the string
    let string_to_clean1 = string_to_clean.trim();
    let string_to_clean2 = string_to_clean1.replace("-", "");
    let string_to_clean3 = string_to_clean2.trim();
    let string_to_clean4 = string_to_clean3.replace("**:", "");
    let cleaned_string = string_to_clean4.trim().trim_matches('*').to_string();
    // Add quotes around the cleaned string
    format!("\"{}\"", cleaned_string)
}

