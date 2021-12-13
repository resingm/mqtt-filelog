use paho_mqtt as mqtt;
use std::{env, process, time};

fn main() {
    env_logger::init();

    let uri = env::args()
        .nth(1)
        .unwrap_or_else(|| "tcp://localhost:1883".to_string());

    let topic = env::args().nth(2).unwrap_or_else(|| "#".to_string());
    let qos: i32 = 1;

    // Create a client
    // TODO: Make client ID configurable
    let opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(uri)
        .client_id("mqtt-filelog")
        .finalize();

    let mut client = mqtt::Client::new(opts).unwrap_or_else(|e| {
        println!("Error creating the MQTT client: {:?}", e);
        process::exit(1);
    });

    // Initialize the consumer before connecting
    let rx = client.start_consuming();

    let opts = mqtt::ConnectOptionsBuilder::new()
        .user_name("mqtt-filelog")
        .password("secret")
        .keep_alive_interval(time::Duration::from_secs(20))
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
                    match client.subscribe(&topic, qos) {
                        Ok(qosv) => println!("QoS granted: {:?}", qosv),
                        Err(e) => {
                            println!("Error subscribing to {}: {:?}", topic, e);
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

    // Consume incoming messages...
    println!("Waiting for messages...");
    for msg in rx.iter() {
        if let Some(msg) = msg {
            println!("{}", msg);
        } else {
            println!("The other case happened....");
            println!("Checkout the solution at:");
            println!(
                "  https://github.com/eclipse/paho.mqtt.rust/blob/master/examples/sync_consume.rs"
            );
        }
    }

    // Disconnect from broker
    if client.is_connected() {
        println!("Disconnecting...");
        client.unsubscribe(&topic).unwrap();
        client.disconnect(None).unwrap();
    }
}
