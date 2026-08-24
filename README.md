# crossbind

跨版本协议适配 —— 让协议版本和服务端不一致的基岩版客户端也能进服。用 Rust 写的
LeviLamina mod。

思路和 ViaVersion 一样：在字节层拦下每个包，按版本差异逐字段改写，然后放行。服务端
本身完全不知道客户端是别的版本。

## 覆盖范围

**能互通的版本**（任意两两组合，靠链式串联）：

|   协议 | Minecraft |
|-----:|-----------|
| 2168 | 1.26.40   |
| 1001 | 1.26.30   |
|  975 | 1.26.20   |
|  944 | 1.26.10   |
|  924 | 1.26.0    |
|  898 | 1.21.130  |
|  860 | 1.21.124  |
|  859 | 1.21.120  |

相邻版本各有一个双向翻译步骤，不相邻的走链式串联 —— 比如 v859 客户端连 v2168 服务端
会经过七跳。

**认得但翻译不了的版本**：844 / 827 / 819 / 818 / 800 / 786 / 776 / 766 / 729。这些只在
`versions.rs` 里登记了版本名，用来把拒绝理由说清楚，**没有**翻译逻辑。

为什么不做？往下扩每加一个版本，都需要那一档边界上确切的线格式差异。我手上没有 844
及更早版本的权威依据，凭印象编出来的 handler 只会把包改坏 —— 那比不做更糟，因为报错
会指向完全无关的地方。

## 已知缺口

翻译是尽力而为，不是完备的。目前明确知道不完整的地方：

- **v944 ↔ v975 的物品栏**。v975 换了物品堆的编码（定长 i16 id、没有空气短路、网络 id
  多了个变体标签）。`PlayerEquipment` / `AddPlayer` / `AddItemActor` 里的物品已经转了，
  但 `InventoryContent` / `InventorySlot` / `CraftingData` / 物品交互那一族**没有** ——
  这些包里物品前面的字段在两个版本之间也动过，我没有可靠依据，硬猜会把每一次格子更新
  都改坏。所以这对版本组合下容器里的物品会显示错乱。
- **v860 ↔ v898 的 `AvailableCommands` / `CommandOutput`**。类型（`commands.rs`）齐了并
  且有测试，但包级 handler 还没接上，所以这对组合下命令补全会不正常。
