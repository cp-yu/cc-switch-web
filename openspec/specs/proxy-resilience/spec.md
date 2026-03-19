# Proxy Resilience Specification

## Purpose

`cap.proxy.resilience` 定义代理链路中的故障转移、断路器与上游切换行为。
它关注运行中切换上游时的韧性控制与故障隔离。


## Requirements

### Requirement: Provider Failover

系统 SHALL 在代理路径中支持上游供应商切换与故障转移，以降低单点失败影响。

#### Scenario: Switching proxy upstream provider

- **WHEN** 用户在代理运行时切换上游供应商
- **THEN** 系统更新代理目标到新的供应商
- **AND** 后续请求流向新的上游目标

### Requirement: Circuit Breaker Controls

系统 SHALL 暴露断路器配置与重置能力，以便在异常期间限制故障扩散。

#### Scenario: Resetting a circuit breaker

- **WHEN** 用户重置某个供应商的断路器状态
- **THEN** 系统清除对应的断路器故障状态
- **AND** 后续健康检查可重新评估该供应商
