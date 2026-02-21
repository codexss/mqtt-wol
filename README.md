# 使用方法

```
docker run -d \
  --name mqtt-wol 
  -e BEMFA_PRIVATE_KEY=你的私钥 \
  -e TOPIC=你的topic \
  -e MAC_ADDRESS=AA:BB:CC:DD:EE:FF \
  --network host \
  mqtt-wol
```