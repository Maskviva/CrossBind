<h1 align="center">crossbind</h1>

<p align="center">
  <b>让协议版本和服务端不一致的基岩版客户端也能进服。</b>
</p>

<p align="center">
  <a href="../../actions/workflows/build.yml"><img src="../../actions/workflows/build.yml/badge.svg" alt="Build"></a>
  <a href="https://github.com/Maskviva/crossbind/releases"><img src="https://img.shields.io/github/v/release/Maskviva/crossbind?color=334155" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue" alt="Apache-2.0"></a>
  <img src="https://img.shields.io/badge/Pier-26.20.1-62B47A" alt="Pier 26.20.1">
  <img src="https://img.shields.io/badge/LeviLamina-26.20.4-8B5CF6" alt="LeviLamina 26.20.4">
</p>

crossbind 在字节层拦下每一个数据包，按版本差异逐字段改写，然后放行。服务端本身完全不
知道客户端是别的版本。思路和 ViaVersion 一样，只是跑在基岩版上。

它是一个 [Pier](https://github.com/Maskviva/pier) 模组，用 Rust 写。

## 安装

用 [lip](https://lip.futrime.com)：

```bash
lip install github.com/Maskviva/crossbind
```

或者把发布压缩包解压到 `mods/crossbind/`。

需要先装好 [Pier](https://github.com/Maskviva/pier) 26.20.1，它是让 LeviLamina 能装载
Rust 模组的装载器。没有它，crossbind 根本不会被扫到。

装上之后不需要配置。服务端启动时它会打印自己认得哪些版本，之后按连进来的客户端自动选
翻译链。

## 覆盖范围

**能互通的版本**，任意两两组合：

|   协议 | Minecraft |
|-----:|-----------|
| 2169 | 1.26.45   |
| 2168 | 1.26.40   |
| 1001 | 1.26.30   |
|  975 | 1.26.20   |
|  944 | 1.26.10   |
|  924 | 1.26.0    |
|  898 | 1.21.130  |
|  860 | 1.21.124  |
|  859 | 1.21.120  |

相邻版本各有一个双向翻译步骤，不相邻的走链式串联——v859 的客户端连 v2169 的服务端会
经过八跳。

**认得但翻译不了的版本**：844 / 827 / 819 / 818 / 800 / 786 / 776 / 766 / 748 / 729。
它们只在版本表里登记了名字，用来把拒绝理由说清楚，没有翻译逻辑。往下扩每加一个版本，
都需要那一档边界上确切的线格式差异，而那不是读代码能读出来的。

## 它做不到什么

**不改服务端行为。** 客户端看到的世界仍然是服务端那个版本生成的。新版本才有的方块，在
旧客户端上是它翻译得到的最接近的那个，不是原样。

**不能凭空造出不存在的东西。** 一条 v2169 才有的配方，在 v1001 的客户端上会被整条丢
掉，而不是变成一条半对的配方。丢弃是有意的：一条形状对不上的配方会让客户端的合成界面
错乱，而少一条配方只是少一条。

**翻译不了的版本就是进不来。** 那时客户端看到的是正常的版本不匹配提示，和没装
crossbind 时一样。

## 架构

```
crates/
  bedrock-codec       线格式的读写：primitives、item、nbt、commands 等类型编解码
  bedrock-protocol    版本图、包 id 表、翻译步骤
  crossbind-mod       Pier 模组本体：挂上数据包拦截，按连接选翻译链
```

`bedrock-protocol` 里的两个目录分工是硬的：

- **`steps/`** 只放协议转换逻辑。一个目录一个版本对，目录里按数据包分文件，`mod.rs`
  只做注册、不含任何转换。
- **`convert/`** 放被 steps 调用的转换工具。它们不注册任何包，也不认识版本图。

数据表不放代码里。`data/*.tsv` 用 `include_str!` 读进来，代码侧只有解析和索引。

## 从源码构建

```bash
cargo build --release -p crossbind-mod                    # 服务端
cargo build --release -p crossbind-mod --features client  # 客户端
```

产物是 `target/release/crossbind.dll`，和 `manifest.json` 一起放进 `mods/crossbind/`。

`client` 只置模组 vtable 里 `mod_flags` 的一位，不增删任何 API，所以同一份源码两个目标
都编得过；装错目标会在 Pier 的握手阶段被明确拒绝。

```bash
cargo test -p bedrock-codec -p bedrock-protocol
```

翻译步骤的测试是往返式的：构造一个版本的字节，翻过去再翻回来，比对是否一致。协议转换
器最容易出的错是「翻过去看起来对，翻回来少了一个字段」，只有往返能抓到。

## 参与开发

`devdocs/` 有各 crate 的说明和 `ADDING-A-VERSION.md`。

## 许可

Apache-2.0，见 [LICENSE](LICENSE)。
