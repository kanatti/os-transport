# Transport Actions

Complete catalog of OpenSearch transport actions (OS 3.7.0).

Action names follow the pattern: `<scope>:<category>/<operation>`

## Cluster Admin

```
cluster:admin/component_template/delete
cluster:admin/component_template/get
cluster:admin/component_template/put
cluster:admin/decommission/awareness/delete
cluster:admin/decommission/awareness/get
cluster:admin/decommission/awareness/put
cluster:admin/filecache/prune
cluster:admin/indices/dangling/delete
cluster:admin/indices/dangling/find
cluster:admin/indices/dangling/import
cluster:admin/indices/dangling/list
cluster:admin/ingest/pipeline/delete
cluster:admin/ingest/pipeline/get
cluster:admin/ingest/pipeline/put
cluster:admin/ingest/pipeline/simulate
cluster:admin/nodes/reload_secure_settings
cluster:admin/remote_store/metadata
cluster:admin/remotestore/restore
cluster:admin/repository/_cleanup
cluster:admin/repository/delete
cluster:admin/repository/get
cluster:admin/repository/put
cluster:admin/repository/verify
cluster:admin/reroute
cluster:admin/routing/awareness/weights/delete
cluster:admin/routing/awareness/weights/get
cluster:admin/routing/awareness/weights/put
cluster:admin/script/delete
cluster:admin/script/get
cluster:admin/script/put
cluster:admin/script_context/get
cluster:admin/script_language/get
cluster:admin/search/pipeline/delete
cluster:admin/search/pipeline/get
cluster:admin/search/pipeline/put
cluster:admin/settings/update
cluster:admin/snapshot/clone
cluster:admin/snapshot/create
cluster:admin/snapshot/delete
cluster:admin/snapshot/get
cluster:admin/snapshot/restore
cluster:admin/snapshot/status
cluster:admin/tasks/cancel
cluster:admin/views/create
cluster:admin/views/delete
cluster:admin/views/update
cluster:admin/voting_config/add_exclusions
cluster:admin/voting_config/clear_exclusions
```

## Cluster Monitor

```
cluster:monitor/allocation/explain
cluster:monitor/health
cluster:monitor/main
cluster:monitor/nodes/hot_threads
cluster:monitor/nodes/info
cluster:monitor/nodes/liveness
cluster:monitor/nodes/stats
cluster:monitor/nodes/usage
cluster:monitor/remote/info
cluster:monitor/_remotestore/stats
cluster:monitor/shards
cluster:monitor/state
cluster:monitor/stats
cluster:monitor/task
cluster:monitor/task/get
cluster:monitor/tasks/lists
cluster:monitor/wlm/stats
```

## Indices Admin

```
indices:admin/aliases
indices:admin/aliases/get
indices:admin/analyze
indices:admin/auto_create
indices:admin/block/add
indices:admin/cache/clear
indices:admin/close
indices:admin/create
indices:admin/data_stream/create
indices:admin/data_stream/delete
indices:admin/data_stream/get
indices:admin/delete
indices:admin/exists
indices:admin/flush
indices:admin/forcemerge
indices:admin/get
indices:admin/index_template/delete
indices:admin/index_template/get
indices:admin/index_template/put
indices:admin/index_template/simulate
indices:admin/index_template/simulate_index
indices:admin/ingestion/pause
indices:admin/ingestion/resume
indices:admin/ingestion/updateState
indices:admin/mapping/auto_put
indices:admin/mapping/put
indices:admin/mappings/fields/get
indices:admin/mappings/get
indices:admin/open
indices:admin/refresh
indices:admin/resize
indices:admin/resolve/index
indices:admin/rollover
indices:admin/scale/search_only
indices:admin/settings/update
indices:admin/shards/search_shards
indices:admin/template/delete
indices:admin/template/get
indices:admin/template/put
indices:admin/tier/hot_to_warm
indices:admin/upgrade
indices:admin/validate/query
```

## Indices Data Read

