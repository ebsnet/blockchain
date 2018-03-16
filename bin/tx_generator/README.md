# Transaction Generator

This module handles the generation of transactions (e.g. handling of keys, signing and mining the
block).

## Security

The keys are stored encrypted on disk. The password is read from the environment variable
`PRIVATE_KEY_PASS` or if it doesn't exist, the user is prompted for a password.
