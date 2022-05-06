# steam-tradeoffers

Makes steam trade offers easy.

Still a work in progress as I flesh out and test the APIs.

## Features

- Richly-featured API for creating, accepting, cancelling, and declining trade offers.
- Manages account trade offer state.
- Loading inventories.
- Mobile confirmations.
- Automatically cancels offers past a set duration.
- Loads descriptions (classinfos) for assets. Classinfos are cached to file and read when available. The manager holds a [Least frequently used (LFU) cache](https://en.wikipedia.org/wiki/Least_frequently_used) of classinfos in memory to reduce file reads.

## Usage

See [examples](https://github.com/juliarose/steam-tradeoffers/tree/main/examples).

## Thanks

Based on the excellent [node-steam-tradeoffer-manager](https://github.com/DoctorMcKay/node-steam-tradeoffer-manager) module. Thanks to https://github.com/dyc3/steamguard-cli (steamguard) for functionality relating to mobile confirmations.

## LICENSE

MIT