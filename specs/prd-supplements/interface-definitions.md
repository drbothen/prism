---
document_type: prd-supplement
level: L3
section: "interface-definitions"
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
---

# Interface Definitions

## 1. MCP Tool Schemas

### 1.1 Common Input Fields

Every MCP tool input includes these fields:

```json
{
  "client_id": {
    "type": ["string", "null"],
    "description": "MSSP client identifier. Non-null: query specific client. Null: cross-client query across all configured clients.",
    "pattern": "^[a-zA-Z0-9_-]+$"
  }
}
```

### 1.2 Sensor Query Tool — get_{sensor}_alerts

```json
{
  "name": "get_crowdstrike_alerts",
  "inputSchema": {
    "type": "object",
    "required": ["client_id"],
    "properties": {
      "client_id": { "type": ["string", "null"], "pattern": "^[a-zA-Z0-9_-]+$" },
      "severity": {
        "type": "string",
        "enum": ["low", "medium", "high", "critical"],
        "description": "Minimum severity filter. Valid values: low, medium, high, critical."
      },
      "status": {
        "type": "string",
        "enum": ["open", "in_progress", "closed", "resolved"],
        "description": "Alert status filter."
      },
      "time_range": {
        "type": "string",
        "description": "Time range filter. Examples: 'last_24h', 'last_7d', 'last_30d', or ISO8601 range 'start..end'."
      },
      "cursor": {
        "type": ["string", "null"],
        "description": "Pagination cursor from a previous response. Null for first page."
      },
      "page_size": {
        "type": "integer",
        "minimum": 1,
        "maximum": 100,
        "default": 25,
        "description": "Number of results per page (default: 25, max: 100)."
      }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": {
        "type": "object",
        "properties": {
          "tool": { "type": "string" },
          "data_source": { "type": "string" },
          "query_time": { "type": "string", "format": "date-time" },
          "trust_level": { "type": "string", "enum": ["untrusted_external", "internal"] },
          "safety_flags": { "type": "array", "items": { "type": "string" } },
          "total_results": { "type": "integer" },
          "page": { "type": "integer" },
          "has_more": { "type": "boolean" },
          "next_cursor": { "type": ["string", "null"] }
        }
      },
      "results": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "alert_id": { "type": "string" },
            "severity": { "type": "string" },
            "status": { "type": "string" },
            "title": { "type": "string" },
            "hostname": { "type": "string" },
            "hostname_safety_flag": { "type": ["string", "null"] },
            "detected_at": { "type": "string", "format": "date-time" },
            "ocsf": { "type": "object", "description": "OCSF v1.x normalized representation" },
            "raw_extensions": { "type": "object", "additionalProperties": true }
          }
        }
      }
    }
  },
  "annotations": {
    "readOnlyHint": true,
    "destructiveHint": false,
    "idempotentHint": true,
    "openWorldHint": true
  }
}
```

### 1.3 Health Check Tool — check_sensor_health

```json
{
  "name": "check_sensor_health",
  "inputSchema": {
    "type": "object",
    "required": ["client_id"],
    "properties": {
      "client_id": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$" },
      "sensor_id": {
        "type": ["string", "null"],
        "enum": ["crowdstrike", "cyberint", "claroty", "armis", null],
        "description": "Specific sensor to check, or null for all sensors."
      }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": {
        "type": "object",
        "properties": {
          "tool": { "type": "string" },
          "trust_level": { "const": "internal" }
        }
      },
      "summary": {
        "type": "object",
        "properties": {
          "healthy_count": { "type": "integer" },
          "unhealthy_count": { "type": "integer" },
          "total_count": { "type": "integer" }
        }
      },
      "sensors": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "sensor_id": { "type": "string" },
            "reachable": { "type": ["boolean", "null"] },
            "auth_valid": { "type": ["boolean", "null"] },
            "rate_limit": {
              "type": "object",
              "properties": {
                "is_rate_limited": { "type": "boolean" },
                "remaining_requests": { "type": ["integer", "null"] },
                "reset_at": { "type": ["string", "null"], "format": "date-time" },
                "retry_after_seconds": { "type": ["integer", "null"] }
              }
            },
            "last_successful_query_at": { "type": ["string", "null"], "format": "date-time" },
            "suggestion": { "type": ["string", "null"] }
          }
        }
      }
    }
  },
  "annotations": {
    "readOnlyHint": true,
    "destructiveHint": false,
    "idempotentHint": true,
    "openWorldHint": true
  }
}
```

### 1.4 Capabilities Meta-Tool — list_capabilities

```json
{
  "name": "list_capabilities",
  "inputSchema": {
    "type": "object",
    "required": ["client_id"],
    "properties": {
      "client_id": { "type": ["string", "null"], "pattern": "^[a-zA-Z0-9_-]+$" }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "clients": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "client_id": { "type": "string" },
            "capabilities": {
              "type": "array",
              "items": {
                "type": "object",
                "properties": {
                  "path": { "type": "string", "description": "Hierarchical capability path (e.g., sensor.crowdstrike.containment)" },
                  "status": { "type": "string", "enum": ["enabled", "runtime_disabled", "compile_time_disabled"] },
                  "resolution_chain": { "type": "array", "items": { "type": "string" } }
                }
              }
            }
          }
        }
      }
    }
  },
  "annotations": {
    "readOnlyHint": true,
    "destructiveHint": false,
    "idempotentHint": true,
    "openWorldHint": false
  }
}
```

### 1.5 Error Response Schema

