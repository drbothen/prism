---
document_type: prd-supplement
level: L3
section: "interface-definitions"
version: "2.2"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
---

# Interface Definitions

## 1. MCP Tool Schemas

### 1.1 Common Input Fields

Client scoping follows two patterns depending on the tool type:

**Read tools** (`query`, `explain_query`) use a `clients` array:

```json
{
  "clients": {
    "type": ["array", "null"],
    "items": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$" },
    "default": null,
    "description": "Client IDs to query. Null means all configured clients."
  }
}
```

**Write tools and management tools** use a scalar `client_id`:

```json
{
  "client_id": {
    "type": "string",
    "description": "MSSP client identifier. Required and non-null for write operations.",
    "pattern": "^[a-zA-Z0-9_-]+$"
  }
}
```

### 1.2 REMOVED -- Per-Sensor Query Tools

Per-sensor read tools (`get_crowdstrike_alerts`, `get_claroty_devices`, etc.) have been removed. All data access is now through the `query` tool (section 1.9). See BC-2.11.001.

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
        "pattern": "^[a-z][a-z0-9_-]*$", "description": "Sensor identifier matching a loaded spec file's sensor_id. Null for all.",
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

#### configure_credential_source

**Capability gate:** `credential.write` — Reversible

Configures where Prism resolves a named credential for a given `(client_id, sensor_id)` pair. Accepts a source type reference only; raw credential values NEVER transit the AI context (AI-opaque credentials model). Replaces the former `set_credential` tool.

