# jsonc-wasm

[![npm version](https://badge.fury.io/js/jsonc-wasm.svg)](https://badge.fury.io/js/jsonc-wasm)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

By using this package, you can acquire JSON strings that can be parsed from JSONC(JSON with comment).

## Install
```shell
$ npm install jsonc-wasm
$ yarn add jsonc-wasm
```

## Usage

```js
import('jsonc-wasm').then(({ toJsonString }) => {
  const res = toJsonString(`{
  // comment line
  /**
   * comment block
   */
  "browsers": {
    "firefox": {
      "name": "Firefox", // FF
      "pref_url": "about:config",
      "releases": {
        "1": {
          "release_date": "2004-11-09",
          "status": "retired",
          "engine": "Gecko",
          "engine_version": "1.7"
        }
      }
    }
  }
}`)
  console.log(JSON.parse(res))
})
```

## Feature

- You can remove comment in JSONC.
- You can use trailing comma.
