# API 参考

Base URL: `http://<host>:8001`

## 认证

- 未设置 `MEMOS_AUTH_TOKEN`：不需要鉴权（默认开发模式）
- 已设置 `MEMOS_AUTH_TOKEN`：所有 `/product/*` 端点都需要
  `Authorization: Bearer <token>`
- `GET /health` 不需要鉴权

## 通用响应结构

```json
{
  "code": 200,
  "message": "Success",
  "data": {}
}
```

## 请求追踪

- 所有接口支持可选请求头 `X-Request-Id`
- 若请求携带该头，服务端会原样透传到响应头
- 若未携带，服务端会自动生成并返回 `X-Request-Id`

## `POST /product/add`

写入记忆。

关键字段：

- `user_id` string 必填
- `async_mode` string，`sync` 或 `async`，默认 `sync`
- `messages` array，可选
- `memory_content` string，可选（当 `messages` 缺失时使用）
- `mem_cube_id` string，可选
- `writable_cube_ids` array，可选
- `relations` array，可选。用于在“新写入的记忆”和“已有记忆”之间建边：
  - `memory_id`：已有记忆 id
  - `relation`：关系类型字符串
  - `direction`：`outbound|inbound|both`，默认 `outbound`
  - `metadata`：可选边元数据

说明：如果 `async_mode=async`，返回 `task_id`，随后通过调度接口查询状态。

## `GET /product/scheduler/status`

查询异步写入任务。

Query 参数：

- `user_id` string 必填
- `task_id` string 必填

返回：

- 200：任务存在且归属该 `user_id`
- 404：任务不存在或非该 `user_id` 所有

## `POST /product/search`

检索记忆。

关键字段：

- `query` string 必填
- `user_id` string 必填
- `top_k` number，可选，默认 10
- `mem_cube_id` string，可选
- `readable_cube_ids` array，可选
- `filter` object，可选（会与服务端租户过滤合并）
- `relativity` number，可选（相似度阈值，`> 0` 时生效）

注意：服务端会强制注入 `mem_cube_id` 过滤，不能通过 `filter` 读取其他租户数据。

## `POST /product/update_memory`

更新已有记忆。

关键字段：

- `memory_id` string 必填
- `user_id` string 必填
- `memory` string 可选（更新文本并重建向量）
- `metadata` object 可选

## `POST /product/delete_memory`

删除记忆。

关键字段：

- `memory_id` string 必填
- `user_id` string 必填
- `soft` bool，可选，默认 `false`

行为：

- `soft=true`：标记 tombstone，并从向量库移除
- `soft=false`：图节点和向量都硬删除

## `POST /product/get_memory`

按 id 获取单条记忆。

关键字段：

- `memory_id` string 必填
- `user_id` string 必填
- `include_deleted` bool 可选，默认 `false`

## `POST /product/graph/neighbors`

按图关系查询某个记忆节点的邻居。

关键字段：

- `memory_id` string 必填（源节点）
- `user_id` string 必填
- `mem_cube_id` string，可选
- `relation` string，可选（按关系类型过滤）
- `direction` string，可选：`outbound|inbound|both`，默认 `outbound`
- `limit` number，可选，默认 `10`
- `cursor` string，可选。上一页返回的 `next_cursor`
- `include_deleted` bool，可选，默认 `false`

返回：

- `data.items` 为数组，每项包含：
  - `edge`：边信息（`id/from/to/relation/metadata`）
  - `memory`：邻居节点（`id/memory/metadata`）
- `data.next_cursor`：下一页游标；为 `null` 表示没有更多

错误码：

- `400`：参数非法（如 `cursor` 非数字）
- `404`：源节点不存在或无权限

## `POST /product/graph/path`

查询两个记忆节点之间的最短跳数路径（BFS）。

关键字段：

- `source_memory_id` string 必填
- `target_memory_id` string 必填
- `user_id` string 必填
- `mem_cube_id` string，可选
- `relation` string，可选（仅在该关系类型内找路径）
- `direction` string，可选：`outbound|inbound|both`，默认 `outbound`
- `max_depth` number，可选，默认 `6`
- `include_deleted` bool，可选，默认 `false`

返回：

- `data.hops`：路径边数
- `data.nodes`：按路径顺序的节点列表（含起点和终点）
- `data.edges`：按路径顺序的边列表

错误码：

- `404`：节点不存在/无权限，或路径不存在

## `POST /product/graph/paths`

查询两个记忆节点间的多条候选最短路径（按跳数优先，最多返回 `top_k_paths` 条）。

关键字段：

- `source_memory_id` string 必填
- `target_memory_id` string 必填
- `user_id` string 必填
- `mem_cube_id` string，可选
- `relation` string，可选
- `direction` string，可选：`outbound|inbound|both`，默认 `outbound`
- `max_depth` number，可选，默认 `6`
- `top_k_paths` number，可选，默认 `3`
- `include_deleted` bool，可选，默认 `false`

返回：

- `data` 为路径数组，每项结构与 `/product/graph/path` 的 `data` 一致（`hops/nodes/edges`）。

错误码：

- `400`：参数非法（如 `top_k_paths <= 0`）
- `404`：节点不存在/无权限，或无可用路径

## `GET /product/audit/list`

查询审计日志。

Query 参数（全可选）：

- `user_id`
- `cube_id`
- `since`（ISO8601）
- `limit`
- `offset`

## `GET /health`

返回 `ok`。

## `GET /metrics`

Prometheus 文本格式指标导出（`text/plain; version=0.0.4`）。

当前包含：

- `mem_api_requests_total`（按 endpoint/method/status）
- `mem_api_errors_total`（按 endpoint/method/status）
- `mem_api_request_duration_ms`（按 endpoint/method）
