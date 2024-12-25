# Changelog

## 0.4.1 (2024-12-25)

### Changed

- Bumped `another-steam-totp` to `0.3.5`.

## 0.4.0 (2024-03-30)

### Changed
- `Error::Http` to `Error::StatusCode`.
- `HOSTNAME` constants to private.
- `ClassInfoCache` to use `Arc<Mutex<T>>` internally rather than requiring it to be wrapped.
- `TradeOfferManagerBuilder` no longer requires a data directory. The data directory now defaults to the user's config directory.
- `TradeOfferManagerBuilder` no longer requires an API key.
- `ParameterError::CannotAcceptOfferThatIsOurs` into `ParameterError::CannotAcceptOfferWeCreated` for consistency.
- `TradeOfferManagerBuilder`, `SteamTradeOfferAPIBuilder`, `MobileAPIBuilder`, and `NewTradeOfferBuilder` fields are now private.
- `PollResult` to `Result`.
- Moved `ServerTime` to `types::ServerTime`.
- Reduced contention on `ClassInfoCache` by moving inserts to `get_asset_classinfos` from `get_app_asset_classinfos_chunk`.
- Moved `save_classinfos` to a `tokio` task so that classinfo data can be returned without waiting for files to be written.
- Poll data now trims to only offers returned in a full update.
- `Error::MalformedResponse` now contains message with error details.
- Re-exported all inner values for error variants.

### Added
- Some missing derives for various structs.
- `PollAction::StopPolling`.

## 0.3.0 (2023-10-08)

### Changed
- `chrono` version to `^0.4.27` to avoid a potential error when compiling with older versions.
- Exposed `mobile_api`.

## 0.2.0 (2023-06-26)

### Changed
- Mobile confirmations to use the new Steam endpoints.
- `TradeOfferManager#start_polling` was modified in favor of using sender/receiver-style messaging.
- `Error::ConfirmationUnsuccessful` now holds an optional message string.
- `ConfirmationType::Unknown` now holds a u32 value holding the code for the unknown confirmation type.

### Removed
- `TradeOfferManager#do_poll` in favor of utilizing senders.

## 0.1.0 (2023-05-16)