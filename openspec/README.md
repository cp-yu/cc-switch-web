# OpenSpec Baseline

## Purpose

这份目录是 `cc-switch / CC Switch Fork` 当前的 OpenSpec 主基线，不是临时 bootstrap 残留。

## What Exists

- `config.yaml`: OpenSpec schema、文档语言和项目上下文约束。
- `project.opsx.yaml`: 正式架构图主文件，项目标识为 `cc-switch` / `CC Switch Fork`。
- `project.opsx.relations.yaml`: domain 与 capability 之间的结构关系。
- `project.opsx.code-map.yaml`: capability 到代码锚点的映射。
- `specs/`: capability 粒度的主行为规范。
- `specs/README.md`: `spec slug` 到 `cap.*` capability ID 的映射索引。

## Usage Contract

- 新 change 的 delta specs 应复用 `specs/README.md` 里现有的 `spec slug`。
- 架构相关引用优先使用 `project.opsx*.yaml`，行为相关引用优先使用 `specs/*/spec.md`。
- 生成 proposal、design、tasks 或 delta specs 时，应遵守 `config.yaml` 中的 `context` 和 `docLanguage`。

## Validation

当前主 specs 已通过：

```bash
openspec validate --specs --json
```
