---
document_type: prd-supplement
level: L3
section: "interface-definitions"
version: "2.0"
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

#### set_credential

```json
{
  "name": "set_credential",
  "inputSchema": {
    "type": "object",
    "required": ["client_id", "sensor_id", "credential_name", "credential_value"],
    "properties": {
      "client_id": { "type": "string", "pattern": "^[a-zA-Z0-9_-]+$", "description": "Client that owns the credential." },
      "sensor_id": { "type": "string", "pattern": "^[a-z][a-z0-9_-]*$", "description": "Sensor identifier matching a loaded spec file's sensor_id (e.g., crowdstrike, cyberint, claroty, armis, or any config-driven sensor)" },
      "credential_name": { "type": "string", "pattern": "^[a-zA-Z0-9_.\\-]+$", "description": "Credential key name (e.g., 'client_secret', 'api_key')." },
      "credential_value": { "type": "string", "description": "The credential value to store. Never echoed in responses." }
    }
  },
  "outputSchema": {
    "type": "object",
    "properties": {
      "_meta": { "type": "object", "properties": { "trust_level": { "const": "internal" } } },
      "status": { "type": "string", "enum": ["created", "confirmation_required"] },
      "confirmation_token": { "type": ["object", "null"], "properties": { "token_id": { "type": "string" }, "action_summary": { "type": "string" }, "expires_at": { "type": "string", "format": "date-time" } }, "description": "Present when updating an existing credential (confirmation required per BC-2.03.005)." }
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
      "query": { "type": "string", "description": "AxiQL query string to execute on each interval." },
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
      "discovery_query": { "type": ["string", "null"], "default": null, "description": "AxiQL query that must return >= 1 row for the pack to be active for a client (DEC-034)." },
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
      "predicate": { "type": "string", "description": "AxiQL predicate expression that defines the detection condition." },
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
                "condition": { "type": "string", "description": "AxiQL predicate expression for this step." },
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
- Alias names must match `[a-zA-Z_][a-zA-Z0-9_]*` and must not conflict with AxiQL keywords
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
