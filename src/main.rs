use rumqttc::{AsyncClient, MqttOptions, QoS, Event, Packet};
use std::env;
use std::time::Duration;
use wakey::WolPacket;

#[tokio::main(flavor = "current_thread")] // 使用单线程运行时以减小体积
async fn main() {
    let private_key = env::var("BEMFA_PRIVATE_KEY").expect("环境变量 BEMFA_PRIVATE_KEY 未设置");
    let topic = env::var("TOPIC").expect("环境变量 TOPIC 未设置");
    let mac_address = env::var("MAC_ADDRESS").expect("环境变量 MAC_ADDRESS 未设置");

    println!("🚀 MQTT-WOL 服务已启动");

    let mut mqtt_options = MqttOptions::new(&private_key, "bemfa.com", 9501);
    mqtt_options.set_keep_alive(Duration::from_secs(30));

    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);

    // 订阅主题
    let _ = client.subscribe(&topic, QoS::AtMostOnce).await;

    let sep = if mac_address.contains('-') { '-' } else { ':' };

    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::Publish(publish))) => {
                let msg = String::from_utf8_lossy(&publish.payload);
                if msg.trim() == "on" {
                    println!("📢 收到指令，正在唤醒...");
                    if let Ok(packet) = WolPacket::from_string(&mac_address, sep) {
                        let _ = packet.send_magic();
                        println!("✅ 魔术包已发出");
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️ 连接异常: {:?}, 5秒后重连...", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
            _ => {}
        }
    }
}