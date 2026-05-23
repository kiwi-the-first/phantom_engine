// NOT YET IMPLENTNED

pub fn serialize<T: serde::Serialize>(value: &T) -> Vec<u8> {
    // swap this line to change backend
    // TODO ADD FEATURE
    bincode::serialize(value).unwrap()
    // serde_json::to_vec(value).unwrap()
}

pub fn deserialize<T: serde::de::DeserializeOwned>(data: &[u8]) -> T {
    // swap this line to change backend
    // TODO ADD FEATURE
    bincode::deserialize(data).unwrap()
    // serde_json::from_slice(data).unwrap()
}
