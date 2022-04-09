# Kaminari

The ever fast websocket tunnel built on top of [lightws](https://github.com/zephyrchien/lightws).

## Feature

- Client side accepts tcp and sends [tcp/ws/tls/wss].

- Server side accepts [tcp/ws/tls/wss] and sends tcp.

- Options for tcp/ws/tls/wss are defined in the 3rd argument.

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

Command line:

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

- `host=<host>`: set http host

- `path=<path>`: set http path

### TLS Options

use `tls` to enable tls.

Client side options:

- `sni=<sni>`: set sni

- `insecure`: skip server cert verification

Server side options:

- `key=<path/to/key>`: private key path

- `cert=<path/to/cert>`: certificate path

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