- **v1001 ↔ v2168（1.26.30 ↔ 1.26.40）覆盖面明显低于其它档**。1.26.40 把大量序列化
  搬到了 Cereal 反射上，gophertunnel 侧一次动了 53 个文件，是本项目跨过的最大一道坎。
  登录/出生/移动这条链路是通的，但下面这些包**只丢不转**，因为它们的子结构是独立变的，
  半对的猜测会在离出错点很远的地方炸：`PlayerSkin`（重做后的皮肤块经由另一层外壳传递，
  那层外壳还没有实际抓包核对过）、`MapData`、
  `ClientboundUpdateSoundData`、`SubChunk`（条目里四个字段
  全变成了 optional）、`PlayerLocation`（唯一 id 和 type 换了位置、type 从定长 i32
  变成 varint；gophertunnel 自己的 v2168 marshal 还把 type 写了两遍，它的模型要么在
  描述一个真的重复字段、要么就是错的，两种猜法都不值得赌）。
  `SetScore`(108) / `SetScoreboardIdentity`(112) **已实现**，见 `steps/set_score_v2168.rs`。
  这两个包就是侧边栏：107 画标题、108 画下面每一行，所以单丢 108 的症状精确等于「计分板只有
  标题、一行都没有」。1.26.40 把 108 的每个条目改成了 Cereal 变体，判别符是
  **`varuint32` 下标 + 一个名字字符串**（`Remove` / `ChangePlayer` / `ChangeEntity` /
  `ChangeFakePlayer`），包头那个 `ScorePacketType` 字节整个没了；`Remove` 条目不再带 score，
  objective 名变成可选。112 相反，保留类型字节，只是每条的 player id 变成可选。
  `CraftingData`(52) **已实现**，见 `steps/crafting_data_v2168.rs`。一个配方表拆成了八个
  定型数组，而 `recipeType` 判别值**不是连续的**：`recipe.go` 的 `iota` 里有两个空位
  （炉子、炉子数据留下的），真实取值是 0, 1, 4, 5, 6, 7, 8, 9。另外配方里的物品描述符
  从数字变成了名字，靠 `ItemRegistry`(162) 缓存的注册表回填；查不到名字的成分只丢那一条
  配方，其余照发。
  `PlayerList`(63) **已实现**，见 `steps/player_list_v2168.rs`。action 判别符从包头搬进了
  每条 entry（v2168 两种情况都还在，所以往上转不需要拆包）；`PlayerColour` 只是字节序
  从小端变成大端；`EntityUniqueID` 的 `ActorUniqueID` 是纯改名，底下还是 `Varint64`。
  真正的工作量在皮肤块的十来处字段改动。唯一一处推断是 `PersonaPiece.PieceType`
  从名字变成了枚举号，gophertunnel 只给了枚举没给名字表；表里认不出的名字一律落到
  `PieceTypeUnknown`，不猜相邻值——这一项只影响 persona 皮肤外观，不影响条目本身。
  `ItemStackRequest`(147) / `ItemStackResponse`(148) **已实现**，见
  `steps/item_stack_v2168.rs`。动作的 type id 其实没有重编号 —— 变的是它现在按 Cereal
  变体表的下标发送，而那张表略去了客户端从不发送的两个容器动作，所以第 7 项往上整体
  少 2，是严格可逆的两行公式。真正容易漏的是 `StackRequestSlotInfo.StackNetworkID`
  从 varint 收窄成定长 i32：它嵌在九种动作里，错一处等于毁掉几乎所有请求。
  `CraftRecipeAuto`（Shift 点击合成）带配料描述符，整条请求丢弃。
  `CraftResultsDeprecated` 曾经**发空结果列表** —— 理由是"这个动作已弃用、内容冗余"。
  服务端无条件读取第一个结果，于是空指针解引用、**整个服务端崩溃**。"有损但无害"
  变成了比原 bug 严重得多的故障。现在它借助服务端自己发的物品注册表
  （`ItemRegistry`，id=162）把名字翻回数字 id；注册表里查不到的名字一律拒绝整条请求，
  绝不编造 id。写出的结果条数永远等于读入的条数，有回归测试盯着。
  `CraftingData`(52) 仍然丢弃，这是合成不了的直接原因 —— 它的配方输入输出走同一套
  描述符，需要把整张注册表映射铺到 `recipe.go` 那一族上，还没做。
  另外 1.26.40 有三处"看着像稳定尾巴、其实变了"的地方，都已经处理：
  `StartGame` 末尾的 `ServerJoinInformation`（**整个结构体外面还套着一层 optional**，
  内部四个字段又各自包进了 optional）、`CreativeContent`（分组 `Category` 从
  **定长 i32 小端**收窄成 byte，且里面的物品描述符取消了空气短路），以及 v1001 的
  游戏规则整数是**无符号 varint 而不是 zigzag**（读的字节数一样，所以不会错位，
  只会让客户端拿到一个没人发过的值）。出生流程里前两者都必发，漏掉任何一个都表现为
  "登录成功、进服前掉线"。
  `StructureBlockUpdate`(90) **已实现**。它唯一的变化是尾巴上的 `RedstoneSaveMode`
  从 varint 收窄成一个字节，而这个字段在 `StructureSettings` 那一大坨后面 —— 从包头
  走过去需要整个 settings 的线格式，这是它当初被丢掉的原因。现在改成从**尾部**定锚：
  末三字节是 `[RedstoneSaveMode][ShouldTrigger][Waterlogged]`，只重编中间那一个字节，
  长度不变，前面所有内容原样搬运、一个字节都不解析。动手前先校验（两个 bool 必须真是
  0/1、模式值在范围内、varint 那侧必须是单字节非负），校验不过就退回丢包 —— 也就是
  改动之前的行为，不会写出半成品。
- 目标版本没有的包会被**丢弃**而不是硬转。丢包损失一个功能，硬转损失整条连接。
- **翻译链上翻译失败也一样丢包。** 曾经是「转发原包」，理由是"原包只是版本不对，还能救、
  而且够吵"。这个理由是错的：把一份 v975 的包原样交给 v2168 客户端，对端不会看出这是
  另一个版本，它会按自己的布局去解析 —— 长度当成计数、计数当成长度 —— 结果不是少一个
  子区块，而是客户端当场死掉。而且链式翻译时它还会把前一跳已经翻好的结果一起扔掉。
  只有 base 步骤（登录改写）保留原语义：那一处丢包会让玩家连不进来，而转发原包只是
  退化成正常的"版本不匹配"界面。失败提示按「方向 + 包 id」每条连接只报一次。

## 依赖

需要 levilamina-rust-loader 26.20.4 以上的版本。

