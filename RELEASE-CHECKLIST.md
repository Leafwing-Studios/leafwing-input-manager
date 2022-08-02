# LWIM Release Checklist

## Adding a new input kind

1. Ensure that `reset_inputs` for `MutableInputStreams` is resetting all relevant fields.
2. Ensure that `RawInputs` struct has fields that cover all necessary input types.
3. Ensure that `send_input` and `release_input` check all possible fields on `RawInputs`.

## Before release

1. Ensure no tests (other than ones in the README) are ignored.
2. Manually verify that all examples work.