```json
{
  "type": "object",
  "properties": {
    "error": {
      "type": "object",
      "required": ["code", "message", "category", "retryable", "suggestion"],
      "properties": {
        "code": { "type": "string", "pattern": "^E-[A-Z]+-\\d{3}$", "description": "Error code from the Prism error taxonomy" },
        "message": { "type": "string", "description": "Human-readable error description" },
        "category": { "type": "string", "enum": ["transient", "authentication", "validation", "not_found", "permission", "upstream_error", "configuration", "safety"] },
        "retryable": { "type": "boolean" },
        "retry_after_seconds": { "type": ["integer", "null"] },
        "suggestion": { "type": "string", "description": "Actionable guidance for the LLM agent" },
        "source": { "type": "string", "description": "Origin of the error" },
        "original_params_valid": { "type": "boolean", "description": "Whether the tool parameters were valid" },
        "upstream_status": { "type": ["integer", "null"] },
        "upstream_message": { "type": ["string", "null"], "description": "Error message from upstream sensor API (untrusted)" },
        "failed_parameter": { "type": ["string", "null"] },
        "failed_value": {},
        "allowed_values": { "type": ["array", "null"], "items": { "type": "string" } }
      }
    }
  }
}
```

---

## 2. TOML Configuration Schema

### 2.1 Top-Level Structure

```toml
# Global defaults
[defaults]
log_level = "info"                     # PRISM_LOG_LEVEL override
state_dir = "./state"                  # PRISM_STATE_DIR override
credential_backend = "keyring"         # "keyring" | "file"
credential_encryption_key_env = "PRISM_CREDENTIAL_KEY"  # env var name for file backend

[defaults.capabilities]
# Global capability defaults (deny-by-default)
# sensor.write = false                 # implicit default

# Per-client configuration
[clients.acme]
display_name = "Acme Corporation"

[clients.acme.sensors.crowdstrike]
enabled = true
api_base = "https://api.crowdstrike.com"
region = "us-1"                        # us-1 | us-2 | eu-1 | ap-1
credential_ref = "crowdstrike_oauth"
data_sources = ["alerts", "detections", "hosts"]

[clients.acme.sensors.claroty]
enabled = true
api_base = "https://acme.claroty.cloud"
credential_ref = "claroty_bearer"
data_sources = ["alerts", "devices", "vulnerabilities"]

[clients.acme.sensors.cyberint]
enabled = false                        # Disabled sensor

[clients.acme.sensors.armis]
enabled = true
api_base = "https://acme.armis.com"
credential_ref = "armis_api_key"
data_sources = ["alerts", "devices", "activities"]

[clients.acme.capabilities]
# Per-client capability overrides (more-specific wins)
sensor.crowdstrike.containment = true
sensor.claroty.write = false           # Explicit deny

[clients.globex]
display_name = "Globex Industries"

[clients.globex.sensors.crowdstrike]
enabled = true
api_base = "https://api.us-2.crowdstrike.com"
region = "us-2"
credential_ref = "crowdstrike_oauth"
data_sources = ["alerts"]
```

### 2.2 Required Fields Per Client

| TOML Path | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `clients.{id}` | table | yes | — | Client definition; `{id}` must match `[a-zA-Z0-9_-]+` |
| `clients.{id}.display_name` | string | yes | — | Human-readable client name |
| `clients.{id}.sensors.{sensor}` | table | no | — | Sensor config; `{sensor}` is one of: crowdstrike, cyberint, claroty, armis |
| `clients.{id}.sensors.{sensor}.enabled` | bool | no | `true` | Whether the sensor is active |
| `clients.{id}.sensors.{sensor}.api_base` | string (URL) | yes (if sensor present) | — | Sensor API base URL |
| `clients.{id}.sensors.{sensor}.credential_ref` | string | yes (if sensor present) | — | Reference to credential in store |
| `clients.{id}.sensors.{sensor}.data_sources` | array of string | no | all available | Data sources to enable for this sensor |
| `clients.{id}.sensors.{sensor}.region` | string | CrowdStrike only | `"us-1"` | CrowdStrike region |
| `clients.{id}.capabilities` | table | no | inherits defaults | Capability overrides |

---

## 3. CLI Flags

```
prism [OPTIONS]

OPTIONS:
  -c, --config <PATH>           Path to TOML configuration file [default: ./prism.toml]
                                 Env: PRISM_CONFIG
  --dry-run                     Validate configuration, print redacted config, and exit
  --log-level <LEVEL>           Log level: trace, debug, info, warn, error
                                 [default: info] Env: PRISM_LOG_LEVEL
  --state-dir <PATH>            Directory for cursor state files
                                 [default: ./state] Env: PRISM_STATE_DIR
  --credential-backend <TYPE>   Credential backend: keyring, file
                                 [default: keyring] Env: PRISM_CREDENTIAL_BACKEND
  -V, --version                 Print version information and exit
  -h, --help                    Print help information
```

---

## 4. Exit Codes

| Code | Meaning | Example |
|------|---------|---------|
| 0 | Clean exit | Graceful shutdown on SIGTERM, `--dry-run` with valid config, `--version`, `--help` |
| 1 | Configuration error | Invalid TOML, missing required fields, `--dry-run` validation failure |
| 2 | Credential error | Keyring unavailable at startup, encryption key missing |
| 3 | State error | OCSF descriptor load failure, fingerprint mismatch (fatal) |
| 4 | Runtime error | Unexpected panic, unrecoverable I/O error |
| 130 | SIGINT (Ctrl-C) | User-initiated interrupt with graceful shutdown |
| 143 | SIGTERM | Process manager-initiated termination with graceful shutdown |
