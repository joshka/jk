#!/usr/bin/env bash
# Source before creating or opening demo repositories; never inherit personal jj settings.
export JJ_CONFIG=/dev/null
export JJ_USER='Demo Author'
export JJ_EMAIL='demo@example.invalid'
export JJ_TIMESTAMP='2026-01-01T12:00:00Z'
export JJ_OP_TIMESTAMP="$JJ_TIMESTAMP"
