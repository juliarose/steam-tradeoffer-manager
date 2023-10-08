# Changelog

## 0.1.0 (2023-05-16)

## 0.2.0 (2023-06-26)

### Changed
- Mobile confirmations to use the new Steam endpoints.
- `TradeOfferManager#start_polling` was modified in favor of using sender/receiver-style messaging.
- `Error::ConfirmationUnsuccessful` now holds an optional message string.
- `ConfirmationType::Unknown` now olds a u32 value holding the code for the unknown confirmation type.

### Removed
- `TradeOfferManager#do_poll` in favor of utilizing senders.

## 0.3.0 (2023-10-08)

### Changed
- `chrono` version to `^0.4.27` to avoid a potential error when compiling with older versions.
- Exposed `mobile_api`.