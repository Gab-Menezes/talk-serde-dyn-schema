pub use de::*;
pub use ser::*;

mod de;
mod ser;


#[cfg(test)]
mod test {
    use super::{deserialize_alloc, deserialize, serialize};
    use crate::{array_def, struct_def, ty::Ty, JsonValue};

    // #[test]
    // fn bool_roundtrip() {
    //     let ty = Ty::Bool;
    //     let value = JsonValue::Bool(false);

    //     let bytes = deserialize(&ty, &value.to_string()).unwrap();
    //     let new_value = serialize(serde_json::value::Serializer, &ty, &bytes).unwrap();
    //     assert_eq!(value, new_value);
    // }

    #[test]
    fn simple_roundtrip() {
        let ty = struct_def!({
            "name": Ty::String,
            "age": Ty::U64,
            "hobbies": array_def!(Ty::String),
            "rustacean": Ty::Bool,
        });

        let value = serde_json::json!({
            "name": "Alexander",
            "age": 27,
            "hobbies": [
                "music",
                "programming"
            ],
            "rustacean": true,
        });

        let bytes = deserialize_alloc(&ty, &value).unwrap();
        println!("{:?}", bytes);
        let new_value = serialize(&ty, &bytes).unwrap();
        assert_eq!(value, new_value);
    }

    #[test]
    fn only_string() {
        let ty = Ty::String;
        let value =  "Alexander".into();

        let bytes = deserialize_alloc(&ty, &value).unwrap();
        println!("{:?}", bytes);
        let new_value = serialize(&ty, &bytes).unwrap();
        assert_eq!(value, new_value);
    }

    #[test]
    fn vec_string() {
        let ty = array_def!(Ty::String);
        let value =  Vec::from(["Alexander".to_string(), "Gabriel".to_string()]).into();

        let bytes = deserialize_alloc(&ty, &value).unwrap();
        println!("{:?}", bytes);
        let new_value = serialize(&ty, &bytes).unwrap();
        assert_eq!(value, new_value);
    }

    #[test]
    fn struct_of_struct() {
        let ty = struct_def!({
            "name": Ty::String,
            "age": Ty::U64,
            "parents": array_def!(struct_def!({
                "name": Ty::String,
                "age": Ty::U64,
            })),
            "b": Ty::Bytes,
            "rustacean": Ty::Bool,
        });

        let value = serde_json::json!({
            "name": "Alexander",
            "age": 27,
            "parents": [
                {
                    "name": "ABC",
                    "age": 100
                },
                {
                    "name": "DEF",
                    "age": 40
                }
            ],
            "rustacean": true,
            "b": [100, 50, 200, 255, 0]
        });

        let bytes = deserialize_alloc(&ty, &value).unwrap();
        println!("{:?}", bytes);
        let new_value = serialize(&ty, &bytes).unwrap();
        assert_eq!(value, new_value);
    }
}