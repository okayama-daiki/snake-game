<br />
<p align="center">
  <a href="https://okayama-daiki.github.io/snake-game/" target="_blank">
    <img src="./public/favicon.ico" height="250" width="250" />
  </a>
</p>

<h3 align="center">Online <b>PvP</b> Snake Game :snake:</h3>

<p align="center">
  <a href="https://open.vscode.dev/okayama-daiki/snake-game">
    <img
      src="https://img.shields.io/static/v1?logo=visualstudiocode&label=&message=Open%20in%20Visual%20Studio%20Code&labelColor=2c2c32&color=007acc&logoColor=007acc"
      alt="Open in Visual Studio Code"
    />
  </a>
  <a href="https://github.com/okayama-daiki/snake-game/actions">
    <img
      src="https://github.com/okayama-daiki/snake-game/actions/workflows/static.yml/badge.svg"
      alt="CI/CD"
    />
  </a>
</p>
<p align="center">
  <img
    src="https://img.shields.io/badge/Rust-black?logo=rust&logoColor=#E57324"
  />
  <img
    src="https://img.shields.io/badge/actix-web?color=%23111
"
  />
  <img
    src="https://img.shields.io/badge/TypeScript-007ACC?logo=typescript&logoColor=white"
  />
  <img
    src="https://img.shields.io/badge/React-20232A?logo=react&logoColor=61DAFB"
  />
</p>

<p align="center">
  <!-- TODO: Add description -->
</p>

## Demo

You can have fun from [this link](https://okayama-daiki.github.io/snake-game/).

## Screenshots

![Title](/screenshots/title.webp)
![Playing](/screenshots/playing.webp)

## Installation

```bash
git clone https://github.com/okayama-daiki/snake-game
```

## Setup

If the backend server is not ready, you can get from [here](https://github.com/okayama-daiki/snake-game-backend/).

```bash
echo VITE_WSS_URI="<URI of backend server>" > .env.local
```

After setting up the server, install the necessary dependencies.

```bash
wasm-pack build --target web
npm install
```

## Tech Stack

### Frontend

- [Vite](https://vitejs.dev) - Build tool that aims to provide a faster and leaner development experience for modern web projects
- [React](https://react.dev) - The library for web and native user interfaces
- [wasm-pack](https://github.com/rustwasm/wasm-pack) - A one-stop shop for building and working with rust- generated WebAssembly
- [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) - Facilitating high-level interactions between Wasm modules and JavaScript

### Backend

- [Actix Web](https://actix.rs) - A powerful, pragmatic, and extremely fast web framework for Rust

## Credit and references

- [slither.io](https://slither.io/) - A smash hit multiplayer online video game
