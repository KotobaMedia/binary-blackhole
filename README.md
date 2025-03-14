# BinaryBlackhole

![Nothing here, move along.](https://media3.giphy.com/media/v1.Y2lkPTc5MGI3NjExNXRjZjh2ZDJ2NXEyZnMzbWtqNGE1YWw4Z2xqNXdkdmRlajZtcXZ3NCZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/3o84sF21zQYacFcl68/giphy.gif)

# Prerequsites

- Install the Node version specified in `mise.toml` (or just use Mise)
- Install [cargo-lambda](https://www.cargo-lambda.info/guide/installation.html)

# Local Dev

`pnpm dev`

Starts `packages/frontend pnpm dev` and `cargo lambda watch`.
Frontend on [http://localhost:5173](http://localhost:5173) and backend on :9000.

# Info

- TypeScript packages live in `packages/*`. `pnpm` workspaces are utilized. Use `corepack prepare --activate` to get started.
- Rust crates live in `crates/*`. Add new crates with `cargo new crates/<name> --(bin|lib)`.
    - `cargo test` assumes DynamoDB Local is running. Start it with `pnpm dev`
- Backend infrastructure is managed by AWS CDK. See [Deploy docs](./packages/cdk/README.md) for more info.

# Useful commands

- `pnpm update -i --latest -r` update all npm packages recursively

# License

If the AGPL is not appropriate for your use case, a commercial license is available by request. [Contact me if you're interested.](mailto:keita@kotobamedia.com)

Copyright © 2025 KotobaMedia

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
[See the GNU Affero General Public License](./LICENSE) for more details.
