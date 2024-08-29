/// Extracts the I/O name from an MQTT topic.
///
/// This function splits the given MQTT topic by the '/' character and checks if
/// the topic has at least four parts. If it does, the third part (index 2) is returned
/// as the I/O name. Otherwise, `None` is returned.
///
/// # Examples
///
/// ```rust
/// assert_eq!(io_name_from_mqtt_topic("hmd/switch/my_io/state"), Some("my_io"));
/// assert_eq!(io_name_from_mqtt_topic("hmd/switch"), None);
/// ```
///
/// # Arguments
///
/// * `topic` - A string slice holding the MQTT topic to parse.
///
/// # Returns
///
/// * `Some(&str)` containing the I/O name if the topic is valid.
/// * `None` if the topic is not valid.
pub fn io_name_from_mqtt_topic(topic: &str) -> Option<&str> {
    let parts: Vec<&str> = topic.split('/').collect();

    if parts.len() >= 4 {
        Some(parts[2])
    } else {
        None
    }
}

/// Creates an MQTT state topic from an I/O name.
///
/// This function formats a string to create an MQTT state topic using the given
/// I/O name. The resulting topic has the format `hmd/switch/{io_name}/state`.
///
/// # Examples
///
/// ```rust
/// assert_eq!(mqtt_state_topic_from_io_name("my_io"), "hmd/switch/my_io/state");
/// ```
///
/// # Arguments
///
/// * `io_name` - A string slice holding the I/O name to include in the topic.
///
/// # Returns
///
/// * `String` containing the formatted MQTT state topic.
pub fn mqtt_state_topic_from_io_name(io_name: &str) -> String {
    format!("hmd/switch/{}/state", io_name)
}
