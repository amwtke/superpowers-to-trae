---

# DDD methodology (addon)

> 本项目通过 `superpowers-trae --addons ddd` 安装了 DDD harness。
> 任何代码生成前，请先确认 DDD 工作流前置阶段已完成。

## DDD 工作流（在 superpowers brainstorming 之前）

代码实现必须经过这三个战略阶段：

1. **事件风暴**：用户用 `/ddd-storm <业务描述>` 或说"使用 ddd-storm ..."；
   产出 `docs/ddd/01-event-storming-*.md`。
   你必须 `Read` `.trae/skills/ddd/ddd-storm/SKILL.md` 后按其流程执行。
2. **领域建模**：用 `/ddd-model` 或"使用 ddd-model ..."；
   维护 `DOMAIN.md`（Single Source of Truth）。
   `Read` `.trae/skills/ddd/ddd-model/SKILL.md` 后执行。
3. **生成 spec**：用 `/ddd-spec <场景>` 或"使用 ddd-spec ..."；
   产出 `docs/specs/spec-*.md`，作为 superpowers brainstorming 的输入。
   `Read` `.trae/skills/ddd/ddd-spec/SKILL.md` 后执行。

完成后才进入 superpowers `brainstorming → writing-plans → executing-plans` 流程。

## 强制约束（Hard Rules）

### R1. 战略层先行 + 技术栈先决策
- 任何新特性必须按 `/ddd-storm → /ddd-model → /ddd-spec → superpowers brainstorming` 顺序进行。
- `DOMAIN.md` 的"技术栈约定"段未填前，**禁止** superpowers `writing-plans`（含实现）。
- 用户要求跳步，请明确指出违反 harness 约定。

### R2. 术语一致性（Ubiquitous Language）
- 所有代码命名必须引用 `DOMAIN.md` 中的领域术语表。
- 发现代码命名与 DOMAIN.md 不一致 → **停下来询问用户**，不要自作主张修改。

### R3. 富领域模型（禁止贫血）
- 聚合根封装业务行为；Application Service 只做编排（取聚合 / 调方法 / 持久化 / 发事件 / 事务）。
- `if/else`、`for` 中含业务判断的代码必须在 Domain 层，不在 Application。

### R4. 聚合边界
- 一个事务只修改**一个**聚合实例；跨聚合协作用领域事件，不用同步调用。
- 聚合之间只引用 ID，不持有对象。

### R5. Repository 只对聚合根
- 不为 VO / 聚合内子实体单独建 Repository。

### R6. TDD 节奏
- 进入 superpowers `executing-plans` 后严格 spec → test → code 节奏。
- 一次一个 spec：先写测试 → 失败 → 最小代码通过 → 重构 → 下一个。
- **禁止一次性生成整套代码**。

### R7. 包结构 / 分层
- 必须有 4 层：interfaces / application / domain / infrastructure。
- 依赖方向：上层依赖下层；Domain 层不依赖任何其他层。
- 具体语言/框架的目录约定由 `brainstorming` 阶段在 `DOMAIN.md` 技术栈段确定后填回。

## 修改 DOMAIN.md 的流程

`DOMAIN.md` 是领域模型 SSOT，**不得随意修改**。允许的修改路径：

1. 通过 `/ddd-model` 重新建模（推荐）
2. 在 `/ddd-spec` 过程中发现缺失，**停下先修 DOMAIN.md 再继续**

禁止：
- superpowers 实现过程中擅自修改 DOMAIN.md
- 为了让代码通过测试而改 DOMAIN.md 术语

## 代码质量底线

- 每个聚合必须有单元测试（覆盖不变式）
- 每个 Application Service 方法必须有集成测试
- 测试命名使用业务语言（如 `shouldRedeemPointsWhenBalanceIsSufficient`）
- 禁止魔法数字
- 禁止 public 字段（除语言 record/struct 字段约定）
