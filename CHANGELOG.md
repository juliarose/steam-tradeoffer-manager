# Changelog

## 0.6.0 (2025-10-10)

### Added
- `properties` field to `Asset` which contains the `asset_properties` of the asset (inventory responses only).

### Changed
- `TradeOfferManager::get_steamid` now returns `Option<SteamID>` instead of `Result<SteamID>`.
- `TradeOfferManager::get_my_inventory` and `TradeOfferManager::get_inventory` now takes a `tradable_only` parameter to specify whether to include untradable items.
- Polling uses `CancellationToken` rather than aborting the task to allow for more graceful shutdowns. This does not change the public API.
- Bumped `steamid-ng` to `2.0.0`.

### Removed
- `TradeOfferManager::get_inventory_with_untradables`. Use `TradeOfferManager::get_inventory` with `tradable_only` set to `false` instead.

## 0.5.1 (2025-09-15)

### Fixed
- Typo where some requests were passing "acccess_token" instead of "access_token".

## 0.5.0 (2025-08-20)

### Changed
- Bumped `another-steam-totp` to `0.4`.
- `set_cookies` for `TradeOfferManager`, `SteamTradeOfferAPI`, `MobileAPI` now accepts a `Vec<Cookie>` and returns an error if cookies could not be set.

### Added
- `access_token` to `TradeOfferManagerBuilder` and `SteamTradeOfferAPIBuilder`. This can be used for authenticated requests to the Steam API.
- `page_size` to `GetInventoryOptions`.
- `owner_descriptions` to `ClassInfo`.
- `sealed` to `ClassInfo`.
- `api` and `mobile_api` methods to `TradeOfferManager`.

### Changed
- Several error types.

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