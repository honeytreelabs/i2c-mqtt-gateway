mod config;

use config::{Config, ConfigParser, IOs};
use docopt::Docopt;
use std::env::args;
use std::path::Path;
use std::process;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rumqttc::{Client, Connection, LastWill, MqttOptions, QoS};

const USAGE: &str = "
Expose I2C devices to Home Assistant via MQTT.

Usage:
  i2c-mqtt-gateway <config-file>
  i2c-mqtt-gateway (-h | --help)
  i2c-mqtt-gateway --version

Options:
  -h --help    Show this help text.
  --version    Show version.
";
const VERSION: &str = "0.0.1";

#[cfg(any(target_os = "linux"))]
fn main() {
    // Initialize the logger
    pretty_env_logger::init();

    let args = Docopt::new(USAGE)
        .and_then(|d| d.argv(args()).parse())
        .unwrap_or_else(|e| e.exit());
    if args.get_bool("--version") {
        println!("i2c-mqtt-gateway version: {}", VERSION);
        process::exit(0);
    }
    let file_path = args.get_str("<config-file>");
    let config: Config = match Path::new(file_path).parse_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error parsing config file: {}", e);
            process::exit(1);
        }
    };

    let mut mqttoptions = MqttOptions::new(
        config.mqtt.connection.id,
        config.mqtt.connection.host,
        config.mqtt.connection.port,
    );
    let will = LastWill::new("hello/world", "good bye", QoS::AtMostOnce, false);
    mqttoptions
        .set_keep_alive(Duration::from_secs(5))
        .set_last_will(will)
        .set_credentials(
            config.mqtt.credentials.user,
            config.mqtt.credentials.password,
        );
    let (client, connection) = Client::new(mqttoptions, 10);

    thread::spawn(move || notify_cb(connection));
    let ios = Arc::new(config.ios);
    let ios_clone_1 = Arc::clone(&ios);
    thread::spawn(move || mqtt_worker(client, &ios_clone_1));
    let ios_clone_2 = Arc::clone(&ios);
    thread::spawn(move || i2c_worker(&ios_clone_2));

    loop {
        thread::park();
    }
}

fn notify_cb(mut connection: Connection) {
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

fn mqtt_worker(client: Client, ios: &Arc<IOs>) {
    // Wait for one second before subscribing to a topic
    thread::sleep(Duration::from_secs(1));
    client.subscribe("hello/+/world", QoS::AtMostOnce).unwrap();

    // Send ten messages with lengths ranging from 0 to 9, each message's QoS being at least once
    for (i, input) in ios.inputs.iter().enumerate() {
        let payload = format!(
            "Input -- Address: 0x{:02x}, Chip:{}",
            input.address, input.chip
        );
        let topic = format!("hello/{i}/world");
        println!("Publishing '{}' to topic '{}'", payload, topic);
        let qos = QoS::AtLeastOnce;

        client.publish(topic, qos, true, payload).unwrap();
    }

    for (i, output) in ios.outputs.iter().enumerate() {
        let payload = format!(
            "Output -- Address: 0x{:02x}, Chip:{}",
            output.address, output.chip
        );
        let topic = format!("hello/{i}/world");
        println!("Publishing '{}' to topic '{}'", payload, topic);
        let qos = QoS::AtLeastOnce;

        client.publish(topic, qos, true, payload).unwrap();
    }

    thread::sleep(Duration::from_secs(1));
}

fn i2c_worker(_ios: &Arc<IOs>) {}