```json
{
  "name": "configure_credential_source",
  "inputSchema": {
    "type": "object",
    "required": ["client_id", "sensor_id", "name", "source"],
    "properties": {
      "client_id": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$", "description": "Client that owns the credential." },
      "sensor_id": { "type": "string", "pattern": "^[a-z][a-z0-9_-]*$", "description": "Sensor identifier matching a loaded spec file's sensor_id (e.g., crowdstrike, cyberint, claroty, armis, or any config-driven sensor)." },
      "name": { "type": "string", "pattern": "^[a-zA-Z0-9_.\\-]+$", "description": "Credential key name (e.g., 'client_secret', 'api_key')." },
      "source": {
        "type": "object",
        "required": ["type"],
        "description": "Source reference for the credential. Raw values are NEVER accepted here — only reference pointers.",
        "properties": {
          "type": { "type": "string", "enum": ["env", "file", "vault", "keyring"], "description": "Source backend type." },
          "env_var": { "type": "string", "description": "Environment variable name. Required when type='env'." },
          "file_path": { "type": "string", "description": "Absolute path to the credential file. Required when type='file'." },
          "vault_path": { "type": "string", "description": "Vault secret path (e.g., 'secret/data/prism/crowdstrike'). Required when type='vault'." },
          "keyring_service": { "type": ["string", "null"], "description": "Keyring service name. Optional when type='keyring'; defaults to 'prism'." }
        }
      },
      "dry_run": { "type": "boolean", "default": false, "description": "If true, validate the source reference without persisting it." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["configured", "confirmation_required", "dry_run_ok", "dry_run_failed"] },
      "confirmation_token": { "type": ["object", "null"], "properties": { "token_id": { "type": "string" }, "action_summary": { "type": "string" }, "expires_at": { "type": "string", "format": "date-time" } }, "description": "Present when updating an existing credential source reference (confirmation required per BC-2.03.005)." }
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
      "sensor_id": { "type": "string", "pattern": "^[a-z][a-z0-9_-]*$", "description": "Sensor identifier matching a loaded spec file's sensor_id (e.g., crowdstrike, cyberint, claroty, armis, or any config-driven sensor)" },
      "credential_name": { "type": "string", "pattern": "^[a-zA-Z0-9_.\\-]+$" }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["confirmation_required"] },
      "confirmation_token": { "type": "object", "properties": { "token_id": { "type": "string" }, "action_summary": { "type": "string" }, "expires_at": { "type": "string", "format": "date-time" } }, "description": "Confirmation token; call confirm_action to execute deletion (per BC-2.03.005)." }
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
      "client_id": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$", "description": "Client ID. Required and non-null — cross-client credential listing is not supported to prevent MSSP client portfolio disclosure." },
      "sensor_id": { "type": ["string", "null"], "pattern": "^[a-z][a-z0-9_-]*$", "description": "Sensor identifier matching a loaded spec file's sensor_id. Null for all." }
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
        "description": "Client ID. Must match the client_id embedded in the confirmation token. Prevents cross-client token replay attacks. Note: client_id is validated against the token's embedded client_id, not against client config. The sentinel '__global__' is valid for global-scope operations (aliases, schedules, packs, global rules)."
      },
      "token_id": {
        "type": "string",
        "description": "The confirmation token ID returned by a write operation tool (e.g., crowdstrike_contain_host, configure_credential_source, delete_credential)."
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
        "description": "PrismQL query string. Auto-detects mode: filter (boolean expressions), SQL (SELECT/FROM), or pipe (stages separated by |)."
      },
      "clients": {
        "type": ["array", "null"],
        "items": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$" },
        "default": null,
        "description": "Client IDs to query. Null means all configured clients."
      },
      "sensors": {
        "type": ["array", "null"],
        "items": { "type": "string", "pattern": "^[a-z][a-z0-9_-]*$", "description": "Sensor identifier matching a loaded spec file's sensor_id (e.g., crowdstrike, cyberint, claroty, armis, or any config-driven sensor)" },
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
        "description": "PrismQL query string to explain (parsed and planned but not executed)."
      },
      "clients": {
        "type": ["array", "null"],
        "items": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$" },
        "default": null,
        "description": "Client IDs to scope the explain. Null means all configured clients."
      },
      "sensors": {
        "type": ["array", "null"],
        "items": { "type": "string", "pattern": "^[a-z][a-z0-9_-]*$", "description": "Sensor identifier matching a loaded spec file's sensor_id (e.g., crowdstrike, cyberint, claroty, armis, or any config-driven sensor)" },
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
        "description": "Alias identifier. Must not conflict with PrismQL keywords."
      },
      "scope": {
        "type": "string",
        "pattern": "^(global|client:[a-zA-Z0-9_-]+)$",
        "description": "Alias scope: 'global' or 'client:<client_id>'."
      },
      "query": {
        "type": "string",
        "description": "PrismQL expression or template string for the alias."
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
      "confirmation_token": { "type": ["object", "null"], "properties": { "token_id": { "type": "string" }, "action_summary": { "type": "string" }, "expires_at": { "type": "string", "format": "date-time" } }, "description": "Present when updating an existing alias (confirmation required)." }
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
      },
      "force": {
        "type": "boolean",
        "default": false,
        "description": "If true, cascade-delete all dependent aliases that reference this alias. If false (default), deletion is blocked when dependents exist (E-ALIAS-005)."
      }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["confirmation_required"] },
      "confirmation_token": { "type": "object", "properties": { "token_id": { "type": "string" }, "action_summary": { "type": "string" }, "expires_at": { "type": "string", "format": "date-time" } }, "description": "Confirmation token; call confirm_action to execute deletion." },
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

### 1.15 Create Schedule Tool — create_schedule (Subsystem 12: Scheduled Queries)

```json
{
  "name": "create_schedule",
  "inputSchema": {
    "type": "object",
    "required": ["name", "query", "interval"],
    "properties": {
      "name": { "type": "string", "description": "Human-readable schedule name. Must be unique." },
      "query": { "type": "string", "description": "PrismQL query string to execute on each interval." },
      "interval": { "type": "string", "pattern": "^\\d+(s|m|h|d)$", "description": "Execution interval (e.g., '5m', '1h'). Minimum 60s." },
      "clients": {
        "type": ["array", "null"],
        "items": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$" },
        "default": null,
        "description": "Client IDs to scope the scheduled query. Null means all configured clients."
      },
      "sensors": {
        "type": ["array", "null"],
        "items": { "type": "string", "pattern": "^[a-z][a-z0-9_-]*$", "description": "Sensor identifier matching a loaded spec file's sensor_id (e.g., crowdstrike, cyberint, claroty, armis, or any config-driven sensor)" },
        "default": null,
        "description": "Sensor types to scope the scheduled query. Null means all enabled sensors."
      },
      "splay_percent": { "type": "integer", "minimum": 0, "maximum": 25, "default": 10, "description": "Percentage of interval to randomize execution start time to avoid thundering herd." },
      "snapshot_mode": { "type": "boolean", "default": false, "description": "If true, store full results on every run (not just differential). Useful for audit baselines." },
      "track_removed": { "type": "boolean", "default": true, "description": "If true, include records that disappeared between runs in the differential output." },
      "enabled": { "type": "boolean", "default": true, "description": "Whether the schedule is active immediately after creation." },
      "dry_run": { "type": "boolean", "default": true, "description": "If true, validate and preview the schedule without creating it. Default: true per BC-2.04.008." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "schedule_id": { "type": "string", "description": "Unique identifier for the created schedule." },
      "next_run": { "type": "string", "format": "date-time", "description": "Timestamp of the next scheduled execution." },
      "splay_offset": { "type": "string", "description": "Computed splay offset applied to this schedule (e.g., '12s')." }
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

### 1.16 List Schedules Tool — list_schedules (Subsystem 12: Scheduled Queries)

```json
{
  "name": "list_schedules",
  "inputSchema": {
    "type": "object",
    "properties": {
      "limit": { "type": "integer", "default": 100, "minimum": 1, "maximum": 1000, "description": "Maximum number of results to return." },
      "offset": { "type": "integer", "default": 0, "minimum": 0, "description": "Number of results to skip for pagination." },
      "client_id": { "type": ["string", "null"], "pattern": "^[a-zA-Z0-9_-]+$", "default": null, "description": "Filter schedules to those scoped to a specific client. Null returns all." },
      "enabled_only": { "type": "boolean", "default": false, "description": "If true, return only enabled schedules." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "total_count": { "type": "integer", "description": "Total number of schedules (before limit/offset)." },
      "schedules": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "schedule_id": { "type": "string" },
            "name": { "type": "string" },
            "query": { "type": "string" },
            "interval": { "type": "string" },
            "enabled": { "type": "boolean" },
            "splay_percent": { "type": "integer" },
            "snapshot_mode": { "type": "boolean" },
            "track_removed": { "type": "boolean" },
            "clients": {
              "type": "array",
              "items": {
                "type": "object",
                "properties": {
                  "client_id": { "type": "string" },
                  "last_run": { "type": ["string", "null"], "format": "date-time" },
                  "next_run": { "type": ["string", "null"], "format": "date-time" },
                  "epoch": { "type": "integer", "description": "Schedule epoch counter for this client." },
                  "counter": { "type": "integer", "description": "Number of completed executions for this client." }
                }
              },
              "description": "Per-client execution state for this schedule."
            },
            "created_at": { "type": "string", "format": "date-time" }
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

### 1.17 Delete Schedule Tool — delete_schedule (Subsystem 12: Scheduled Queries)

```json
{
  "name": "delete_schedule",
  "inputSchema": {
    "type": "object",
    "required": ["schedule_id"],
    "properties": {
      "schedule_id": { "type": "string", "description": "ID of the schedule to delete." },
      "client_id": { "type": ["string", "null"], "pattern": "^[a-zA-Z0-9_-]+$", "default": null, "description": "Client scope for confirmation token. Null uses '__global__' sentinel for global schedules." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["confirmation_required"] },
      "confirmation_token": { "type": "object", "properties": { "token_id": { "type": "string" }, "action_summary": { "type": "string" }, "expires_at": { "type": "string", "format": "date-time" } }, "description": "Confirmation token; call confirm_action to execute deletion." }
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

### 1.18 Get Diff Results Tool — get_diff_results (Subsystem 12: Scheduled Queries)

```json
{
  "name": "get_diff_results",
  "inputSchema": {
    "type": "object",
    "required": ["schedule_id"],
    "properties": {
      "schedule_id": { "type": "string", "description": "ID of the schedule to retrieve differential results for." },
      "client_id": {
        "type": ["string", "null"],
        "pattern": "^[a-zA-Z0-9_-]+$",
        "default": null,
        "description": "Filter diff results to a specific client. Null returns results for all clients in the schedule."
      }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "added": { "type": "array", "items": { "type": "object" }, "description": "Records present in the latest run but absent in the previous run." },
      "removed": { "type": "array", "items": { "type": "object" }, "description": "Records present in the previous run but absent in the latest run." },
      "epoch": { "type": "integer", "description": "Monotonic epoch counter for the schedule's result set." },
      "counter": { "type": "integer", "description": "Number of differential computations performed for this schedule." }
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

### 1.19 List Packs / Explain Pack Tools — list_packs, explain_pack (Subsystem 12: Scheduled Queries)

```json
{
  "name": "list_packs",
  "inputSchema": {
    "type": "object",
    "properties": {
      "pack_id": {
        "type": ["string", "null"],
        "default": null,
        "description": "Specific pack ID to retrieve. Null returns all packs."
      }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "packs": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "pack_id": { "type": "string" },
            "name": { "type": "string" },
            "description": { "type": ["string", "null"] },
            "queries": {
              "type": "array",
              "items": {
                "type": "object",
                "properties": {
                  "schedule_id": { "type": "string" },
                  "name": { "type": "string" },
                  "interval": { "type": "string" }
                }
              }
            },
            "discovery_status": { "type": "string", "enum": ["active", "disabled", "partial"], "description": "Whether the pack's queries are actively running." }
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

```json
{
  "name": "explain_pack",
  "inputSchema": {
    "type": "object",
    "required": ["pack_id"],
    "properties": {
      "pack_id": { "type": "string", "description": "Pack ID to explain." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "pack": {
        "type": "object",
        "properties": {
          "pack_id": { "type": "string" },
          "name": { "type": "string" },
          "description": { "type": ["string", "null"] },
          "queries": {
            "type": "array",
            "items": {
              "type": "object",
              "properties": {
                "schedule_id": { "type": "string" },
                "name": { "type": "string" },
                "query": { "type": "string" },
                "interval": { "type": "string" },
                "enabled": { "type": "boolean" }
              }
            }
          },
          "discovery_status": { "type": "string" }
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

### 1.19b Create Pack Tool — create_pack (Subsystem 12: Scheduled Queries)

```json
{
  "name": "create_pack",
  "inputSchema": {
    "type": "object",
    "required": ["name", "client_id"],
    "properties": {
      "name": { "type": "string", "description": "Human-readable pack name. Must be unique." },
      "client_id": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$", "description": "Client ID for capability gating. Required for DI-008 client data separation." },
      "description": { "type": ["string", "null"], "default": null, "description": "Optional pack description." },
      "query_refs": {
        "type": "array",
        "items": { "type": "string" },
        "default": [],
        "description": "Array of schedule name strings referencing existing scheduled queries to include in the pack."
      },
      "detection_refs": {
        "type": "array",
        "items": { "type": "string" },
        "default": [],
        "description": "Array of rule_id strings referencing existing detection rules to include in the pack."
      },
      "discovery_query": { "type": ["string", "null"], "default": null, "description": "PrismQL query that must return >= 1 row for the pack to be active for a client (DEC-034)." },
      "enabled": { "type": "boolean", "default": true, "description": "Whether the pack is enabled on creation." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "pack_id": { "type": "string", "description": "Assigned pack ID." },
      "status": { "type": "string", "enum": ["created"], "description": "Creation status." }
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

### 1.19c Delete Pack Tool — delete_pack (Subsystem 12: Scheduled Queries)

```json
{
  "name": "delete_pack",
  "inputSchema": {
    "type": "object",
    "required": ["pack_id"],
    "properties": {
      "pack_id": { "type": "string", "description": "ID of the pack to delete." },
      "client_id": { "type": ["string", "null"], "pattern": "^[a-zA-Z0-9_-]+$", "default": null, "description": "Client scope for confirmation token. Null uses '__global__' sentinel for global packs." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["confirmation_required"], "description": "Deletion requires confirmation via confirm_action." },
      "confirmation_token": {
        "type": "object",
        "properties": {
          "token_id": { "type": "string" },
          "action_summary": { "type": "string" },
          "expires_at": { "type": "string", "format": "date-time" }
        },
        "description": "Confirmation token for the delete operation. Pass to confirm_action to execute."
      }
    }
  },
  "annotations": {
    "readOnlyHint": false,
    "destructiveHint": true,
    "idempotentHint": false,
    "openWorldHint": false
  }
}
```

### 1.20 Create Rule Tool — create_rule (Subsystem 13: Detection Engine)

```json
{
  "name": "create_rule",
  "inputSchema": {
    "type": "object",
    "required": ["name", "predicate", "match_mode", "severity", "template", "scope"],
    "properties": {
      "name": { "type": "string", "description": "Human-readable rule name. Must be unique within scope." },
      "predicate": { "type": "string", "description": "PrismQL predicate expression that defines the detection condition." },
      "match_mode": { "type": "string", "enum": ["single", "correlation", "sequence"], "description": "Detection match mode: single event, correlated events, or ordered sequence." },
      "severity": { "type": "string", "enum": ["info", "low", "medium", "high", "critical"], "description": "Alert severity when the rule fires. 'info' matches the domain entity Severity enum." },
      "template": {
        "type": "object",
        "description": "Alert template with title and description, both supporting {variable} interpolation placeholders.",
        "required": ["title", "description"],
        "properties": {
          "title": { "type": "string", "description": "Alert title template with {variable} interpolation." },
          "description": { "type": "string", "description": "Alert description template with {variable} interpolation." }
        }
      },
      "scope": {
        "type": "string",
        "pattern": "^(global|client:[a-zA-Z0-9_-]+|analyst)$",
        "description": "Rule scope: 'global', 'client:<client_id>', or 'analyst' (personal scope)."
      },
      "correlation_config": {
        "type": ["object", "null"],
        "default": null,
        "properties": {
          "group_by": { "type": "array", "items": { "type": "string" }, "description": "Fields to group correlated events by." },
          "window": { "type": "string", "description": "Time window for correlation (e.g., '5m')." },
          "threshold": { "type": "integer", "minimum": 2, "description": "Minimum event count to trigger." }
        },
        "description": "Required when match_mode is 'correlation'."
      },
      "sequence_config": {
        "type": ["object", "null"],
        "default": null,
        "properties": {
          "steps": {
            "type": "array",
            "items": {
              "type": "object",
              "properties": {
                "name": { "type": "string", "description": "Step name, unique within the sequence." },
                "condition": { "type": "string", "description": "PrismQL predicate expression for this step." },
                "step_type": { "type": "string", "enum": ["required", "optional", "negated"], "description": "Step type controlling match semantics." }
              }
            },
            "description": "Ordered sequence of steps that must match."
          },
          "window": { "type": "string", "description": "Time window for the full sequence (e.g., '10m')." }
        },
        "description": "Required when match_mode is 'sequence'."
      },
      "dry_run": { "type": "boolean", "default": true, "description": "If true, validate and preview the rule without creating it. Default: true per BC-2.04.008." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["created", "confirmation_required"] },
      "rule_id": { "type": ["string", "null"], "description": "Assigned rule ID. Present when status is 'created'." },
      "confirmation_token": { "type": ["object", "null"], "properties": { "token_id": { "type": "string" }, "action_summary": { "type": "string" }, "expires_at": { "type": "string", "format": "date-time" } }, "description": "Present when updating an existing rule (confirmation required)." }
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

### 1.21 List Rules Tool — list_rules (Subsystem 13: Detection Engine)

```json
{
  "name": "list_rules",
  "inputSchema": {
    "type": "object",
    "properties": {
      "scope": {
        "type": ["string", "null"],
        "pattern": "^(global|client:[a-zA-Z0-9_-]+|analyst)$",
        "default": null,
        "description": "Filter by scope. Null returns all accessible rules."
      },
      "client_id": {
        "type": ["string", "null"],
        "pattern": "^[a-zA-Z0-9_-]+$",
        "default": null,
        "description": "Filter by client ID. Null returns rules for all clients."
      },
      "limit": { "type": "integer", "default": 100, "minimum": 1, "maximum": 1000, "description": "Maximum number of results to return." },
      "offset": { "type": "integer", "default": 0, "minimum": 0, "description": "Number of results to skip for pagination." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "total_count": { "type": "integer", "description": "Total number of matching rules (before limit/offset)." },
      "rules": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "rule_id": { "type": "string" },
            "name": { "type": "string" },
            "match_mode": { "type": "string" },
            "severity": { "type": "string" },
            "scope": { "type": "string" },
            "enabled": { "type": "boolean" }
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

### 1.22 Delete Rule Tool — delete_rule (Subsystem 13: Detection Engine)

```json
{
  "name": "delete_rule",
  "inputSchema": {
    "type": "object",
    "required": ["rule_id", "scope"],
    "properties": {
      "rule_id": { "type": "string", "description": "ID of the rule to delete." },
      "scope": { "type": "string", "enum": ["global", "client", "analyst"], "description": "Determines deletion behavior. For scope 'client', client_id is required. For scope 'global', uses __global__ sentinel for confirmation token. For scope 'analyst', deletion is immediate (no confirmation)." },
      "client_id": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$", "description": "Client ID. Required when scope is 'client'." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["confirmation_required"] },
      "confirmation_token": { "type": "object", "properties": { "token_id": { "type": "string" }, "action_summary": { "type": "string" }, "expires_at": { "type": "string", "format": "date-time" } }, "description": "Confirmation token; call confirm_action to execute deletion." }
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

### 1.23 List Alerts Tool — list_alerts (Subsystem 13: Detection Engine)

```json
{
  "name": "list_alerts",
  "inputSchema": {
    "type": "object",
    "required": ["client_id"],
    "properties": {
      "client_id": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$", "description": "Client ID to list alerts for." },
      "severity": {
        "type": ["string", "null"],
        "enum": ["info", "low", "medium", "high", "critical", null],
        "default": null,
        "description": "Filter by severity. Null returns all severities."
      },
      "status": {
        "type": ["string", "null"],
        "enum": ["open", "acknowledged", "resolved", null],
        "default": null,
        "description": "Filter by alert status. Null returns all statuses."
      },
      "since": {
        "type": ["string", "null"],
        "format": "date-time",
        "default": null,
        "description": "Return alerts created after this timestamp. Null returns all."
      },
      "limit": { "type": "integer", "default": 100, "minimum": 1, "maximum": 1000, "description": "Maximum number of results to return." },
      "offset": { "type": "integer", "default": 0, "minimum": 0, "description": "Number of results to skip for pagination." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "total_count": { "type": "integer", "description": "Total number of matching alerts (before limit/offset)." },
      "alerts": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "alert_id": { "type": "string" },
            "rule_id": { "type": "string" },
            "rule_name": { "type": "string" },
            "severity": { "type": "string" },
            "status": { "type": "string" },
            "client_id": { "type": "string" },
            "created_at": { "type": "string", "format": "date-time" },
            "matched_event_count": { "type": "integer" }
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

### 1.24 Get Alert Tool — get_alert (Subsystem 13: Detection Engine)

```json
{
  "name": "get_alert",
  "inputSchema": {
    "type": "object",
    "required": ["alert_id", "client_id"],
    "properties": {
      "alert_id": { "type": "string", "description": "ID of the alert to retrieve." },
      "client_id": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$", "description": "Client ID for authorization. The alert is only returned if its client_id matches this value." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "alert": {
        "type": "object",
        "properties": {
          "alert_id": { "type": "string" },
          "rule_id": { "type": "string" },
          "rule_name": { "type": "string" },
          "severity": { "type": "string" },
          "status": { "type": "string" },
          "client_id": { "type": "string" },
          "created_at": { "type": "string", "format": "date-time" },
          "matched_events": {
            "type": "array",
            "items": { "type": "object", "description": "OCSF-normalized event records that triggered the alert." }
          },
          "template_rendered": { "type": "string", "description": "Alert message with interpolated field values." }
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

### 1.24b Acknowledge Alert Tool — acknowledge_alert (Subsystem 13: Detection Engine)

```json
{
  "name": "acknowledge_alert",
  "inputSchema": {
    "type": "object",
    "required": ["alert_id", "client_id"],
    "properties": {
      "alert_id": { "type": "string", "description": "ID of the alert to acknowledge." },
      "client_id": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$", "description": "Client ID that owns the alert." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "alert": {
        "type": "object",
        "properties": {
          "alert_id": { "type": "string" },
          "status": { "type": "string", "enum": ["acknowledged"] },
          "acknowledged_at": { "type": "string", "format": "date-time" }
        }
      }
    }
  },
  "annotations": {
    "readOnlyHint": false,
    "destructiveHint": false,
    "idempotentHint": true,
    "openWorldHint": false
  }
}
```

### 1.25 Create Case Tool — create_case (Subsystem 14: Case Management)

```json
{
  "name": "create_case",
  "inputSchema": {
    "type": "object",
    "required": ["title", "client_id"],
    "properties": {
      "title": { "type": "string", "minLength": 1, "maxLength": 256, "description": "Case title (1-256 chars)." },
      "client_id": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$", "description": "Client ID for the case." },
      "description": { "type": ["string", "null"], "default": null, "description": "Case description. Null for no description." },
      "alert_ids": {
        "type": "array",
        "items": { "type": "string" },
        "default": [],
        "description": "Alert IDs to link to the case. Empty array creates a manual investigation case (EC-14-001)."
      },
      "severity": {
        "type": ["string", "null"],
        "enum": ["info", "low", "medium", "high", "critical", null],
        "default": null,
        "description": "Case severity. Null infers from highest-severity linked alert, or 'medium' if no alerts."
      },
      "assignee": {
        "type": ["string", "null"],
        "default": null,
        "description": "Analyst identifier to assign the case to. Null for unassigned."
      }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "case_id": { "type": "string", "description": "Unique identifier for the created case." },
      "status": { "type": "string", "enum": ["new"], "description": "Initial case status (always 'new')." },
      "severity": { "type": "string", "description": "Resolved severity (explicit or inferred)." }
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

### 1.26 Update Case Tool — update_case (Subsystem 14: Case Management)

```json
{
  "name": "update_case",
  "inputSchema": {
    "type": "object",
    "required": ["case_id", "client_id"],
    "properties": {
      "case_id": { "type": "string", "description": "ID of the case to update." },
      "client_id": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$", "description": "Client ID that owns the case. Required for DI-008 client data separation." },
      "status": {
        "type": ["string", "null"],
        "enum": ["new", "acknowledged", "investigating", "resolved", "closed", null],
        "default": null,
        "description": "New case status per the 5-state model (DI-025). Null leaves status unchanged."
      },
      "disposition": {
        "type": ["object", "null"],
        "default": null,
        "description": "Case disposition as a structured object. Required when status is 'resolved'. The `detail` field maps to variant-specific metadata: `impact_level` for true_positive, `reason` for false_positive, `explanation` for benign. Inconclusive does not require detail.",
        "properties": {
          "variant": { "type": "string", "enum": ["true_positive", "false_positive", "benign", "inconclusive"], "description": "Disposition classification." },
          "detail": { "type": ["string", "null"], "description": "Variant-specific detail: impact_level (true_positive), reason (false_positive), explanation (benign). Optional for inconclusive." }
        },
        "required": ["variant"]
      },
      "annotation": {
        "type": ["object", "null"],
        "default": null,
        "description": "Annotation to append to the case timeline.",
        "properties": {
          "type": { "type": "string", "enum": ["note", "evidence_link", "ot_impact"], "description": "Annotation type. status_change and alert_link are system-generated only." },
          "content": { "type": "string", "minLength": 1, "maxLength": 10000, "description": "Annotation content text." }
        },
        "required": ["type", "content"]
      },
      "severity": {
        "type": ["string", "null"],
        "enum": ["info", "low", "medium", "high", "critical", null],
        "default": null,
        "description": "Updated case severity. Null leaves severity unchanged."
      },
      "assignee": {
        "type": ["string", "null"],
        "default": null,
        "description": "Analyst identifier to assign the case to. Null leaves assignee unchanged."
      },
      "link_alert_ids": {
        "type": ["array", "null"],
        "default": null,
        "description": "Alert IDs to link to this case. Appended to existing linked alerts.",
        "items": { "type": "string" }
      }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "case": {
        "type": "object",
        "properties": {
          "case_id": { "type": "string" },
          "status": { "type": "string" },
          "disposition": { "type": ["object", "null"], "properties": { "variant": { "type": "string", "enum": ["true_positive", "false_positive", "benign", "inconclusive"] }, "detail": { "type": ["string", "null"] } } },
          "updated_at": { "type": "string", "format": "date-time" }
        }
      }
    }
  },
  "annotations": {
    "readOnlyHint": false,
    "destructiveHint": false,
    "idempotentHint": true,
    "openWorldHint": false
  }
}
```

### 1.27 List Cases Tool — list_cases (Subsystem 14: Case Management)

```json
{
  "name": "list_cases",
  "inputSchema": {
    "type": "object",
    "properties": {
      "client_id": {
        "type": ["string", "null"],
        "pattern": "^[a-zA-Z0-9_-]+$",
        "default": null,
        "description": "Filter by client ID. Null returns cases for all clients."
      },
      "status": {
        "type": ["string", "null"],
        "enum": ["new", "acknowledged", "investigating", "resolved", "closed", null],
        "default": null,
        "description": "Filter by case status per the 5-state model (DI-025). Null returns all statuses."
      },
      "severity": {
        "type": ["string", "null"],
        "enum": ["info", "low", "medium", "high", "critical", null],
        "default": null,
        "description": "Filter by case severity."
      },
      "assignee": {
        "type": ["string", "null"],
        "default": null,
        "description": "Filter by assignee identifier."
      },
      "sort_by": {
        "type": "string",
        "enum": ["created_at", "updated_at", "severity", "status"],
        "default": "created_at",
        "description": "Field to sort results by."
      },
      "sort_order": {
        "type": "string",
        "enum": ["asc", "desc"],
        "default": "desc",
        "description": "Sort direction."
      },
      "limit": { "type": "integer", "default": 25, "minimum": 1, "maximum": 100, "description": "Maximum number of results to return." },
      "offset": { "type": "integer", "default": 0, "minimum": 0, "description": "Number of results to skip for pagination." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "total_count": { "type": "integer", "description": "Total number of matching cases (before limit/offset)." },
      "cases": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "case_id": { "type": "string" },
            "title": { "type": "string" },
            "client_id": { "type": "string" },
            "status": { "type": "string" },
            "severity": { "type": "string" },
            "assignee": { "type": ["string", "null"] },
            "disposition": { "type": ["object", "null"], "properties": { "variant": { "type": "string" }, "detail": { "type": ["string", "null"] } } },
            "alert_count": { "type": "integer" },
            "created_at": { "type": "string", "format": "date-time" },
            "updated_at": { "type": "string", "format": "date-time" }
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

### 1.28 Get Case Tool — get_case (Subsystem 14: Case Management)

```json
{
  "name": "get_case",
  "inputSchema": {
    "type": "object",
    "required": ["case_id", "client_id"],
    "properties": {
      "case_id": { "type": "string", "description": "ID of the case to retrieve." },
      "client_id": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$", "description": "Client ID. Required for DI-008 client data separation." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "case": {
        "type": "object",
        "properties": {
          "case_id": { "type": "string" },
          "title": { "type": "string" },
          "client_id": { "type": "string" },
          "status": { "type": "string" },
          "disposition": { "type": ["object", "null"], "properties": { "variant": { "type": "string", "enum": ["true_positive", "false_positive", "benign", "inconclusive"] }, "detail": { "type": ["string", "null"] } } },
          "created_at": { "type": "string", "format": "date-time" },
          "updated_at": { "type": "string", "format": "date-time" },
          "timeline": {
            "type": "array",
            "items": {
              "type": "object",
              "properties": {
                "timestamp": { "type": "string", "format": "date-time" },
                "event_type": { "type": "string", "enum": ["created", "status_change", "annotation", "alert_added", "disposition_set", "priority_changed", "assignee_changed"] },
                "detail": { "type": "string" }
              }
            },
            "description": "Chronological timeline of case events."
          },
          "alerts": {
            "type": "array",
            "items": { "type": "object", "description": "Alert summaries linked to this case." }
          },
          "metrics": {
            "type": "object",
            "properties": {
              "time_to_detect": { "type": ["number", "null"], "description": "Seconds from event to alert (MTTD)." },
              "time_to_respond": { "type": ["number", "null"], "description": "Seconds from alert to case resolution (MTTR)." }
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

### 1.29 Case Metrics Tool — case_metrics (Subsystem 14: Case Management)

```json
{
  "name": "case_metrics",
  "inputSchema": {
    "type": "object",
    "properties": {
      "client_id": {
        "type": ["string", "null"],
        "pattern": "^[a-zA-Z0-9_-]+$",
        "default": null,
        "description": "Filter metrics by client ID. Null aggregates across all clients."
      },
      "since": {
        "type": ["string", "null"],
        "format": "date-time",
        "default": null,
        "description": "Compute metrics for cases created after this timestamp. Null includes all cases."
      }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "mttd_seconds": { "type": ["number", "null"], "description": "Mean Time To Detect (seconds from event to alert)." },
      "mttr_seconds": { "type": ["number", "null"], "description": "Mean Time To Respond (seconds from alert to case resolution)." },
      "counts_by_status": {
        "type": "object",
        "properties": {
          "new": { "type": "integer" },
          "acknowledged": { "type": "integer" },
          "investigating": { "type": "integer" },
          "resolved": { "type": "integer" },
          "closed": { "type": "integer" }
        }
      },
      "total_cases": { "type": "integer" }
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

### 1.30 Watchdog Status Tool — watchdog_status (Subsystem 15: Platform)

```json
{
  "name": "watchdog_status",
  "inputSchema": {
    "type": "object",
    "properties": {}
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "limits": {
        "type": "object",
        "properties": {
          "max_query_memory_bytes": { "type": "integer" },
          "max_concurrent_queries": { "type": "integer" },
          "max_concurrent_schedules": { "type": "integer" }
        },
        "description": "Current resource limits enforced by the watchdog."
      },
      "denylisted_queries": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "query_hash": { "type": "string" },
            "reason": { "type": "string" },
            "added_at": { "type": "string", "format": "date-time" }
          }
        },
        "description": "Queries currently on the denylist due to resource violations."
      },
      "memory_usage": {
        "type": "object",
        "properties": {
          "current_bytes": { "type": "integer" },
          "peak_bytes": { "type": "integer" },
          "budget_bytes": { "type": "integer" }
        },
        "description": "Current memory usage statistics."
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

### 1.31 Reload Config Tool — reload_config (Subsystem 16: Config-Driven Sensor Adapters)

```json
{
  "name": "reload_config",
  "inputSchema": {
    "type": "object",
    "properties": {
      "dry_run": {
        "type": "boolean",
        "default": false,
        "description": "If true, validate the new config without applying it. Returns validation results only."
      }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["reloaded", "partial", "failed", "dry_run_ok", "dry_run_failed"], "description": "Overall reload result. 'partial' means some tiers succeeded (see per-tier results)." },
      "tiers": {
        "type": "object",
        "properties": {
          "config": { "type": "object", "properties": { "status": { "type": "string", "enum": ["ok", "failed", "unchanged"] }, "errors": { "type": "array", "items": { "type": "string" } } }, "description": "Tier 1: prism.toml (all-or-nothing per DI-031)." },
          "aliases": { "type": "object", "properties": { "status": { "type": "string", "enum": ["ok", "failed", "unchanged"] }, "errors": { "type": "array", "items": { "type": "string" } } }, "description": "Tier 2: aliases.toml (all-or-nothing per DI-031)." },
          "sensor_specs": {
            "type": "object",
            "properties": {
              "status": { "type": "string", "enum": ["ok", "partial", "failed", "unchanged"] },
              "loaded": { "type": "array", "items": { "type": "string" }, "description": "Spec files successfully loaded." },
              "rejected": {
                "type": "array",
                "items": {
                  "type": "object",
                  "properties": {
                    "file": { "type": "string" },
                    "errors": { "type": "array", "items": { "type": "string" } }
                  }
                },
                "description": "Spec files that failed validation (per-file independent, DI-030)."
              }
            },
            "description": "Tier 3: sensor spec files (per-file independent per DI-031)."
          }
        }
      },
      "tools_changed": { "type": "boolean", "description": "Whether the available tool list changed (triggers notifications/tools/list_changed)." }
    }
  },
  "annotations": {
    "readOnlyHint": false,
    "destructiveHint": false,
    "idempotentHint": true,
    "openWorldHint": false
  }
}
```

### 1.32 Add Sensor Spec Tool — add_sensor_spec (Subsystem 16: Config-Driven Sensor Adapters)

```json
{
  "name": "add_sensor_spec",
  "inputSchema": {
    "type": "object",
    "required": ["spec_toml"],
    "properties": {
      "spec_toml": { "type": "string", "description": "Full TOML content of the sensor spec file to add." },
      "file_name": {
        "type": ["string", "null"],
        "pattern": "^[a-z][a-z0-9_-]*\\.sensor\\.toml$",
        "default": null,
        "description": "File name to save as. If null, derived from sensor_id in the spec (e.g., 'newvendor.sensor.toml')."
      },
      "dry_run": {
        "type": "boolean",
        "default": false,
        "description": "If true, validate the spec without persisting it."
      }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["added", "validation_failed", "dry_run_ok", "dry_run_failed", "confirmation_required"], "description": "Result of the add operation." },
      "sensor_id": { "type": "string", "description": "The sensor_id from the parsed spec." },
      "tables": { "type": "array", "items": { "type": "string" }, "description": "Table names registered by this spec." },
      "validation_errors": {
        "type": ["array", "null"],
        "items": { "type": "string" },
        "description": "Validation errors if status is validation_failed or dry_run_failed."
      },
      "confirmation_token": { "type": ["object", "null"], "properties": { "token_id": { "type": "string" }, "action_summary": { "type": "string" }, "expires_at": { "type": "string", "format": "date-time" } }, "description": "Present when replacing an existing spec (confirmation required)." }
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

### 1.33 List Sensor Specs Tool — list_sensor_specs (Subsystem 16: Config-Driven Sensor Adapters)

```json
{
  "name": "list_sensor_specs",
  "inputSchema": {
    "type": "object",
    "properties": {
      "sensor_id": {
        "type": ["string", "null"],
        "pattern": "^[a-z][a-z0-9_-]*$",
        "default": null,
        "description": "Filter to a specific sensor. Null returns all loaded specs."
      },
      "include_tables": {
        "type": "boolean",
        "default": true,
        "description": "Include table definitions in the response."
      }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "specs": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "sensor_id": { "type": "string" },
            "display_name": { "type": "string" },
            "source": { "type": "string", "enum": ["file", "runtime"], "description": "Whether loaded from disk or added via add_sensor_spec at runtime." },
            "file_path": { "type": ["string", "null"], "description": "Path to the spec file (null for runtime-added specs)." },
            "version": { "type": ["string", "null"] },
            "tables": {
              "type": "array",
              "items": {
                "type": "object",
                "properties": {
                  "table_name": { "type": "string" },
                  "ocsf_class": { "type": "string" },
                  "column_count": { "type": "integer" },
                  "required_columns": { "type": "array", "items": { "type": "string" } }
                }
              },
              "description": "Tables registered by this spec (omitted if include_tables is false)."
            },
            "auth_type": { "type": "string", "description": "Authentication type (e.g., 'oauth2', 'bearer_token', 'api_key')." }
          }
        }
      },
      "total_count": { "type": "integer" }
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

### 1.34 Credential Status Tool — credential_status (Subsystem 03: Credential Management)

Always-visible read-only tool. Returns set/missing status per `(client_id, sensor_id, credential_name)` with source type. Values are NEVER returned (AI-opaque credentials model).

```json
{
  "name": "credential_status",
  "inputSchema": {
    "type": "object",
    "required": ["client_id"],
    "properties": {
      "client_id": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$", "description": "Client ID. Required and non-null — cross-client credential status is not supported to prevent MSSP portfolio disclosure." },
      "sensor_id": { "type": ["string", "null"], "pattern": "^[a-z][a-z0-9_-]*$", "default": null, "description": "Filter to a specific sensor. Null returns status for all sensors." }
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
            "sensor_id": { "type": "string" },
            "name": { "type": "string", "description": "Credential key name." },
            "status": { "type": "string", "enum": ["set", "missing"], "description": "Whether the credential is configured and resolvable." },
            "source_type": { "type": ["string", "null"], "enum": ["keyring", "env", "vault", "file", null], "description": "Source backend type if set; null if missing." }
          }
        },
        "description": "Per-credential status entries. Values are NEVER included."
      }
    }
  },
  "errors": ["E-CRED-001 (credential not found)", "E-AUTH-002 (client not found)"],
  "annotations": {
    "readOnlyHint": true,
    "destructiveHint": false,
    "idempotentHint": true,
    "openWorldHint": false
  }
}
```

### 1.35 CrowdStrike Lift Containment Tool — crowdstrike_lift_containment (Subsystem 08: Sensor Adapters)

**Capability gate:** `sensor.crowdstrike.containment` — Reversible

Lifts network isolation (containment) on a CrowdStrike-managed host. Uses the reversible dry-run pattern: first call returns a preview, second call with `dry_run: false` executes via the CrowdStrike API.

```json
{
  "name": "crowdstrike_lift_containment",
  "inputSchema": {
    "type": "object",
    "required": ["client_id", "device_id"],
    "properties": {
      "client_id": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$", "description": "Client ID. Must be non-null for write operations." },
      "device_id": { "type": "string", "description": "CrowdStrike host/device ID to lift containment on." },
      "reason": { "type": ["string", "null"], "default": null, "description": "Human-readable justification. Included in audit log." },
      "dry_run": { "type": "boolean", "default": true, "description": "If true, preview the operation without executing. Default: true per reversible write pattern." }
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
      "status": { "type": "string", "enum": ["preview", "executed", "failed"] },
      "preview": {
        "type": ["object", "null"],
        "properties": {
          "device_id": { "type": "string" },
          "hostname": { "type": ["string", "null"] },
          "current_status": { "type": "string", "description": "Current containment state of the host." }
        },
        "description": "Present when dry_run=true. Shows what would be executed."
      },
      "result": {
        "type": ["object", "null"],
        "properties": {
          "device_id": { "type": "string" },
          "contained": { "type": "boolean", "description": "False after successful lift." },
          "sensor_response": { "type": "object", "description": "Raw CrowdStrike API response." }
        },
        "description": "Present when dry_run=false and execution succeeded."
      }
    }
  },
  "errors": ["E-SENSOR-001 (sensor unreachable)", "E-AUTH-001 (auth failure)", "E-FLAG-001 (capability disabled)"],
  "annotations": {
    "readOnlyHint": false,
    "destructiveHint": false,
    "idempotentHint": true,
    "openWorldHint": true
  }
}
```

### 1.36 Get Help Tool — get_help (Subsystem 11: Query Engine)

Always-visible bridge tool. Lets the AI agent actively retrieve reference documentation without depending on Claude Code having pre-loaded MCP resources. Reads the same content as the `prism://docs/` resources.

```json
{
  "name": "get_help",
  "inputSchema": {
    "type": "object",
    "required": ["topic"],
    "properties": {
      "topic": {
        "type": "string",
        "description": "Documentation topic to retrieve.",
        "enum": ["prismql", "prismql.functions", "prismql.pipes", "prismql.examples", "ocsf.fields", "detection-rules", "errors"]
      }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "topic": { "type": "string" },
      "content": { "type": "string", "description": "Markdown documentation for the requested topic." },
      "mime_type": { "type": "string", "const": "text/markdown" }
    }
  },
  "errors": ["E-QUERY-010 (unknown topic)"],
  "annotations": {
    "readOnlyHint": true,
    "destructiveHint": false,
    "idempotentHint": true,
    "openWorldHint": false
  }
}
```

Note: `errors.{code}` lookup (single error code) is handled by passing `topic: "errors"` then filtering, or by querying the `prism://docs/errors/{code}` resource directly.

### 1.37 Get Diagnostics Tool — get_diagnostics (Subsystem 15: Platform / Observability)

Always-visible read-only tool. Returns aggregated operational state, event counts, and recent errors/warnings for a named subsystem. Mirrors the `prism://diagnostics/{subsystem}` resource.

```json
{
  "name": "get_diagnostics",
  "inputSchema": {
    "type": "object",
    "required": ["subsystem"],
    "properties": {
      "subsystem": {
        "type": "string",
        "enum": ["scheduler", "detection", "actions", "config", "plugins", "infusions", "credentials", "fanout", "watchdog", "storage"],
        "description": "Subsystem to retrieve diagnostics for."
      },
      "client_id": { "type": ["string", "null"], "pattern": "^[a-zA-Z0-9_-]+$", "default": null, "description": "Scope diagnostics to a specific client. Null returns cross-client aggregated state." },
      "since": { "type": ["string", "null"], "format": "date-time", "default": null, "description": "Return errors/warnings since this timestamp. Null returns the most recent entries (up to 100)." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "subsystem": { "type": "string" },
      "status": { "type": "string", "enum": ["healthy", "degraded", "failed"], "description": "Overall subsystem health." },
      "state": { "type": "object", "description": "Subsystem-specific state snapshot (counts, rates, resource usage). Structure varies per subsystem." },
      "recent_errors": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "timestamp": { "type": "string", "format": "date-time" },
            "error_code": { "type": "string", "pattern": "^E-[A-Z]+-\\d{3}$" },
            "message": { "type": "string" },
            "client_id": { "type": ["string", "null"] }
          }
        },
        "description": "Recent error log entries for this subsystem."
      },
      "recent_warnings": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "timestamp": { "type": "string", "format": "date-time" },
            "message": { "type": "string" },
            "client_id": { "type": ["string", "null"] }
          }
        },
        "description": "Recent warning log entries for this subsystem."
      }
    }
  },
  "errors": ["E-DIAG-001 (subsystem not found)"],
  "annotations": {
    "readOnlyHint": true,
    "destructiveHint": false,
    "idempotentHint": true,
    "openWorldHint": false
  }
}
```

### 1.38 List Plugins Tool — list_plugins (Subsystem 17: WASM Plugin Runtime)

Always-visible read-only tool. Lists all loaded WASM plugins with load status, ABI version, memory usage, and CPU epoch stats.

```json
{
  "name": "list_plugins",
  "inputSchema": {
    "type": "object",
    "properties": {}
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "plugins": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "plugin_id": { "type": "string", "description": "Unique plugin identifier." },
            "display_name": { "type": ["string", "null"] },
            "status": { "type": "string", "enum": ["loaded", "loading", "failed", "unloading"], "description": "Current load status." },
            "abi_version": { "type": "string", "description": "WASM ABI version the plugin was compiled against." },
            "memory_bytes": { "type": "integer", "description": "Current memory usage of the plugin's WASM linear memory." },
            "cpu_epochs_consumed": { "type": "integer", "description": "Total CPU epoch units consumed by this plugin." },
            "last_error": { "type": ["string", "null"], "description": "Most recent error message if status is 'failed'." }
          }
        }
      },
      "total_count": { "type": "integer" }
    }
  },
  "errors": [],
  "annotations": {
    "readOnlyHint": true,
    "destructiveHint": false,
    "idempotentHint": true,
    "openWorldHint": false
  }
}
```

### 1.39 Plugin Status Tool — plugin_status (Subsystem 17: WASM Plugin Runtime)

Always-visible read-only tool. Returns detailed runtime statistics for a named WASM plugin.

```json
{
  "name": "plugin_status",
  "inputSchema": {
    "type": "object",
    "required": ["plugin_id"],
    "properties": {
      "plugin_id": { "type": "string", "description": "Plugin identifier to retrieve status for." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "plugin": {
        "type": "object",
        "properties": {
          "plugin_id": { "type": "string" },
          "display_name": { "type": ["string", "null"] },
          "status": { "type": "string", "enum": ["loaded", "loading", "failed", "unloading"] },
          "abi_version": { "type": "string" },
          "loaded_at": { "type": "string", "format": "date-time" },
          "invoke_latency_p50_ms": { "type": ["number", "null"], "description": "Median invocation latency in milliseconds." },
          "invoke_latency_p99_ms": { "type": ["number", "null"], "description": "p99 invocation latency in milliseconds." },
          "memory_bytes_current": { "type": "integer" },
          "memory_bytes_peak": { "type": "integer" },
          "cpu_epochs_consumed": { "type": "integer" },
          "last_error": { "type": ["string", "null"] }
        }
      }
    }
  },
  "errors": ["E-PLUGIN-001 (plugin not found)"],
  "annotations": {
    "readOnlyHint": true,
    "destructiveHint": false,
    "idempotentHint": true,
    "openWorldHint": false
  }
}
```

### 1.40 Reload Plugin Tool — reload_plugin (Subsystem 17: WASM Plugin Runtime)

**Capability gate:** `plugin.write` — Reversible

Hot-reloads a WASM plugin: compiles the new module, instantiates it, and arc-swaps the registry after in-flight calls drain. Preserves plugin state continuity.

```json
{
  "name": "reload_plugin",
  "inputSchema": {
    "type": "object",
    "required": ["plugin_id"],
    "properties": {
      "plugin_id": { "type": "string", "description": "Plugin identifier to reload." },
      "dry_run": { "type": "boolean", "default": true, "description": "If true, validate the reload path without executing. Default: true per reversible write pattern." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["reloaded", "dry_run_ok", "failed"] },
      "plugin_id": { "type": "string" },
      "previous_abi_version": { "type": ["string", "null"] },
      "new_abi_version": { "type": ["string", "null"] },
      "drain_wait_ms": { "type": ["integer", "null"], "description": "Time spent waiting for in-flight calls to drain before swap (ms)." }
    }
  },
  "errors": ["E-PLUGIN-001 (plugin not found)", "E-PLUGIN-002 (compile error)", "E-FLAG-001 (capability disabled)"],
  "annotations": {
    "readOnlyHint": false,
    "destructiveHint": false,
    "idempotentHint": false,
    "openWorldHint": false
  }
}
```

### 1.41 List Infusions Tool — list_infusions (Subsystem 19: Infusion Engine)

Always-visible read-only tool. Lists all loaded infusion specs with status, source type, data age, and cache hit rates.

```json
{
  "name": "list_infusions",
  "inputSchema": {
    "type": "object",
    "properties": {
      "client_id": { "type": ["string", "null"], "pattern": "^[a-zA-Z0-9_-]+$", "default": null, "description": "Filter to infusions scoped to a specific client. Null returns all loaded infusions." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "infusions": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "infusion_id": { "type": "string", "description": "Unique infusion identifier." },
            "status": { "type": "string", "enum": ["loaded", "loading", "failed", "stale"], "description": "Current data load status." },
            "source_type": { "type": "string", "enum": ["file", "url", "inline"], "description": "Where the infusion data is sourced from." },
            "record_count": { "type": "integer", "description": "Number of records currently loaded." },
            "data_age_seconds": { "type": ["number", "null"], "description": "Age of the loaded data in seconds. Null if never loaded." },
            "cache_hit_rate": { "type": ["number", "null"], "description": "Three-tier cache hit rate (0.0-1.0) since last reload." },
            "next_reload_at": { "type": ["string", "null"], "format": "date-time", "description": "Scheduled next reload time. Null if no auto-reload is configured." }
          }
        }
      },
      "total_count": { "type": "integer" }
    }
  },
  "errors": [],
  "annotations": {
    "readOnlyHint": true,
    "destructiveHint": false,
    "idempotentHint": true,
    "openWorldHint": false
  }
}
```

### 1.42 Infusion Status Tool — infusion_status (Subsystem 19: Infusion Engine)

Always-visible read-only tool. Returns detailed status for a named infusion: data file path, age, records loaded, three-tier cache stats, and next scheduled reload.

```json
{
  "name": "infusion_status",
  "inputSchema": {
    "type": "object",
    "required": ["infusion_id"],
    "properties": {
      "infusion_id": { "type": "string", "description": "Infusion identifier to retrieve status for." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "infusion": {
        "type": "object",
        "properties": {
          "infusion_id": { "type": "string" },
          "status": { "type": "string", "enum": ["loaded", "loading", "failed", "stale"] },
          "source_type": { "type": "string", "enum": ["file", "url", "inline"] },
          "source_path": { "type": ["string", "null"], "description": "File path or URL of the data source." },
          "record_count": { "type": "integer" },
          "data_age_seconds": { "type": ["number", "null"] },
          "last_reload_at": { "type": ["string", "null"], "format": "date-time" },
          "next_reload_at": { "type": ["string", "null"], "format": "date-time" },
          "cache": {
            "type": "object",
            "properties": {
              "l1_hit_rate": { "type": "number", "description": "In-process hot-path cache hit rate (0.0-1.0)." },
              "l2_hit_rate": { "type": "number", "description": "Thread-local cache hit rate (0.0-1.0)." },
              "l3_hit_rate": { "type": "number", "description": "Arc-swapped registry lookup hit rate (0.0-1.0)." }
            },
            "description": "Three-tier cache statistics."
          },
          "last_error": { "type": ["string", "null"] }
        }
      }
    }
  },
  "errors": ["E-INFUSION-001 (infusion not found)"],
  "annotations": {
    "readOnlyHint": true,
    "destructiveHint": false,
    "idempotentHint": true,
    "openWorldHint": false
  }
}
```

### 1.43 Reload Infusion Tool — reload_infusion (Subsystem 19: Infusion Engine)

**Capability gate:** `infusion.write` — Reversible

Triggers an immediate data reload for the named infusion: re-reads the source file or URL, arc-swaps the registry after validation completes.

```json
{
  "name": "reload_infusion",
  "inputSchema": {
    "type": "object",
    "required": ["infusion_id"],
    "properties": {
      "infusion_id": { "type": "string", "description": "Infusion identifier to reload." },
      "dry_run": { "type": "boolean", "default": true, "description": "If true, validate the source without persisting. Default: true per reversible write pattern." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["reloaded", "dry_run_ok", "failed"] },
      "infusion_id": { "type": "string" },
      "record_count": { "type": ["integer", "null"], "description": "Records loaded after reload. Null on dry_run." },
      "load_duration_ms": { "type": ["integer", "null"], "description": "Time taken to load and validate the data (ms). Null on dry_run." }
    }
  },
  "errors": ["E-INFUSION-001 (infusion not found)", "E-INFUSION-002 (source read error)", "E-FLAG-001 (capability disabled)"],
  "annotations": {
    "readOnlyHint": false,
    "destructiveHint": false,
    "idempotentHint": false,
    "openWorldHint": false
  }
}
```

### 1.44 List Actions Tool — list_actions (Subsystem 18: Action Engine)

Always-visible read-only tool. Lists configured actions with status, trigger type, and last fired timestamp.

```json
{
  "name": "list_actions",
  "inputSchema": {
    "type": "object",
    "properties": {
      "client_id": { "type": ["string", "null"], "pattern": "^[a-zA-Z0-9_-]+$", "default": null, "description": "Filter to actions scoped to a specific client. Null returns all actions." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "actions": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "action_id": { "type": "string", "description": "Unique action identifier." },
            "display_name": { "type": ["string", "null"] },
            "trigger_type": { "type": "string", "enum": ["detection_alert", "schedule", "manual"], "description": "What triggers this action." },
            "destination_type": { "type": "string", "description": "Delivery destination (e.g., 'webhook', 'email', 'pagerduty', 'slack')." },
            "status": { "type": "string", "enum": ["active", "disabled", "error"], "description": "Current operational status." },
            "last_fired_at": { "type": ["string", "null"], "format": "date-time" }
          }
        }
      },
      "total_count": { "type": "integer" }
    }
  },
  "errors": [],
  "annotations": {
    "readOnlyHint": true,
    "destructiveHint": false,
    "idempotentHint": true,
    "openWorldHint": false
  }
}
```

### 1.45 Action Status Tool — action_status (Subsystem 18: Action Engine)

Always-visible read-only tool. Returns detailed status for a named action: last fire time, success/failure counts, rate limit state, and suppressed count.

```json
{
  "name": "action_status",
  "inputSchema": {
    "type": "object",
    "required": ["action_id"],
    "properties": {
      "action_id": { "type": "string", "description": "Action identifier to retrieve status for." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "action": {
        "type": "object",
        "properties": {
          "action_id": { "type": "string" },
          "display_name": { "type": ["string", "null"] },
          "status": { "type": "string", "enum": ["active", "disabled", "error"] },
          "trigger_type": { "type": "string", "enum": ["detection_alert", "schedule", "manual"] },
          "destination_type": { "type": "string" },
          "last_fired_at": { "type": ["string", "null"], "format": "date-time" },
          "fire_count_success": { "type": "integer", "description": "Total successful deliveries." },
          "fire_count_failure": { "type": "integer", "description": "Total failed delivery attempts." },
          "suppressed_count": { "type": "integer", "description": "Deliveries suppressed due to rate limiting." },
          "rate_limit": {
            "type": "object",
            "properties": {
              "is_rate_limited": { "type": "boolean" },
              "reset_at": { "type": ["string", "null"], "format": "date-time" }
            },
            "description": "Current rate limit state for this action's destination."
          },
          "last_error": { "type": ["string", "null"] }
        }
      }
    }
  },
  "errors": ["E-ACTION-001 (action not found)"],
  "annotations": {
    "readOnlyHint": true,
    "destructiveHint": false,
    "idempotentHint": true,
    "openWorldHint": false
  }
}
```

### 1.46 Fire Action Tool — fire_action (Subsystem 18: Action Engine)

**Capability gate:** `action.write` — Reversible

Manually triggers an action with a supplied context payload. Uses the dry-run pattern: first call returns a preview of what would be delivered, second call with `dry_run: false` executes delivery. Traces to BC-2.18.003, S-5.06-action-infusion-tools, S-4.08-action-delivery.

```json
{
  "name": "fire_action",
  "inputSchema": {
    "type": "object",
    "required": ["action_id"],
    "properties": {
      "action_id": { "type": "string", "description": "Action identifier to fire." },
      "context": {
        "type": ["object", "null"],
        "default": null,
        "description": "JSON context payload injected into the action template. Keys map to action template variables. Null uses an empty context."
      },
      "dry_run": { "type": "boolean", "default": true, "description": "If true, render the action payload without delivering it. Default: true per reversible write pattern." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["fired", "dry_run_ok", "failed"] },
      "action_id": { "type": "string" },
      "rendered_payload": {
        "type": ["object", "null"],
        "description": "The rendered delivery payload (present on both dry_run and live fire). Lets the analyst verify template expansion before committing."
      },
      "delivery_result": {
        "type": ["object", "null"],
        "properties": {
          "destination_type": { "type": "string" },
          "http_status": { "type": ["integer", "null"] },
          "latency_ms": { "type": ["integer", "null"] }
        },
        "description": "Present when dry_run=false and delivery completed."
      }
    }
  },
  "errors": ["E-ACTION-001 (action not found)", "E-ACTION-002 (delivery failure)", "E-FLAG-001 (capability disabled)"],
  "annotations": {
    "readOnlyHint": false,
    "destructiveHint": false,
    "idempotentHint": false,
    "openWorldHint": true
  }
}
```

### 1.47 Test Action Tool — test_action (Subsystem 18: Action Engine)

**Capability gate:** `action.write` — Reversible

Sends a synthetic test payload to an action's configured destination to validate connectivity and template rendering. Does not require an alert context — uses hardcoded test data. Traces to S-5.06-action-infusion-tools.

```json
{
  "name": "test_action",
  "inputSchema": {
    "type": "object",
    "required": ["action_id"],
    "properties": {
      "action_id": { "type": "string", "description": "Action identifier to test." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["delivered", "failed"] },
      "action_id": { "type": "string" },
      "destination_type": { "type": "string" },
      "rendered_payload": { "type": ["object", "null"], "description": "The test payload that was sent to the destination." },
      "http_status": { "type": ["integer", "null"], "description": "HTTP response status from the destination (if applicable)." },
      "latency_ms": { "type": ["integer", "null"] },
      "error": { "type": ["string", "null"], "description": "Error message if status is 'failed'." }
    }
  },
  "errors": ["E-ACTION-001 (action not found)", "E-ACTION-002 (delivery failure)", "E-FLAG-001 (capability disabled)"],
  "annotations": {
    "readOnlyHint": false,
    "destructiveHint": false,
    "idempotentHint": false,
    "openWorldHint": true
  }
}
```

### 1.48 Create Action Tool — create_action (Subsystem 18: Action Engine)

**Capability gate:** `action.write` — Reversible

Validates and loads a new action spec from TOML content. Writes to `{config_dir}/actions/{action_id}.action.toml`. Traces to BC-2.18.003, S-5.06-action-infusion-tools.

```json
{
  "name": "create_action",
  "inputSchema": {
    "type": "object",
    "required": ["spec_toml"],
    "properties": {
      "spec_toml": { "type": "string", "description": "Full TOML content of the action spec. Must define `action_id`, `trigger`, `destination`, and `template` top-level fields." },
      "dry_run": { "type": "boolean", "default": true, "description": "If true, validate the spec without persisting it. Default: true per reversible write pattern." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["created", "dry_run_ok", "validation_failed", "confirmation_required"] },
      "action_id": { "type": ["string", "null"], "description": "The action_id parsed from the spec. Null if validation_failed." },
      "validation_errors": { "type": ["array", "null"], "items": { "type": "string" }, "description": "Validation errors if status is validation_failed." },
      "confirmation_token": { "type": ["object", "null"], "properties": { "token_id": { "type": "string" }, "action_summary": { "type": "string" }, "expires_at": { "type": "string", "format": "date-time" } }, "description": "Present when replacing an existing action (confirmation required)." }
    }
  },
  "errors": ["E-ACTION-003 (spec validation error)", "E-FLAG-001 (capability disabled)"],
  "annotations": {
    "readOnlyHint": false,
    "destructiveHint": false,
    "idempotentHint": false,
    "openWorldHint": false
  }
}
```

### 1.49 Delete Action Tool — delete_action (Subsystem 18: Action Engine)

**Capability gate:** `action.write` — Irreversible

Removes an action spec file and unregisters it from the ActionEngine. In-flight executions drain before removal. Uses the confirmation-token pattern. Traces to BC-2.18.003, S-5.06-action-infusion-tools.

```json
{
  "name": "delete_action",
  "inputSchema": {
    "type": "object",
    "required": ["action_id"],
    "properties": {
      "action_id": { "type": "string", "description": "Action identifier to delete." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["confirmation_required"] },
      "confirmation_token": { "type": "object", "properties": { "token_id": { "type": "string" }, "action_summary": { "type": "string" }, "expires_at": { "type": "string", "format": "date-time" } }, "description": "Confirmation token; call confirm_action to execute deletion. In-flight executions drain before removal." }
    }
  },
  "errors": ["E-ACTION-001 (action not found)", "E-FLAG-001 (capability disabled)"],
  "annotations": {
    "readOnlyHint": false,
    "destructiveHint": true,
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

# Sensor spec files directory (CAP-029)
# All sensors — including CrowdStrike, Cyberint, Claroty, Armis — are defined as TOML spec files.
# The four initial sensors ship as bundled spec files alongside the binary.
sensor_specs_dir = "./sensor-specs"    # PRISM_SENSOR_SPECS_DIR override

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
alias.write = true                     # Allow alias mutations (create_alias, delete_alias) for this client. Required for client-scoped aliases targeting this client. For global aliases, alias.write must be enabled for at least one configured client.
schedule.write = true                  # Allow schedule mutations (create_schedule, delete_schedule) for this client.
detection.write = true                 # Allow detection rule mutations (create_rule, delete_rule) for this client. For global-scope rules, detection.write.global is additionally required.
case.write = true                      # Allow case mutations (create_case, update_case) for this client.
sensor_spec.write = true               # Allow sensor spec mutations (add_sensor_spec) for this client.
pack.write = true                      # Allow pack mutations (create_pack, delete_pack) for this client.

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
| `clients.{id}.sensors.{sensor}` | table | no | — | Sensor config; `{sensor}` must match a loaded spec file's `sensor_id` (e.g., crowdstrike, cyberint, claroty, armis, or any config-driven sensor) |
| `clients.{id}.sensors.{sensor}.enabled` | bool | no | `true` | Whether the sensor is active |
| `clients.{id}.sensors.{sensor}.api_base` | string (URL) | yes (if sensor present) | — | Sensor API base URL |
| `clients.{id}.sensors.{sensor}.credential_ref` | string | yes (if sensor present) | — | Reference to credential in store |
| `clients.{id}.sensors.{sensor}.data_sources` | array of string | no | all available | Data sources to enable for this sensor |
| `clients.{id}.sensors.{sensor}.region` | string | no | — | Sensor-specific config (e.g., CrowdStrike region). Passed to spec engine as extra context. |
| `clients.{id}.capabilities` | table | no | inherits defaults | Capability overrides |

### 2.3 Aliases Configuration — `aliases.toml`

A dedicated file (separate from `prism.toml`) that stores query aliases. Written atomically by `create_alias` and `delete_alias` tools. Loaded at startup alongside main config.

```toml
# Global aliases (available to all clients)
[aliases.critical_alerts]
query = "severity >= critical AND status = open"
description = "All open critical-severity alerts"

[aliases.recent_events]
query = "time_range = last_{{window}}"
description = "Events within a configurable time window"

[aliases.recent_events.parameters]
window = "24h"    # default value; agent can override at invocation

[aliases.active_threats]
query = "@critical_alerts AND time_range = last_24h"
description = "Composed alias referencing critical_alerts"

# Per-client aliases (override global aliases of same name for that client)
[clients.acme.aliases.critical_alerts]
query = "severity >= high AND status = open AND sensor = crowdstrike"
description = "Acme-specific critical alerts (includes high severity, CrowdStrike only)"

[clients.acme.aliases.acme_hosts]
query = "source = hosts AND client_id = acme AND status = {{host_status}}"
description = "Acme host inventory by status"

[clients.acme.aliases.acme_hosts.parameters]
host_status = "active"    # default value
```

**Schema rules:**
- Global aliases are defined under `[aliases.<name>]`
- Per-client aliases are defined under `[clients.<client_id>.aliases.<name>]`
- Parameterized aliases use `[aliases.<name>.parameters]` or `[clients.<client_id>.aliases.<name>.parameters]` sub-tables mapping parameter names to default values (all parameters must have defaults)
- Parameter placeholders in `query` use `{{param_name}}` syntax (double-brace to avoid TOML conflicts)
- Alias names must match `[a-zA-Z_][a-zA-Z0-9_]*` and must not conflict with PrismQL keywords
- Composition references use `@alias_name` prefix in query text; max depth 3, no cycles
- Per-client aliases with the same name as a global alias override the global for that client

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

---

## 5. Changelog

| Version | Date | Burst | Change |
|---------|------|-------|--------|
| 2.2 | 2026-04-19 | Burst 42 Track 2 | Stale `set_credential` reference in `confirm_action` `token_id` description (line 388) replaced with `configure_credential_source`. Closes P3P40-A-MED-001. |
| 2.1 | 2026-04-19 | Deferred Cleanup Track 2 (L-101) | Added 16 missing tool interface definitions (1.34–1.49) for Phase 3-patch tools added in Bursts 33-37: `credential_status`, `crowdstrike_lift_containment`, `get_help`, `get_diagnostics`, `list_plugins`, `plugin_status`, `reload_plugin`, `list_infusions`, `infusion_status`, `reload_infusion`, `list_actions`, `action_status`, `fire_action`, `test_action`, `create_action`, `delete_action`. Drift fix: renamed `set_credential` → `configure_credential_source` to match api-surface.md v1.3 AI-opaque credentials model (reference-based, no raw values). Closes L-101. |
| 2.0 | 2026-04-14 | Phase 1a | Initial interface definitions. |