`crossbind-mod/Cargo.toml` 里的 `levilamina`
现在指向本地路径，按你的实际布局改，你也可以指向 [github仓库](https://github.com/Maskviva/levilamina-rust-loader)
或者使用 [crates.io上的levilamina包](https://crates.io/crates/levilamina)。

## 构建

```bash
cargo test --workspace          # 先跑测试，见下
cargo build --release -p crossbind-mod
```

**产物只有一个 DLL**：`target/release/crossbind.dll`。连同 `manifest.json` 一起丢进
`plugins/crossbind/` 就行。

workspace 里虽然有三个 crate，但只有 `crossbind-mod` 声明了 `crate-type = ["cdylib"]`；
另外两个是默认的 `rlib`，编译期就静态链进 DLL 里了。`target/release/` 下面你会看到
`libbedrock_codec.rlib` / `libbedrock_protocol.rlib`，那是中间产物不是插件；MSVC 还会
额外产出 `crossbind.dll.lib` / `.exp` / `.pdb`，同样不用管。

## 配置

不用配。服务端协议版本在 `on_enable` 里通过 `ctx.server().protocol_version()` 读出来
（底层是 `SharedConstants::NetworkProtocolVersion()`，走 loader 已有的 `server_info_str`
槽位，不需要新 ABI）。

有三个环境变量在极少数情况下需要动。

第一个是 `CROSSBIND_SUBCHUNK`，控制 `SubChunk` 条目往 1.26.40 转的形态。默认 `c`：
**hash 非零才公布给客户端**（全是空气的子区块 hash 为 0，那是填充值，不是真 blob）。

这一档有过一次失败的尝试，记在这里省得再走一遍：曾把默认改成 `e`（一个 hash 都不公布），
理由是兑现 hash 要走 `ClientCacheBlobStatus`(135) / `ClientCacheMissResponse`(136)，而
135 在这一档没 handler、136 在任何一档都没有。抓包否掉了：1.26.40 客户端登录、出生、
请求了 156 个子区块、收到四个各短了 617 字节的 `SubChunk`（156 × 8 字节 hash），然后在
出生后一秒左右直接退出世界。**包头说缓存开着、条目又带 payload 时，客户端要求这个条目
说出自己是哪个 blob。** 现在 `e` 只作为诊断项保留。

其余取值：`a` 每个条目都公布（gophertunnel 原样模型，会让客户端去要一个 id 为 0 的
blob 然后卡住）、`b`/`d`/`f` 是 `a`/`c`/`e` 再去掉高度图类型字节、`air` 只保留框架不发
方块（诊断用）、`off` 直接丢掉整个包。

第二个是 `CROSSBIND_SET_SCORE`。**默认就是翻译，不用配。** 只有 `off`（含 `0`/`false`/`drop`）
会让 `SetScore`(108) / `SetScoreboardIdentity`(112) 退回丢弃，代价是 1.26.4x 客户端的计分板
只有标题、一行都没有。

之前这里有一整套候选布局阶梯（`probe`/`a`~`g`），现在全删了：v2168 的线格式已经拿到权威依据，
不用再猜。写 `probe` 或任何旧档位名现在都等于"开"。

第三个是 `CROSSBIND_PLAYSOUND_LOOPS`。1.26.40 给 `PlaySound`
加了 `LoopCount`，`-1` 是「永远循环」，`0` 是「放一遍」。老版本的包里没有这个字段，往上
转的时候必须凭空补一个，现在补的是 `0`。**只有当你发现 1.26.40 客户端那边音效一声都不响
时**才把它设成 `1`（那说明这个字段数的是「播放次数」而不是「额外循环次数」）：

```bash
CROSSBIND_PLAYSOUND_LOOPS=1 ./bedrock_server
```

如果服务端本身跑在不能翻译的版本上（比如 1.26.30 / 协议 1001），mod 会打印一条警告然后
**不安装拦截器**。这是有意的：base 步骤会改写 `Login` 里的协议号来骗过 BDS 的版本检查，
如果后面没有翻译链兜着，等于把玩家放进来再喂他一堆看不懂的包 —— 那比让他看到一个正常的
"版本不匹配"界面糟得多。

## 诊断：抓那种「不报错但也活不下来」的包

翻译**失败**是会自己叫的：`run_steps` 会推一条 notice，日志里长这样：

```
v2168->v1001: failed to translate clientbound StartGame: unexpected end of packet: wanted 8, 7 left
```

真正难查的是另一半 —— 每个 handler 都返回 `Ok`，字节却仍然不是对面那个版本能读的形状。
这时候一行日志都没有，只看得到玩家连上、一秒后消失。症状和「服务端主动踢人」完全一样，
从四行日志倒推等于猜。

所以加了一个按需打开的逐包 trace，走的是现成的 notice 通道，loader 那边不用改：

```bash
CROSSBIND_TRACE=1 ./bedrock_server
# 可选：改上限，默认 20 万行
CROSSBIND_TRACE=1 CROSSBIND_TRACE_LIMIT=8000 ./bedrock_server
```

输出：

```
trace 0 clientbound id=11 StartGame [rewrite 4211 B -> 4198 B]
trace 1 clientbound id=162 Packet#162 [forward 88214 B]
trace 2 clientbound id=52 CraftingData [DROP]
trace 3 serverbound id=144 PlayerAuthInput [rewrite 74 B -> 71 B]
```

上限是必要的：出生时那一波是几千个包，不封顶的话最有价值的尾巴会被埋掉。
`Disconnect` 不受上限约束，永远打印，并且附带正文的可打印字符 —— 它回答的是 trace
其余部分回答不了的那个问题：

- **出现了 clientbound `Disconnect`** → 是**服务端**踢的人，正文里通常带原因。
  那么问题出在**上行**：我们把客户端的某个包翻译成了 BDS 读不懂的东西。
  这一对版本上最可疑的是 `PlayerAuthInput`（客户端每 tick 都在发）。
- **完全没有 `Disconnect`** → 是**客户端**自己走的，说明它读崩了某个下行包。
  那就看 trace 最后几行 —— 客户端掉线前收到的最后一个 id 就是要查的那个。

两种情况指向流水线的两半，先分清楚再动手，比继续按症状猜包快得多。

## 测试

```bash
cargo test --workspace
```

纯逻辑测试，不需要跑服务端 —— `bedrock-codec` 和 `bedrock-protocol` 都不依赖 loader，
这是刻意的。覆盖的是最容易出错也最难在线上发现的部分：

- **往返性**：`up` 之后 `down` 必须是恒等变换，否则包每过一跳链就漂一格
- **NBT 边界**：截断的输入必须报错而不是 panic；标签必须停在正确的位置
- **物品编码**：空气在两种编码之间往返必须还是空气（v944 一个字节，v975 六个字节）
- **负 Y 坐标**：v924 ↔ v944 的 BlockPos 编码变化只在 Y < 0 时才体现出来
- **链式路由**：任意两个支持的版本之间都要能找到路径，且是最短路径
- **失败模式**：坏包必须原样放行，不能吐出改了一半的字节
- **多跳链**：`start_game_survives_the_v975_to_v2168_chain` 走的是真实拓扑
  （服务端 975、客户端 2168，中间经 1001 两跳），而不是单步往返。单步往返对
  `StartGame` 尾部是盲区 —— 旧的 v975 fixture 在 `ServerAuthoritativeSound`
  之后就结束了，`passthrough_all` 对空输入是恒等的，所以整条尾巴从来没被测到过。
  新 fixture 带了填满的 `ServerJoinInformation` 和四个尾部 ID。
- **"全缺席也要对"**：一个 optional 三个成员都不设时，两个版本编码完全一致，
  只测那种情况的 handler 看起来永远是对的。所以 optional 一律测两次：全填满、全缺席，
  且全缺席那次要断言 handler 没有多吃后面的字段

## 结构

```
crates/
  bedrock-codec/       reader / writer / 字段类型体系（无依赖，可单独测）
  bedrock-protocol/    版本表、包 id、翻译步骤、链式路由（只依赖 codec）
  crossbind-mod/       cdylib，把上面两个接到 loader 的抓包钩子上
devdocs/               每个源文件一份设计说明（见下）
```

`bedrock-protocol` 不依赖 loader 是有意为之：翻译逻辑是这里最值得测也最容易写错的部分，
不该需要一个跑着的服务端才能验证。

## 致谢

本项目借鉴了 [EndstoneMC/endweave](https://github.com/EndstoneMC/endweave)：早期的低版本
翻译步骤（v859 ～ v1001 这几档）是照着它的 Python 实现移植过来的，整体架构则沿用
ViaVersion 的思路。1.26.4x（协议 2168）这一档是本项目自己做的，上游没有覆盖。

协议号 ↔ 版本号对照表最初参考了 GlacieTeam/ProtocolLib 的公开发布说明，但 924 以下那部分
和 gophertunnel 自己的提交历史对不上（例如把 898 标成了不存在的 "1.25.60"）。现在整张表
已经改成直接从 [gophertunnel](https://github.com/Sandertv/gophertunnel) 的
`minecraft/protocol/info.go` 提交历史逐条核对过的版本——那条线是 1.18 → 1.19 → 1.20 →
1.21.x（一路到 1.21.130）→ 1.26.0，中间没有 1.25.x。线格式本身的权威依据同样是
gophertunnel 的提交历史，1.26.30 ↔ 1.26.40 这一档的每处字段改动都对着 `b36ddad~1` 和
`HEAD` 逐行核对过。
