# scan

This module contains functions for searching through memory for pattern masks.

```admonish info title="Scanning uses SIMD"
If the computer has AVX2, this is used. If not, but it has SSE4.2, then this is used. If it has none of those, then a regular scalar search is used.

AVX2 and SSE4.2 can reach search speeds of about 1.8gb/s, and scalar search can reach about 1gb/s.

```