```
indices:data/read/explain
indices:data/read/field_caps
indices:data/read/get
indices:data/read/mget
indices:data/read/msearch
indices:data/read/mtv
indices:data/read/point_in_time/create
indices:data/read/point_in_time/delete
indices:data/read/point_in_time/readall
indices:data/read/scroll
indices:data/read/scroll/clear
indices:data/read/search
indices:data/read/search/stream
indices:data/read/tv
```

### Search Sub-Actions (shard-level)

These are dispatched internally during a search:

```
indices:data/read/search[can_match]
indices:data/read/search[phase/dfs]
indices:data/read/search[phase/query]
indices:data/read/search[phase/query/id]
indices:data/read/search[phase/query/scroll]
indices:data/read/search[phase/query+fetch/scroll]
indices:data/read/search[phase/fetch/id]
indices:data/read/search[phase/fetch/id/scroll]
indices:data/read/search[create_context]
indices:data/read/search[update_context]
indices:data/read/search[free_context]
indices:data/read/search[free_context/pit]
indices:data/read/search[free_context/scroll]
indices:data/read/search[free_pit_contexts]
indices:data/read/search[clear_scroll_contexts]
```

## Indices Data Write

```
indices:data/write/bulk
indices:data/write/delete
indices:data/write/index
indices:data/write/update
```

## Indices Monitor

```
indices:monitor/data_stream/stats
indices:monitor/ingestion/state
indices:monitor/point_in_time/segments
indices:monitor/recovery
indices:monitor/segment_replication
indices:monitor/segments
indices:monitor/settings/get
indices:monitor/shard_stores
indices:monitor/stats
indices:monitor/upgrade
```

## Internal: Coordination

```
internal:coordination/fault_detection/follower_check
internal:coordination/fault_detection/leader_check
internal:cluster/coordination/commit_state
internal:cluster/coordination/join
internal:cluster/coordination/join/validate
internal:cluster/coordination/publish_remote_state
internal:cluster/coordination/publish_state
internal:cluster/coordination/start_join
internal:cluster/request_pre_vote
```

## Internal: Shard Management

```
internal:cluster/shard/failure
internal:cluster/shard/started
internal:cluster/node/mapping/refresh
internal:cluster/nodes/indices/shard/store
internal:cluster/nodes/indices/shard/store/batch
internal:cluster/snapshot/update_snapshot_status
```

## Internal: Recovery

```
internal:index/shard/recovery/start_recovery
internal:index/shard/recovery/reestablish_recovery
internal:index/shard/recovery/filesInfo
internal:index/shard/recovery/file_chunk
internal:index/shard/recovery/clean_files
internal:index/shard/recovery/prepare_translog
internal:index/shard/recovery/translog_ops
internal:index/shard/recovery/finalize
internal:index/shard/recovery/handoff_primary_context
```

## Internal: Segment Replication

```
indices:admin/publishCheckpoint
indices:admin/publish_merged_segment
indices:admin/publish_referenced_segments
```

## Internal: Gateway

```
internal:gateway/local/allocate_dangled
internal:gateway/local/meta_state
internal:gateway/local/started_shards
internal:gateway/local/started_shards_batch
```

## Internal: Transport & Discovery

```
internal:transport/handshake
internal:tcp/handshake
internal:discovery/extensions
internal:discovery/request_peers
```

## Internal: Other

```
internal:admin/repository/verify
internal:admin/tasks/ban
internal:index/seq_no/resync
internal:indices/admin/upgrade
internal:indices/flush/synced/pre
internal:monitor/term
```

## Views

```
views:data/read/get
views:data/read/list
views:data/read/search
```

## Shard-Level Suffixes

Many actions have shard-level variants with `[s]` suffix (e.g. `indices:data/write/bulk[s]`).
These carry the same request type but targeted at a specific shard.
Node-level variants use `[n]` suffix (e.g. `cluster:monitor/nodes/stats[n]`).

## Action Name Structure

```
<scope>:<category>/<path>[<suffix>]

scope:     cluster | indices | internal | views
category:  admin | monitor | data/read | data/write | coordination
suffix:    [s] = shard-level, [n] = node-level, [phase/*] = search phase
```
