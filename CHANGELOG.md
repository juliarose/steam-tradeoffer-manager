# Changelog

## 0.1.0 (2023-05-16)

## 0.2.0 (2023-06-26)

### Changed
- Mobile confirmations to use the new method.
- `start_polling` now returns the sender.
- `Error::ConfirmationUnsuccessful` now holds an optional message string.
- `ConfirmationType::Unknown` now olds a u32 value holding the code for the unknown confirmation type.

### Removed
- `do_poll` method from `TradeOfferManager` in favor of utilizing senders.