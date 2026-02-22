# 使用方法
```
docker run -d \
  --name mqtt-wol 
  -e MQTT_PRIVATE_KEY=你的私钥 \
  -e MQTT_WOL_TOPIC=你的topic \
  -e WOL_MAC_ADDRESS=AA:BB:CC:DD:EE:FF \
  --network host \
  lentin/mqtt-wol
```
巴法云与米家关联相方法参考  
https://blog.csdn.net/m0_60388586/article/details/150111899#t1
