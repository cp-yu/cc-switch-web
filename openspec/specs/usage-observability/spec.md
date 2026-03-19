# Usage Observability Specification

## Purpose

`cap.proxy.usage` 定义用量统计、价格配置、请求日志与健康观测行为。
它把成本、请求明细与健康信号收敛到同一观测面。


## Requirements

### Requirement: Usage Reporting

系统 SHALL 提供聚合后的用量、趋势、供应商统计和模型统计视图。

#### Scenario: Reading usage summary

- **WHEN** 用户查询用量面板数据
- **THEN** 系统返回汇总指标、趋势或统计结果
- **AND** 数据可按应用或供应商维度过滤

### Requirement: Request-Level Inspection

系统 SHALL 记录可查询的请求日志与请求详情，以支持问题排查和成本核对。

#### Scenario: Inspecting request logs

- **WHEN** 用户查看请求日志或某条请求详情
- **THEN** 系统返回对应日志列表或明细内容
- **AND** 保留与供应商和模型相关的上下文信息
