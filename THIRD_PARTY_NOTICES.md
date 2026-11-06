# Third-Party Notices

Titan HFT is released under the [MIT License](LICENSE.txt). It bundles the following third-party components, which remain under their own licenses.

## TradingView Lightweight Charts™ v4.0.1
- File: `src/lightweight-charts.js`
- License: Apache License 2.0 — full text in [LICENSES/Apache-2.0.txt](LICENSES/Apache-2.0.txt)
- NOTICE ([LICENSES/NOTICE-lightweight-charts.txt](LICENSES/NOTICE-lightweight-charts.txt)):

```
TradingView Lightweight Charts™
Copyright (с) 2023 TradingView, Inc. https://www.tradingview.com/
```

Charts in Titan HFT are powered by [TradingView Lightweight Charts](https://www.tradingview.com/lightweight-charts/).

## Share Tech Mono font
- File: `src/fonts/ShareTechMono-Regular.woff2`
- License: SIL Open Font License 1.1 — full text in [LICENSES/OFL-ShareTechMono.txt](LICENSES/OFL-ShareTechMono.txt)
- Copyright (c) 2012, Carrois Type Design, Ralph du Carrois, with Reserved Font Name 'Share'

## Rust crates
Rust dependencies are downloaded from crates.io at build time. They are **not** included in the source package, and each one stays under its own license (mostly MIT and/or Apache-2.0). See `src-tauri/Cargo.lock` and `src-tauri/deny.toml`.

> If you distribute **compiled binaries**, include the license notices of the linked crates. One way is to generate them with [`cargo-about`](https://github.com/EmbarkStudios/cargo-about): `cargo about generate about.hbs > THIRD_PARTY_CRATES.html`.

## Trademarks
Alpaca is a trademark of AlpacaDB, Inc. TradingView and Lightweight Charts are trademarks of TradingView, Inc. All other trademarks belong to their respective owners. Titan HFT is an independent project. It is **not affiliated with, endorsed by or sponsored by** any of these companies.
