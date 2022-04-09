# Kaminari

The ever fast websocket tunnel built on top of [lightws](https://github.com/zephyrchien/lightws).

## Intro

- Client side receives tcp then sends [tcp/ws/tls/wss].

- Server side receives [tcp/ws/tls/wss] then sends tcp.

- Compatible with shadowsocks [SIP003 plugin](https://shadowsocks.org/en/wiki/Plugin.html).

```text
 tcp                           ws/tls/wss                           tcp
 ===                          ============                          ===
        +-------------------+              +-------------------+
        |                   |              |                   |
+------->                   +-------------->                   +------->
        |     kaminaric     |              |     kaminaris     |
<-------+                   <--------------+                   <-------+
        |                   |              |                   |
        +-------------------+              +-------------------+       
```

## Usage

Standalone:

```shell
kaminaric <local_addr> <remote_addr> <options>

kaminaris <local_addr> <remote_addr> <options>
```

As shadowsocks plugin:

```shell
sslocal ... --plugin <path/to/kaminaric> --plugin-opts <options>

ssserver ... --plugin <path/to/kaminaris> --plugin-opts <options>
```

## Options

All options are presented in a single formatted string. An example is "ws;path=/ws;host=example.com", where semicolons, equal signs and backslashes MUST be escaped with a backslash.

### Websocket Options

use `ws` to enable websocket.

Client or server side options:

- `host=<host>`: set http host.

- `path=<path>`: set http path.

### TLS Options

use `tls` to enable tls.

Client side options:

- `sni=<sni>`: set sni.

- `0rtt`: enable early data.

- `insecure`: skip server cert verification.

Server side options:

- `key=<path/to/key>`: private key path.

- `cert=<path/to/cert>`: certificate path.

- `servername=<name>`: generate self signed cert/key, use $name as CN.

### Examples

tcp ⇋ ws --- ws ⇋ tcp:

```shell
kaminaric 127.0.0.1:10000 127.0.0.1:20000 'ws;host=example.com;path=/ws'

kaminaris 127.0.0.1:20000 127.0.0.1:30000 'ws;host=example.com;path=/ws'
```

tcp ⇋ tls --- tls ⇋ tcp:

```shell
kaminaric 127.0.0.1:10000 127.0.0.1:20000 'tls;sni=example.com;insecure'

# use cert + key
kaminaris 127.0.0.1:20000 127.0.0.1:30000 'tls;cert=example.com.crt;key=example.com.key'

# generate self signed cert/key
kaminaris 127.0.0.1:20000 127.0.0.1:30000 'tls;servername=example.com'
```

tcp ⇋ wss --- wss ⇋ tcp:

```shell
kaminaric 127.0.0.1:10000 127.0.0.1:20000 'ws;host=example.com;path=/ws;tls;sni=example.com;insecure'

# use cert + key
kaminaris 127.0.0.1:20000 127.0.0.1:30000 'ws;host=example.com;path=/ws;tls;cert=example.com.crt;key=example.com.key'

# generate self signed cert/key
kaminaris 127.0.0.1:20000 127.0.0.1:30000 'ws;host=example.com;path=/ws;tls;servername=example.com'
```
