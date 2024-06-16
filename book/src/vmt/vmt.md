# vtable

This module contains helpers to hook c++ virtual method tables.

## How?

We find the correct index offset in the vtable's array of pointers and write a new ptr to that location.
