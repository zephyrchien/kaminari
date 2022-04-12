# 电光石火

[![workflow](https://github.com/zephyrchien/kaminari/workflows/release/badge.svg)](https://github.com/zephyrchien/kaminari/actions)
[![crates.io](https://img.shields.io/crates/v/kaminari.svg)](https://crates.io/crates/kaminari)
[![downloads](https://img.shields.io/github/downloads/zephyrchien/kaminari/total?color=green)](https://github.com/zephyrchien/kaminari/releases)
[![telegram](https://img.shields.io/badge/-telegram-blue?style=flat&color=grey&logo=telegram)](https://t.me/+zKbZTvQE2XtiYmIx)

[English](README.md) | [简体中文](README-zh.md)

基于 [lightws](https://github.com/zephyrchien/lightws) 构建的 websocket 隧道工具.

## 简介

- 客户端接收tcp, 发送 [tcp/ws/tls/wss].

- 服务端接收 [tcp/ws/tls/wss], 发送 tcp.

- 兼容 shadowsocks [SIP003 plugin](https://shadowsocks.org/en/wiki/Plugin.html).

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

## 使用方法

单独运行:

```shell
kaminaric <local_addr> <remote_addr> <options>

kaminaris <local_addr> <remote_addr> <options>
```

作为 shadowsocks 插件运行:

```shell
sslocal ... --plugin <path/to/kaminaric> --plugin-opts <options>

ssserver ... --plugin <path/to/kaminaris> --plugin-opts <options>
```

## 选项及定义

所有的选项都包含在一个字符串内, 格式均为`key` 或 `key=value`, 各选项间用`;`分割.

示例:
"ws;path=/ws;host=example.com".

以下是完整的选项列表, 带 `*` 的为必要选项.

### Websocket 选项

添加 `ws` 以启用 websocket.

客户端、服务端通用选项:

- `host=<host>`* : 设置 http host.

- `path=<path>`* : 设置 http path.

### TLS 选项

添加 `tls` 以启用 tls.

客户端选项:

- `sni=<sni>`* : 设置发送的 sni.

- `0rtt`: 启用 early data.

- `insecure`: 跳过证书验证.

服务端选项:

必须提供证书和私匙路径, 或者域名(用于自签证书).

- `key=<path/to/key>`* : 私钥路径.

- `cert=<path/to/cert>`* : 证书路径.

- `servername=<name>`* : 自签证书, 以 $name 为域名.

### 示例

tcp ⇋ ws --- ws ⇋ tcp:

```shell
kaminaric 127.0.0.1:10000 127.0.0.1:20000 'ws;host=example.com;path=/ws'

kaminaris 127.0.0.1:20000 127.0.0.1:30000 'ws;host=example.com;path=/ws'
```

tcp ⇋ tls --- tls ⇋ tcp:

```shell
kaminaric 127.0.0.1:10000 127.0.0.1:20000 'tls;sni=example.com'

# 使用证书和私钥
kaminaris 127.0.0.1:20000 127.0.0.1:30000 'tls;cert=example.com.crt;key=example.com.key'

# 或者使用自签证书
kaminaris 127.0.0.1:20000 127.0.0.1:30000 'tls;servername=example.com'
```

tcp ⇋ wss --- wss ⇋ tcp:

```shell
kaminaric 127.0.0.1:10000 127.0.0.1:20000 'ws;host=example.com;path=/ws;tls;sni=example.com'

# 使用证书和私钥
kaminaris 127.0.0.1:20000 127.0.0.1:30000 'ws;host=example.com;path=/ws;tls;cert=example.com.crt;key=example.com.key'

# 或者使用自签证书
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

*如果要在客户端使用`v2ray-plugin`, 需要添加`mux=0`, 以禁用多路复用, 这样 `v2ray-plugin` 就会发送标准的 websocket.

```shell
sslocal -b "127.0.0.1:1080" -s "example.com:8080" -m "aes-128-gcm" -k "123456" \
    --plugin "path/to/v2ray-plugin" \
    --plugin-opts "mux=0;host=example.com;path=/chat"
```
