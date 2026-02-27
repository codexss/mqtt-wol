use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::env;
use std::time::Duration;
use wakey::WolPacket;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let private_key = env::var("MQTT_PRIVATE_KEY").expect("环境变量 MQTT_PRIVATE_KEY 未设置");
    let topic = env::var("MQTT_WOL_TOPIC").expect("环境变量 MQTT_WOL_TOPIC 未设置");
    let mac_address = env::var("WOL_MAC_ADDRESS").expect("环境变量 WOL_MAC_ADDRESS 未设置");

    let sep = if mac_address.contains('-') { '-' } else { ':' };

    println!("🚀 MQTT-WOL 服务已启动");

    let mut current_delay: u64 = 1;
    let max_delay: u64 = 60;

    loop {
        let mut mqtt_options = MqttOptions::new(&private_key, "bemfa.com", 9501);
        mqtt_options.set_keep_alive(Duration::from_secs(30));

        let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);

        let mut subscribed = false;

        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(Packet::ConnAck(_))) => {
                    println!("✅ 已连接到 MQTT 服务器");
                    current_delay = 1;

                    // 连接建立后订阅一次
                    if !subscribed {
                        match client.subscribe(&topic, QoS::AtMostOnce).await {
                            Ok(_) => {
                                subscribed = true;
                                println!("📡 已订阅主题: {topic}");
                            }
                            Err(e) => {
                                eprintln!("❌ 订阅失败: {:?}", e);
                            }
                        }
                    }
                }

                Ok(Event::Incoming(Packet::SubAck(_))) => {
                    println!("✅ 主题订阅确认成功");
                }

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
                                eprintln!("❌ MAC 地址解析失败");
                            }
                        }
                    }
                }

                Ok(Event::Incoming(_)) | Ok(Event::Outgoing(_)) => {
                    current_delay = 1;
                }

                Err(e) => {
                    eprintln!("⚠️ 连接异常: {:?}，{} 秒后重连...", e, current_delay);
                    break;
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(current_delay)).await;
        current_delay = (current_delay * 2).min(max_delay);
    }
}


