# mqtt-filelog

A simple but stable MQTT subscriber which writes incoming messages into
a predefined output file. Helpful to easily establish some kind of file
logging for an MQTT broker.


## Features

```
TODO: Define list of features
```

* SSL
* Limited output format options


## Output format

The output format is configurable with limited options:


```
  %ts     Unix Timestamp
  %#      Topic
  %raw    Payload (raw)
  %b64    Payload (Base64 encoded)
```

The default output format is defined as follows:

```
%ts %# %b64
```

