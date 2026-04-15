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
      },
      "force_refresh": {
        "type": "boolean",
        "default": false,
        "description": "If true, bypass the response cache and fetch fresh data from the sensor API. Default: false (use cache if available)."
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

**Cross-Client Response Shape:** When `client_id` is `null` (cross-client query), the response structure changes to wrap results per-client with per-client cursors. The `_meta` object includes `is_cross_client: true` to signal the structural difference:

```json
{
  "_meta": {
    "tool": "get_crowdstrike_alerts",
    "is_cross_client": true,
    "query_time": "2026-04-13T12:00:00Z",
    "trust_level": "untrusted_external",
    "safety_flags": [],
    "total_results": 42,
    "clients_queried": ["acme", "globex", "initech"],
    "clients_skipped": [],
    "partial_failures": [],
    "cursor_cap_reached": false,
    "clients_without_cursor": []
  },
  "client_results": [
    {
      "client_id": "acme",
      "result_count": 25,
      "has_more": true,
      "next_cursor": "eyJjbGllbnQiOiJhY21lIi4uLn0=",
      "results": [ ]
    },
    {
      "client_id": "globex",
      "result_count": 17,
      "has_more": false,
      "next_cursor": null,
      "results": [ ]
    }
  ]
}
```

When `client_id` is non-null (single-client query), the response uses the flat structure shown above with `is_cross_client: false` (or absent). Agents should check `is_cross_client` to determine which response shape to parse.

### 1.3 Health Check Tool — check_sensor_health

```json
{
  "name": "check_sensor_health",
  "inputSchema": {
    "type": "object",
    "required": ["client_id"],
    "properties": {
      "client_id": { "type": ["string", "null"], "pattern": "^[a-zA-Z0-9_-]+$", "description": "Client ID, or null for cross-client health overview." },
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

### 1.6 Credential CRUD Tools

#### set_credential

```json
{
  "name": "set_credential",
  "inputSchema": {
    "type": "object",
    "required": ["client_id", "sensor_id", "credential_name", "credential_value"],
    "properties": {
      "client_id": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$", "description": "Client that owns the credential." },
      "sensor_id": { "type": "string", "enum": ["crowdstrike", "cyberint", "claroty", "armis"], "description": "Sensor the credential is for." },
      "credential_name": { "type": "string", "pattern": "^[a-zA-Z0-9_.\\-]+$", "description": "Credential key name (e.g., 'client_secret', 'api_key')." },
      "credential_value": { "type": "string", "description": "The credential value to store. Never echoed in responses." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["created", "confirmation_required"] },
      "confirmation_token": { "type": ["object", "null"], "description": "Present when updating an existing credential (confirmation required per BC-2.03.005)." }
    }
  },
  "annotations": {
    "readOnlyHint": false,
    "destructiveHint": false,
    "idempotentHint": false,
    "openWorldHint": false
  }
}
```

#### delete_credential

```json
{
  "name": "delete_credential",
  "inputSchema": {
    "type": "object",
    "required": ["client_id", "sensor_id", "credential_name"],
    "properties": {
      "client_id": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$" },
      "sensor_id": { "type": "string", "enum": ["crowdstrike", "cyberint", "claroty", "armis"] },
      "credential_name": { "type": "string", "pattern": "^[a-zA-Z0-9_.\\-]+$" }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["confirmation_required"] },
      "confirmation_token": { "type": "object", "description": "Confirmation token; call confirm_action to execute deletion (per BC-2.03.005)." }
    }
  },
  "annotations": {
    "readOnlyHint": false,
    "destructiveHint": true,
    "idempotentHint": true,
    "openWorldHint": false
  }
}
```

#### list_credentials

```json
{
  "name": "list_credentials",
  "inputSchema": {
    "type": "object",
    "required": ["client_id"],
    "properties": {
      "client_id": { "type": ["string", "null"], "pattern": "^[a-zA-Z0-9_-]+$", "description": "Client ID, or null for all clients." },
      "sensor_id": { "type": ["string", "null"], "enum": ["crowdstrike", "cyberint", "claroty", "armis", null], "description": "Filter by sensor, or null for all." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "credentials": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "client_id": { "type": "string" },
            "sensor_id": { "type": "string" },
            "credential_name": { "type": "string" },
            "backend": { "type": "string", "enum": ["keyring", "encrypted_file"] },
            "last_modified": { "type": ["string", "null"], "format": "date-time" }
          }
        },
        "description": "Metadata only; credential values are never returned."
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

### 1.7 Write Operation Tool — crowdstrike_contain_host (Representative)

```json
{
  "name": "crowdstrike_contain_host",
  "inputSchema": {
    "type": "object",
    "required": ["client_id", "host_id"],
    "properties": {
      "client_id": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$", "description": "Client ID. Must be non-null for write operations." },
      "host_id": { "type": "string", "description": "CrowdStrike host/device ID to contain (network isolate)." },
      "reason": { "type": "string", "description": "Human-readable justification for containment. Included in audit log and confirmation prompt." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": {
        "type": "object",
        "properties": {
          "tool": { "type": "string" },
          "trust_level": { "const": "internal" },
          "safety_flags": { "type": "array", "items": { "type": "string" } }
        }
      },
      "status": { "type": "string", "enum": ["confirmation_required", "executed", "failed"] },
      "confirmation_token": {
        "type": ["object", "null"],
        "properties": {
          "token_id": { "type": "string" },
          "action_summary": { "type": "string" },
          "expires_at": { "type": "string", "format": "date-time" }
        },
        "description": "Present on first call (confirmation_required). Call confirm_action(token_id) to execute."
      },
      "result": {
        "type": ["object", "null"],
        "properties": {
          "host_id": { "type": "string" },
          "contained": { "type": "boolean" },
          "sensor_response": { "type": "object", "description": "Raw CrowdStrike API response (in structuredContent, not prose)." }
        },
        "description": "Present after successful execution via confirm_action."
      }
    }
  },
  "annotations": {
    "readOnlyHint": false,
    "destructiveHint": true,
    "idempotentHint": false,
    "openWorldHint": true
  }
}
```

### 1.8 Confirmation Tool — confirm_action

```json
{
  "name": "confirm_action",
  "inputSchema": {
    "type": "object",
    "required": ["client_id", "token_id"],
    "properties": {
      "client_id": {
        "type": "string",
        "pattern": "^[a-zA-Z0-9_-]+$",
        "description": "Client ID. Must match the client_id embedded in the confirmation token. Prevents cross-client token replay attacks."
      },
      "token_id": {
        "type": "string",
        "description": "The confirmation token ID returned by a write operation tool (e.g., crowdstrike_contain_host, set_credential, delete_credential)."
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
      "status": {
        "type": "string",
        "enum": ["executed", "failed"],
        "description": "Whether the confirmed action was successfully executed."
      },
      "result": {
        "type": ["object", "null"],
        "description": "Action-specific result payload. Present when status is 'executed'. Structure depends on the original write tool (e.g., containment result, credential update confirmation)."
      }
    }
  },
  "annotations": {
    "readOnlyHint": false,
    "destructiveHint": true,
    "idempotentHint": false,
    "openWorldHint": true
  }
}
```

### 1.9 Query Tool — query

```json
{
  "name": "query",
  "inputSchema": {
    "type": "object",
    "required": ["query"],
    "properties": {
      "query": {
        "type": "string",
        "description": "AxiQL query string. Auto-detects mode: filter (boolean expressions), SQL (SELECT/FROM), or pipe (stages separated by |)."
      },
      "clients": {
        "type": ["array", "null"],
        "items": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$" },
        "default": null,
        "description": "Client IDs to query. Null means all configured clients."
      },
      "sensors": {
        "type": ["array", "null"],
        "items": { "type": "string", "enum": ["crowdstrike", "cyberint", "claroty", "armis"] },
        "default": null,
        "description": "Sensor types to query. Null means all enabled sensors."
      },
      "sources": {
        "type": ["array", "null"],
        "items": { "type": "string" },
        "default": null,
        "description": "Data source names to query (e.g., 'alerts', 'devices'). Null means all available sources."
      },
      "limit": {
        "type": "integer",
        "minimum": 1,
        "maximum": 1000,
        "default": 25,
        "description": "Maximum number of results to return. No cross-call pagination; increase limit or narrow query to see more."
      },
      "force_refresh": {
        "type": "boolean",
        "default": false,
        "description": "If true, bypass the response cache and fetch fresh data from sensor APIs."
      }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "query_context": {
        "type": "object",
        "properties": {
          "original_query": { "type": "string", "description": "The raw query string as provided." },
          "expanded_query": { "type": "string", "description": "The query after alias expansion." },
          "clients_queried": { "type": "array", "items": { "type": "string" } },
          "sensors_queried": { "type": "array", "items": { "type": "string" } },
          "is_truncated": { "type": "boolean", "description": "True if total_available > returned results (limit applied)." },
          "total_available": { "type": "integer", "description": "Total matching records before limit truncation." }
        }
      },
      "events": {
        "type": "array",
        "items": {
          "type": "object",
          "description": "OCSF-normalized event records with virtual fields (sensor, client_id, source)."
        }
      },
      "_meta": {
        "type": "object",
        "properties": {
          "safety_flags": { "type": "array", "items": { "type": "string" } },
          "trust_level": { "type": "string", "enum": ["untrusted_external"] }
        }
      },
      "sensor_errors": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "client_id": { "type": "string" },
            "sensor": { "type": "string" },
            "error_code": { "type": "string" },
            "message": { "type": "string" }
          }
        },
        "description": "Errors from individual sensor API calls. Partial results are valid when some sensors succeed."
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

### 1.10 Explain Query Tool — explain_query

```json
{
  "name": "explain_query",
  "inputSchema": {
    "type": "object",
    "required": ["query"],
    "properties": {
      "query": {
        "type": "string",
        "description": "AxiQL query string to explain (parsed and planned but not executed)."
      },
      "clients": {
        "type": ["array", "null"],
        "items": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$" },
        "default": null,
        "description": "Client IDs to scope the explain. Null means all configured clients."
      },
      "sensors": {
        "type": ["array", "null"],
        "items": { "type": "string", "enum": ["crowdstrike", "cyberint", "claroty", "armis"] },
        "default": null,
        "description": "Sensor types to scope the explain. Null means all enabled sensors."
      },
      "sources": {
        "type": ["array", "null"],
        "items": { "type": "string" },
        "default": null,
        "description": "Data source names to scope the explain. Null means all available sources."
      }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "alias_expansion": {
        "type": "object",
        "additionalProperties": { "type": "string" },
        "description": "Map of alias names to their expanded definitions (if any aliases were used)."
      },
      "field_resolution": {
        "type": "object",
        "additionalProperties": {
          "type": "object",
          "properties": {
            "ocsf_path": { "type": "string" },
            "resolution_method": { "type": "string", "enum": ["direct", "alias", "virtual"] }
          }
        },
        "description": "Map of field names used in the query to their OCSF paths and resolution method."
      },
      "push_down_plan": {
        "type": "object",
        "additionalProperties": {
          "type": "object",
          "properties": {
            "pushed_filters": { "type": "object", "description": "Sensor-native translated push-down filters." },
            "post_filters": { "type": "array", "items": { "type": "string" }, "description": "Predicates applied after fetch." }
          }
        },
        "description": "Per-sensor push-down plan showing which filters are pushed to the sensor API vs. applied post-fetch."
      },
      "estimated_cost": {
        "type": "object",
        "properties": {
          "record_estimate": { "type": ["integer", "null"], "description": "Estimated record count (null if cannot be estimated)." },
          "sensors_to_query": { "type": "array", "items": { "type": "string" } },
          "estimated_api_calls": { "type": "integer", "description": "Estimated number of sensor API calls." }
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

### 1.11 Create Alias Tool — create_alias

```json
{
  "name": "create_alias",
  "inputSchema": {
    "type": "object",
    "required": ["name", "scope", "query"],
    "properties": {
      "name": {
        "type": "string",
        "pattern": "^[a-zA-Z_][a-zA-Z0-9_]*$",
        "description": "Alias identifier. Must not conflict with AxiQL keywords."
      },
      "scope": {
        "type": "string",
        "pattern": "^(global|client:[a-zA-Z0-9_-]+)$",
        "description": "Alias scope: 'global' or 'client:<client_id>'."
      },
      "query": {
        "type": "string",
        "description": "AxiQL expression or template string for the alias."
      },
      "parameters": {
        "type": ["object", "null"],
        "additionalProperties": { "type": "string" },
        "default": null,
        "description": "Map of parameter names to default values (if parameterized). All parameters must have defaults."
      },
      "description": {
        "type": ["string", "null"],
        "default": null,
        "description": "Human-readable description of the alias."
      }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["created", "confirmation_required"] },
      "alias": {
        "type": "object",
        "properties": {
          "name": { "type": "string" },
          "scope": { "type": "string" },
          "query": { "type": "string" },
          "expanded": { "type": "string" },
          "parameters": { "type": ["object", "null"] }
        }
      },
      "confirmation_token": { "type": ["object", "null"], "description": "Present when updating an existing alias (confirmation required)." }
    }
  },
  "annotations": {
    "readOnlyHint": false,
    "destructiveHint": false,
    "idempotentHint": false,
    "openWorldHint": false
  }
}
```

### 1.12 List Aliases Tool — list_aliases

```json
{
  "name": "list_aliases",
  "inputSchema": {
    "type": "object",
    "properties": {
      "scope": {
        "type": ["string", "null"],
        "pattern": "^(global|client:[a-zA-Z0-9_-]+)$",
        "default": null,
        "description": "Filter by scope: 'global', 'client:<client_id>', or null for all aliases."
      }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "aliases": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "name": { "type": "string" },
            "scope": { "type": "string" },
            "query": { "type": "string" },
            "parameters": { "type": ["object", "null"] },
            "description": { "type": ["string", "null"] }
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

### 1.13 Delete Alias Tool — delete_alias

```json
{
  "name": "delete_alias",
  "inputSchema": {
    "type": "object",
    "required": ["name", "scope"],
    "properties": {
      "name": {
        "type": "string",
        "pattern": "^[a-zA-Z_][a-zA-Z0-9_]*$",
        "description": "Alias name to delete."
      },
      "scope": {
        "type": "string",
        "pattern": "^(global|client:[a-zA-Z0-9_-]+)$",
        "description": "Scope of the alias to delete."
      }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["confirmation_required"] },
      "confirmation_token": { "type": "object", "description": "Confirmation token; call confirm_action to execute deletion." },
      "dependent_aliases": {
        "type": "array",
        "items": { "type": "string" },
        "description": "List of aliases that reference this alias (warning to user before confirming)."
      }
    }
  },
  "annotations": {
    "readOnlyHint": false,
    "destructiveHint": true,
    "idempotentHint": true,
    "openWorldHint": false
  }
}
```

### 1.14 Explain Alias Tool — explain_alias

```json
{
  "name": "explain_alias",
  "inputSchema": {
    "type": "object",
    "required": ["name"],
    "properties": {
      "name": {
        "type": "string",
        "pattern": "^[a-zA-Z_][a-zA-Z0-9_]*$",
        "description": "Alias name to explain."
      },
      "scope": {
        "type": ["string", "null"],
        "pattern": "^(global|client:[a-zA-Z0-9_-]+)$",
        "default": null,
        "description": "Scope to resolve the alias in. Null resolves using default scope precedence."
      }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "alias": {
        "type": "object",
        "properties": {
          "name": { "type": "string" },
          "scope": { "type": "string" },
          "query": { "type": "string" },
          "expanded": { "type": "string", "description": "Fully expanded query after recursive alias resolution." },
          "parameters": { "type": ["object", "null"] },
          "description": { "type": ["string", "null"] },
          "composition_chain": { "type": "array", "items": { "type": "string" }, "description": "Chain of aliases expanded during resolution." },
          "composition_depth": { "type": "integer" }
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
# credential.write = false             # implicit default; global per-client (not per-sensor)

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
# TOML boolean mapping: `true` maps to `Allow` (explicit allow entry in BTreeMap),
# `false` maps to `Deny` (explicit deny entry in BTreeMap).
# Key absent means no entry — resolved via hierarchy walk to implicit deny.
sensor.crowdstrike.containment = true
sensor.claroty.write = false           # Explicit deny
credential.write = true                # Global per-client: allow credential mutations (set/delete) for this client. Not per-sensor — credential write permission applies across all sensors for the client.

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
  --state-dir <PATH>            Directory for credential files and cache data
                                 [default: ./state] Env: PRISM_STATE_DIR
  --credential-backend <TYPE>   Credential backend: keyring, file
                                 [default: keyring] Env: PRISM_CREDENTIAL_BACKEND
  Note: analyst_id is configured via TOML config, env var, or OS detection:
                                 1. TOML config field: `analyst_id = "jsmith"` (highest priority)
                                 2. PRISM_ANALYST_ID environment variable
                                 3. OS username detection (lowest priority)
                                 The resolved value populates user_identity in all
                                 AuditEntry records. Not a CLI flag -- MCP servers are
                                 launched by the host (Claude Code), not interactively.
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
| 3 | State error | OCSF descriptor load failure, configuration validation failure |
| 4 | Runtime error | Unexpected panic, unrecoverable I/O error |
| 130 | SIGINT (Ctrl-C) | User-initiated interrupt with graceful shutdown |
| 143 | SIGTERM | Process manager-initiated termination with graceful shutdown |
