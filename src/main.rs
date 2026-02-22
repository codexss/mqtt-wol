use rumqttc::{AsyncClient, MqttOptions, QoS, Event, Packet};
use std::env;
use std::time::Duration;
use wakey::WolPacket;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let private_key = env::var("BEMFA_PRIVATE_KEY").expect("环境变量 BEMFA_PRIVATE_KEY 未设置");
    let topic = env::var("TOPIC").expect("环境变量 TOPIC 未设置");
    let mac_address = env::var("MAC_ADDRESS").expect("环境变量 MAC_ADDRESS 未设置");

    println!("🚀 MQTT-WOL 服务已启动");

    let mut mqtt_options = MqttOptions::new(&private_key, "bemfa.com", 9501);
    mqtt_options.set_keep_alive(Duration::from_secs(30));

    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);

    let _ = client.subscribe(&topic, QoS::AtMostOnce).await;

    let sep = if mac_address.contains('-') { '-' } else { ':' };

    let mut current_delay = 1;
    let max_delay = 60;

    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::Publish(publish))) => {
                current_delay = 1; 

                let msg = String::from_utf8_lossy(&publish.payload);
                if msg.trim() == "on" {
                    println!("📢 收到唤醒指令");
                    match WolPacket::from_string(&mac_address, sep) {
                        Ok(packet) => {
                            if let Err(e) = packet.send_magic() {
                                eprintln!("❌ 魔术包发送失败: {:?}", e);
                            } else {
                                println!("✅ 魔术包已发出");
                            }
                        }
                        Err(_) => {
                            eprintln!("❌ MAC 地址格式解析失败，请检查配置");
                        }
                    }
                }
            }
            Ok(_) => {
                current_delay = 1;
            }
            Err(e) => {
                eprintln!("⚠️ 连接中断: {:?}, {}秒后重连...", e, current_delay);
                tokio::time::sleep(Duration::from_secs(current_delay)).await;
                current_delay = (current_delay * 2).min(max_delay);
            }
        }
    }
}
