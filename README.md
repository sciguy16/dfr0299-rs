https://wiki.dfrobot.com/DFPlayer_Mini_SKU_DFR0299

```
cargo r --features=mio-serial --example serial_port
```

Note that the example serialised messages in the datasheet are have
incorrect checksums. The checksum algorithm is not described in the
datasheet but is present in the
[official Arduino library code](https://github.com/DFRobot/DFRobotDFPlayerMini/blob/master/DFRobotDFPlayerMini.cpp)

# License
This crate is distributed under the terms of the Mozilla Public License
Version 2.0.
