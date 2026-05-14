# v-contractx WASM runtime

This directory contains the portable contract execution target.

The exported function accepts a JSON payload, evaluates whether the projection input is structurally acceptable, and returns a JSON response. The host runtime can embed this module in browser, node, or edge environments as a portable contract evaluator.
