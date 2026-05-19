# Inspect Workflow

## Goal

Inspect history, change content, file content, and current repository state without leaving `jk`.

## Primary Surfaces

- log
- show
- diff
- status
- file list/show/search/annotate
- bookmark list
- tag list
- workspace root

## Primary Loop

1. start in log
1. select a change
1. open show or diff
1. inspect file-level detail
1. go back
1. inspect related utility state if needed

## UI Bias

- read surfaces first
- inline detail and dedicated drill-down screens before fixed splits
- low chrome

## Acceptance Criteria

- shell ping-pong is reduced materially
- context is preserved across drill-down and backtracking
