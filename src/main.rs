use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::env;
use std::time::Duration;
use wakey::WolPacket;

const MQTT_HOST: &str = "bemfa.com";
const MQTT_PORT: u16 = 9501;
const KEEP_ALIVE_SECS: u64 = 30;
const INITIAL_RECONNECT_DELAY_SECS: u64 = 1;
const MAX_RECONNECT_DELAY_SECS: u64 = 60;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let private_key = env::var("MQTT_PRIVATE_KEY").expect("环境变量 MQTT_PRIVATE_KEY 未设置");
    let topic = env::var("MQTT_WOL_TOPIC").expect("环境变量 MQTT_WOL_TOPIC 未设置");
    let mac_address = env::var("WOL_MAC_ADDRESS").expect("环境变量 WOL_MAC_ADDRESS 未设置");

    let wol_packet = parse_wol_packet(&mac_address);

    println!("🚀 MQTT-WOL 服务已启动");

    let mut current_delay = INITIAL_RECONNECT_DELAY_SECS;

    loop {
        let mut mqtt_options = MqttOptions::new(&private_key, MQTT_HOST, MQTT_PORT);
        mqtt_options.set_keep_alive(Duration::from_secs(KEEP_ALIVE_SECS));

        let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);

        let mut subscribed = false;

        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(Packet::ConnAck(_))) => {
                    println!("✅ 已连接到 MQTT 服务器");
                    current_delay = INITIAL_RECONNECT_DELAY_SECS;

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
                    current_delay = INITIAL_RECONNECT_DELAY_SECS;

                    let msg = String::from_utf8_lossy(&publish.payload);

                    if should_wake(msg.as_ref()) {
                        println!("📢 收到唤醒指令: {}", msg.trim());

                        if let Err(e) = wol_packet.send_magic() {
                            eprintln!("❌ 魔术包发送失败: {:?}", e);
                        } else {
                            println!("✅ 魔术包已发出");
                        }
                    }
                }

                Ok(Event::Incoming(_)) | Ok(Event::Outgoing(_)) => {
                    current_delay = INITIAL_RECONNECT_DELAY_SECS;
                }

                Err(e) => {
                    eprintln!("⚠️ 连接异常: {:?}，{} 秒后重连...", e, current_delay);
                    break;
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(current_delay)).await;
        current_delay = (current_delay * 2).min(MAX_RECONNECT_DELAY_SECS);
    }
}

fn parse_wol_packet(mac_address: &str) -> WolPacket {
    let sep = if mac_address.contains('-') { '-' } else { ':' };

    WolPacket::from_string(mac_address, sep)
        .unwrap_or_else(|_| panic!("环境变量 WOL_MAC_ADDRESS 格式不正确: {mac_address}"))
}

fn should_wake(message: &str) -> bool {
    matches!(
        message.trim().to_ascii_lowercase().as_str(),
        "on" | "1" | "true"
    )
}

#[cfg(test)]
mod tests {
    use super::should_wake;

    #[test]
    fn should_wake_with_supported_values() {
        assert!(should_wake("on"));
        assert!(should_wake("ON"));
        assert!(should_wake("  true  "));
        assert!(should_wake("1"));
    }

    #[test]
    fn should_not_wake_with_other_values() {
        assert!(!should_wake("off"));
        assert!(!should_wake("0"));
        assert!(!should_wake(""));
    }
}
