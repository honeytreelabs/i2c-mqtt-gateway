mod config;

use config::{parse_config, Config};
use std::env;
use std::process;
use std::thread;
use std::time::Duration;

use rumqttc::{Client, Connection, LastWill, MqttOptions, QoS};

fn main() {
    // Initialize the logger
    pretty_env_logger::init();

    let args = env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_config>", args[0]);
        process::exit(1);
    }

    let file_path = &args[1];

    // Parse the YAML file directly from the reader
    let config: Config = match parse_config(file_path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error parsing config file: {}", e);
            process::exit(1);
        }
    };

    let mut mqttoptions = MqttOptions::new("test-1", "raspberry-s.lan", 11883);
    let will = LastWill::new("hello/world", "good bye", QoS::AtMostOnce, false);
    mqttoptions
        .set_keep_alive(Duration::from_secs(5))
        .set_last_will(will)
        .set_credentials(
            config.mqtt.credentials.user,
            config.mqtt.credentials.password,
        );
    let (client, connection) = Client::new(mqttoptions, 10);

    thread::spawn(move || process(connection));
    thread::spawn(move || publish(client));

    println!("Done with the stream!!");

    loop {
        thread::park();
    }
}

fn process(mut connection: Connection) {
    for (i, notification) in connection.iter().enumerate() {
        match notification {
            Ok(notif) => {
                println!("{i}. Notification = {notif:?}");
            }
            Err(error) => {
                println!("{i}. Notification = {error:?}");
                return;
            }
        }
    }
}

/*
 * This is a helper function for publishing MQTT messages. In this function, we first sleep
 * for one second, then subscribe to a topic. Then we loop and send ten messages with lengths
 * ranging from 0 to 9, with each message's QoS being at least once.
 */
fn publish(client: Client) {
    // Wait for one second before subscribing to a topic
    thread::sleep(Duration::from_secs(1));
    client.subscribe("hello/+/world", QoS::AtMostOnce).unwrap();

    // Send ten messages with lengths ranging from 0 to 9, each message's QoS being at least once
    for i in 0..10_usize {
        let payload = vec![1; i];
        let topic = format!("hello/{i}/world");
        let qos = QoS::AtLeastOnce;

        client.publish(topic, qos, true, payload).unwrap();
    }

    thread::sleep(Duration::from_secs(1));
}
