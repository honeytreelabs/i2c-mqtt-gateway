mod config;
mod homeassistant;
mod i2c;
mod util;

use bytes::Bytes;
use config::{Config, ConfigParser, IOs};
use docopt::Docopt;
use homeassistant::EntityInfo;
use i2c::I2CDeviceTree;
// use i2cdev::core::*;
// use i2cdev::linux::LinuxI2CDevice;
use std::env::args;
use std::path::Path;
use std::process;
use std::str;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;
use util::{io_name_from_mqtt_topic, mqtt_state_topic_from_io_name};
use uuid::Uuid;

use rumqttc::{Client, Connection, Event, Incoming, MqttOptions, QoS};

// Receive from MQTT command topic -> set I2C Output -> publish MQTT state topic
// I2C Input changed -> MQTT state topic

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

struct StateChange {
    io_name: String,
    state: bool,
}

fn generate_uuid() -> String {
    let uuid = Uuid::new_v4();
    uuid.to_string()
}

#[cfg(target_os = "linux")]
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

    let tree = I2CDeviceTree::new(&config);
    let shared_tree = Arc::new(tree);

    // let mut dev = LinuxI2CDevice::new(config.i2c.device, 0x38).unwrap();

    let mut mqttoptions = MqttOptions::new(
        config.mqtt.connection.id,
        config.mqtt.connection.host,
        config.mqtt.connection.port,
    );
    mqttoptions
        .set_keep_alive(Duration::from_secs(5))
        .set_credentials(
            config.mqtt.credentials.user,
            config.mqtt.credentials.password,
        );
    let (client, connection) = Client::new(mqttoptions, 10);

    let ios = Arc::new(config.ios);
    let shared_tree_clone_1 = shared_tree.clone();
    let (sender, receiver) = mpsc::channel();
    let sender_1 = sender.clone();
    thread::spawn(move || notify_cb(connection, shared_tree_clone_1, sender_1));
    let ios_clone_1 = Arc::clone(&ios);
    mqtt_register_ios(&client, &ios_clone_1);
    thread::spawn(move || mqtt_publisher(client, receiver));

    // init sequence
    // dev.smbus_write_byte_data(0xF0, 0x55).unwrap();
    // dev.smbus_write_byte_data(0xFB, 0x00).unwrap();
    let shared_tree_clone_2 = shared_tree.clone();
    thread::spawn(move || i2c_worker(shared_tree_clone_2, sender.clone()));

    loop {
        thread::park();
    }
}

fn mqtt_register_ios(client: &Client, ios: &Arc<IOs>) {
    for output in &ios.outputs {
        for pin in &output.pins {
            let cmd_topic = format!("hmd/switch/{}/command", pin);
            let s_topic = format!("hmd/switch/{}/state", pin);
            let entity_info = EntityInfo {
                component: "switch".to_string(),
                name: pin.clone(),
                payload_off: "OFF".to_string(),
                payload_on: "ON".to_string(),
                state_topic: s_topic.clone(),
                json_attributes_topic: format!("hmd/switch/{}/attributes", pin),
                command_topic: cmd_topic.clone(),
                unique_id: generate_uuid(),
            };
            let payload = serde_json::to_string(&entity_info).unwrap();
            let discovery_topic = format!("homeassistant/switch/{}/config", pin);
            println!("Publishing '{}' to topic '{}'", payload, discovery_topic);
            let qos = QoS::AtLeastOnce;

            client.subscribe(cmd_topic, QoS::AtMostOnce).unwrap();
            client.publish(discovery_topic, qos, true, payload).unwrap();
            client
                .publish(s_topic, qos, true, "ON".to_string())
                .unwrap();
        }
    }
}

fn notify_cb(
    mut connection: Connection,
    device_tree: Arc<I2CDeviceTree>,
    sender: mpsc::Sender<StateChange>,
) {
    for notification in connection.iter() {
        let notif = match notification {
            Ok(notif) => notif,
            Err(error) => {
                println!("Error = {:?}", error);
                return;
            }
        };

        let incoming = match notif {
            Event::Incoming(incoming) => incoming,
            _ => continue,
        };

        if let Incoming::Publish(publish) = incoming {
            let payload = str::from_utf8(&publish.payload).unwrap();

            let output_name = match io_name_from_mqtt_topic(&publish.topic) {
                Some(output_name) => output_name,
                None => continue,
            };

            let s = match payload {
                "ON" => true,
                "OFF" => false,
                _ => continue,
            };

            device_tree.update(output_name, s);
            sender
                .send(StateChange {
                    io_name: output_name.to_string(),
                    state: s,
                })
                .unwrap();
        }
    }
}

fn mqtt_publisher(client: Client, receiver: mpsc::Receiver<StateChange>) {
    for state_change in receiver {
        client
            .publish(
                mqtt_state_topic_from_io_name(&state_change.io_name),
                QoS::AtMostOnce,
                true,
                if state_change.state {
                    Bytes::from("ON")
                } else {
                    Bytes::from("OFF")
                },
            )
            .unwrap();
    }
}

fn i2c_worker(
    // mut dev: LinuxI2CDevice,
    device_tree: Arc<I2CDeviceTree>,
    _sender: mpsc::Sender<StateChange>,
) {
    loop {
        // read all input states
        // let mut buf: [u8; 6] = [0; 6];
        // dev.read(&mut buf).unwrap();
        // publish to MQTT command topic via _sender

        // write all changed outputs
        for output in &device_tree.outputs {
            let mut output_guard = output.lock().unwrap();
            if output_guard.is_dirty() {
                output_guard.write();
            }
        }

        thread::sleep(Duration::from_millis(10));
    }
}
