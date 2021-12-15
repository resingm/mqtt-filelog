use chrono::Utc;
use paho_mqtt as mqtt;
use serde::Deserialize;
use std::fs;
use std::io::prelude::*;
use std::{process, thread, time};

#[derive(Deserialize)]
struct Config {
    host: String,
    port: u16,
    id: String,
    username: String,
    password: String,
    topic: String,
    output_file: String,
}

impl Config {
    fn load(path: &str) -> Result<Config, &str> {
        let content: String = fs::read_to_string(path).unwrap_or_else(|e| {
            return format!("{:?}", e);
        });

        let config: Config = toml::from_str(&content).unwrap();
        Ok(config)
    }
}

fn log_message(path: &str, msg: &mqtt::Message) -> std::io::Result<()> {
    let ts = Utc::now();

    let line = format!(
        "{ts};{topic};{payload}\n",
        //ts = ts.format("%Y-%m-%dT%H:%M:%D"),
        ts = ts.to_rfc3339(),
        topic = msg.topic(),
        payload = msg.payload_str()
    );

    let mut file = fs::OpenOptions::new().write(true).append(true).open(path)?;

    //writeln!(file, "{}", line)
    file.write_all(line.as_bytes())
}

fn try_reconnect(cli: &mqtt::Client) -> bool {
    println!("Connection lost. Waiting to retry connection");
    for _ in 0..12 {
        thread::sleep(time::Duration::from_millis(5000));
        if cli.reconnect().is_ok() {
            println!("Successfully reconnected");
            return true;
        }
    }

    println!("Unable to reconnect after several attempts.");
    false
}

fn main() {
    env_logger::init();

    let cfg = Config::load("./config.toml").unwrap_or_else(|e| {
        println!("Error loading configuration file: {:?}", e);
        process::exit(1);
    });

    //let uri = env::args()
    //    .nth(1)
    //    .unwrap_or_else(|| "tcp://localhost:1883".to_string());
    let uri = format!("tcp://{}:{}", cfg.host, cfg.port);
    let qos: i32 = 1;

    // Create a client
    // TODO: Make client ID configurable
    let opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(uri)
        .client_id(cfg.id)
        .finalize();

    let mut client = mqtt::Client::new(opts).unwrap_or_else(|e| {
        println!("Error creating the MQTT client: {:?}", e);
        process::exit(1);
    });

    // Initialize the consumer before connecting
    let rx = client.start_consuming();

    let opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(time::Duration::from_secs(20))
        .user_name(&cfg.username)
        .password(&cfg.password)
        .clean_session(false)
        .finalize();

    // Connect to the broker
    println!("Connecting to the MQTT broker...");
    match client.connect(opts) {
        Ok(rsp) => {
            if let Some(conn_rsp) = rsp.connect_response() {
                println!(
                    "Connected to: '{}' with MQTT version {}",
                    conn_rsp.server_uri, conn_rsp.mqtt_version
                );

                if !conn_rsp.session_present {
                    // Register subscription of topics
                    match client.subscribe(&cfg.topic, qos) {
                        Ok(qosv) => println!("QoS granted: {:?}", qosv),
                        Err(e) => {
                            println!("Error subscribing to {}: {:?}", &cfg.topic, e);
                            client.disconnect(None).unwrap();
                            process::exit(1);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Error connecting to the broker: {:?}", e);
            process::exit(1);
        }
    };

    // Loop on incoming messages.
    // If we get a None message, check if we got disconnected.
    // Afterwards try reconnect;
    println!("Waiting for messages...");
    for msg in rx.iter() {
        if let Some(msg) = msg {
            match log_message(&cfg.output_file, &msg).err() {
                Some(e) => println!("Error logging a message: {:?}", e),
                None => (),
            };

            //println!("{}", msg);
        } else if client.is_connected() || !try_reconnect(&client) {
            break;
        }
    }

    // Disconnect from broker
    if client.is_connected() {
        println!("Disconnecting...");
        client.unsubscribe(&cfg.topic).unwrap();
        client.disconnect(None).unwrap();
    }
}
