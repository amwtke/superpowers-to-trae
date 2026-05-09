---

# Bob 4 环 Clean Architecture (addon)

> 本项目通过 `superpowers-trae --addons bob` 安装了 Bob harness。
> 任何代码生成前，请先确认 Bob 工作流前置阶段已完成。

## Bob 工作流（在 superpowers brainstorming 之前）

代码实现必须经过这三个战略阶段：

1. **身份测试**：用户用 `/bob-identify <业务描述>` 或说"使用 bob-identify ..."；
   产出 `docs/bob/01-identity-*.md`，把每个候选概念分类为 CORE / ADAPTER / FRAMEWORK / TOOL / 违规。
   你必须 `Read` `.trae/skills/bob/bob-identify/SKILL.md` 后按其流程执行。
2. **4 环架构设计**：用 `/bob-onion` 或"使用 bob-onion ..."；
   维护 `BOB.md`（4 环 Clean Architecture Single Source of Truth）。
   `Read` `.trae/skills/bob/bob-onion/SKILL.md` 后执行。
3. **生成 spec**：用 `/bob-spec <用例名>` 或"使用 bob-spec ..."；
   产出 `docs/specs/spec-*.md`，作为 superpowers brainstorming 的输入。
   `Read` `.trae/skills/bob/bob-spec/SKILL.md` 后执行。

完成后才进入 superpowers `brainstorming → writing-plans → executing-plans` 流程。

## 强制约束（Hard Rules）

### R0. 通用判定优先于具体清单
- 4 环依赖方向铁律：只能由外向内。`entity` 不 import 任何东西；
  `usecase` 只 import `entity`；`adapter` 可 import `usecase` 与 `entity`；
  `framework` 可 import 一切。
- 任何新外部库 / 新注解 / 新框架先跑 `BOB.md §配件清单` 扩充，
  再跑 5 问决策树（详见 `/bob-identify` skill）。

### R1. 战略层先行 + 技术栈先决策
- 任何新特性必须按 `/bob-identify → /bob-onion → /bob-spec → superpowers brainstorming` 顺序进行。
- `BOB.md` 的"技术栈约定"段未填前，**禁止** superpowers `writing-plans`（含实现）。
- 用户要求跳步，请明确指出违反 harness 约定。

### R2. 术语一致性
- 所有代码命名必须引用 `BOB.md` 中的：§3 Entity 名 + 状态名 + 方法名；
  §4 端口接口名 + 方法签名；§5 UseCase 类名 + Command/Result record 名。
- 发现代码命名与 BOB.md 不一致 → **停下来询问用户**，不要自作主张修改。

### R3. 富 Entity 模型（禁止贫血）
- Entity 封装业务行为；UseCase 只做编排（取 Entity / 调方法 / 持久化 / 返回 Result）。
- `if/else`、`for` 中含业务判断的代码必须在 Entity 层，不在 UseCase。

### R4. 框架边界外推
- `usecase/**` 包：**禁止**任何 Spring / Jakarta / SLF4J / Lombok import 或注解。
- usecase 是纯 Java POJO，通过构造器注入端口接口。
- 日志需求通过 `LoggerPort` 抽象，不允许直接 `import org.slf4j.*`。

### R5. 接口位置反转
- Business interface（端口）定义在 `usecase/port/`，由 Adapter 来 implement。
- 禁止把 Repository / Gateway 接口放在 `adapter/` 包下。

### R6. 状态机上提
- 业务规则在 Entity 内部，不在 Service / UseCase。
- 非法状态迁移必须 throw `IllegalStateException`。
- Entity 不得 `LocalDateTime.now()` / `UUID.randomUUID()` —— 用 `ClockPort` / `IdGenerator`。

### R7. 唯一事务边界
- 全工程**唯一**的 `@Transactional` 出现在 `shared.framework.transaction.TransactionalUseCaseDecorator`。
- UseCase 自身禁止 `@Transactional`、禁止 `TransactionTemplate`。

### R8. TDD 节奏
- 进入 superpowers `executing-plans` 后严格 spec → test → code 节奏。
- 一次一个 spec：先写测试 → 失败 → 最小代码通过 → 重构 → 下一个。
- **禁止一次性生成整套代码**。

### R9. 包结构 / 分层
- 必须有 4 环：entity / usecase / adapter / framework。
- 具体语言/框架的目录约定由 `brainstorming` 阶段在 `BOB.md` 技术栈段确定后填回。

## 修改 BOB.md 的流程

`BOB.md` 是 4 环架构 SSOT，**不得随意修改**。允许的修改路径：

1. 通过 `/bob-onion --refresh` 增补（推荐）
2. 在 `/bob-spec` 过程中发现缺失，**停下先修 BOB.md 再继续**

禁止：
- superpowers 实现过程中擅自修改 BOB.md
- 为了让代码通过测试而改 BOB.md 术语 / 端口签名

## 代码质量底线

- 每个 Entity 必须有单元测试（覆盖状态机不变式）
- 每个 UseCase 必须有集成测试（验证端口编排）
- 测试命名使用业务语言（如 `shouldRedeemPointsWhenBalanceIsSufficient`）
- 禁止魔法数字
- 禁止 public 字段（除 record 组件）
