# iat

This module allows one to search through a [`Module`](../modules/objects-module.md)'s IAT (import address table) and hook the entries.

## How?

The module's internal structures to find its import addresss table, then we extract the needed information from it. After that, all that needs to be done is write a new ptr to the iat's entry for that specific function.
