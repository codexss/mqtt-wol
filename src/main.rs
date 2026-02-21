use rumqttc::{AsyncClient, MqttOptions, QoS, Event, Packet};
use std::env;
use std::time::Duration;
use wakey::WolPacket;

#[tokio::main]
async fn main() {
    let private_key = env::var("BEMFA_PRIVATE_KEY").expect("环境变量 BEMFA_PRIVATE_KEY 未设置");
    let topic = env::var("TOPIC").expect("环境变量 TOPIC 未设置");
    let mac_address = env::var("MAC_ADDRESS").expect("环境变量 MAC_ADDRESS 未设置");

    println!("🚀 MQTT-WOL 服务已启动");
    println!("📡 监听主题: {}", topic);
    println!("💻 目标 MAC: {}", mac_address);

    let mut mqtt_options = MqttOptions::new(&private_key, "bemfa.com", 9501);
    mqtt_options.set_keep_alive(Duration::from_secs(30));

    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);

    if let Err(e) = client.subscribe(&topic, QoS::AtMostOnce).await {
        eprintln!("❌ 订阅失败: {:?}", e);
    }

    // 自动识别分隔符 (支持 : 或 -)
    let sep = if mac_address.contains('-') { '-' } else { ':' };

    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::Publish(publish))) => {
                let msg = String::from_utf8_lossy(&publish.payload);
                if msg.trim() == "on" {
                    println!("📢 收到 'on' 指令，正在唤醒...");
                    match WolPacket::from_string(&mac_address, sep) {
                        Ok(packet) => {
                            let _ = packet.send_magic();
                            println!("✅ 魔术包已发出");
                        }
                        Err(_) => eprintln!("❌ MAC 地址格式错误: {}", mac_address),
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("⚠️ 连接中断: {:?}, 5秒后重连...", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}