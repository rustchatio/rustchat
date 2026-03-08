- Endpoint or component: Calls plugin packaged as external plugin in Mattermost server distribution
- Source path: `../mattermost/server/Makefile`
- Source lines: `153-161`
- Observed behavior: `mattermost-plugin-calls-v1.11.0` is a prepackaged plugin artifact, indicating calls runtime behavior is primarily plugin-defined.
- Notes: call-specific server internals are not directly present in this repo tree.

- Endpoint or component: Server plugin routes are generic pass-through under `/plugins/{plugin_id}`
- Source path: `../mattermost/server/channels/app/channels.go`
- Source lines: `215-219`
- Observed behavior: Mattermost server registers generic plugin route handling and forwards requests to plugin handlers.
- Notes: compatibility for calls endpoints depends on matching plugin route semantics.

- Endpoint or component: Calls plugin enabled by default in server config
- Source path: `../mattermost/server/public/model/config.go`
- Source lines: `3456-3459`
- Observed behavior: `PluginIdCalls` default state is enabled.
- Notes: Rustchat should preserve enabled-by-default behavior for mobile startup assumptions.

- Endpoint or component: Internal plugin request context + headers
- Source path: `../mattermost/server/channels/app/plugin_requests.go`
- Source lines: `91-104`, `114-123`
- Observed behavior: plugin request handling is centralized and endpoint-specific behavior is plugin-owned.
- Notes: confirms why mobile contract parity should focus on calls plugin REST/WS interfaces.
