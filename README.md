# Kaminari

[![workflow](https://github.com/zephyrchien/kaminari/workflows/release/badge.svg)](https://github.com/zephyrchien/kaminari/actions)
[![crates.io](https://img.shields.io/crates/v/kaminari.svg)](https://crates.io/crates/kaminari)
[![downloads](https://img.shields.io/github/downloads/zephyrchien/kaminari/total?color=green)](https://github.com/zephyrchien/kaminari/releases)
[![telegram](https://img.shields.io/badge/-telegram-blue?style=flat&color=grey&logo=telegram)](https://t.me/+zKbZTvQE2XtiYmIx)

[English](README.md) | [简体中文](README-zh.md)

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

Below is a list of availabe options, `*` means **must**.

### Websocket Options

use `ws` to enable websocket.

Client or server side options:

- `host=<host>`* : set http host.

- `path=<path>`* : set http path.

Client side extra options:

- `mask=<mode>` : set mask mode. Available values: [skipped, standard, fixed]

#### About Mask Mode

A websocket client should mask the payload before sending it.

With `mode=skip`(default mode), we use an empty mask key(0x00..0) to simply skip masking, which can also be detected by our server, and then skip unmasking. Other softwares(Nginx, Haproxy, CDNs..) can still correctly handle our data without knowing this trick.

As for `mode=fixed` or `mode=standard`, client will mask the payload data as normal. In `fixed` mode, client will use the same mask key for a unique websocket connection. While In `standard` mode, client will update the mask key between sending each frames.

### TLS Options

use `tls` to enable tls.

Client side options:

- `sni=<sni>`* : set sni.

- `alpn=<alpn>`: set alpn. e.g.: `h2,http/1.1`.

- `0rtt`: enable early data.

- `insecure`: skip server cert verification.

Server side options:

Requires either `cert+key` or `servername`.

- `key=<path/to/key>`* : private key path.

- `cert=<path/to/cert>`* : certificate path.

- `servername=<name>`* : generate self signed cert/key, use $name as CN.

- `ocsp=<path/to/ocsp>`: der-encoded OCSP response.

#### OCSP Stapling

See [Wikipedia](https://en.wikipedia.org/wiki/OCSP_stapling).

Openssl example for [Let's Encrypt](https://letsencrypt.org/):

```shell
openssl ocsp -issuer <path/to/ca> \
    -cert <path/to/cert> \
    -url http://r3.o.lencr.org \
    -header Host=r3.o.lencr.org \
    -respout <path/to/ocsp> -noverify -no_nonce
```

### Examples

tcp ⇋ ws --- ws ⇋ tcp:

```shell
kaminaric 127.0.0.1:10000 127.0.0.1:20000 'ws;host=example.com;path=/ws'

kaminaris 127.0.0.1:20000 127.0.0.1:30000 'ws;host=example.com;path=/ws'
```

tcp ⇋ tls --- tls ⇋ tcp:

```shell
kaminaric 127.0.0.1:10000 127.0.0.1:20000 'tls;sni=example.com'

# use cert + key
kaminaris 127.0.0.1:20000 127.0.0.1:30000 'tls;cert=example.com.crt;key=example.com.key'

# or generate self signed cert/key
kaminaris 127.0.0.1:20000 127.0.0.1:30000 'tls;servername=example.com'
```

tcp ⇋ wss --- wss ⇋ tcp:

```shell
kaminaric 127.0.0.1:10000 127.0.0.1:20000 'ws;host=example.com;path=/ws;tls;sni=example.com'

# use cert + key
kaminaris 127.0.0.1:20000 127.0.0.1:30000 'ws;host=example.com;path=/ws;tls;cert=example.com.crt;key=example.com.key'

# or generate self signed cert/key
kaminaris 127.0.0.1:20000 127.0.0.1:30000 'ws;host=example.com;path=/ws;tls;servername=example.com'
```

shadowsocks plugin:

```shell
ssserver -s "0.0.0.0:8080" -m "aes-128-gcm" -k "123456" \
    --plugin "path/to/kaminaris" \
    --plugin-opts "ws;host=example.com;path=/chat"
```

```shell
sslocal -b "127.0.0.1:1080" -s "example.com:8080" -m "aes-128-gcm" -k "123456" \
    --plugin "path/to/kaminaric" \
    --plugin-opts "ws;host=example.com;path=/chat"
```

*To use `v2ray-plugin` on client side, add `mux=0` to disable multiplex, so that it sends standard websocket stream which can be handled by `kaminari` or any other middlewares.

```shell
sslocal -b "127.0.0.1:1080" -s "example.com:8080" -m "aes-128-gcm" -k "123456" \
    --plugin "path/to/v2ray-plugin" \
    --plugin-opts "mux=0;host=example.com;path=/chat"
```
