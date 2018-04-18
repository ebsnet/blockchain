# Cryptography Library

This library provides functions for dealing with key pairs (generating, persisting, loading),
handling secrets (prompting the user for a password or loading it from an environment variable),
signing and verifying signatures

All secrets (key pairs and passwords) are stored in secure memory areas that will not be swapped
and be excluded from core dumps.
