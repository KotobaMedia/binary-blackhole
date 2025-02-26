# BinaryBlackhole

![Nothing here, move along.](https://media3.giphy.com/media/v1.Y2lkPTc5MGI3NjExNXRjZjh2ZDJ2NXEyZnMzbWtqNGE1YWw4Z2xqNXdkdmRlajZtcXZ3NCZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/3o84sF21zQYacFcl68/giphy.gif)

# Local Dev

`pnpm dev`

Starts `packages/frontend pnpm dev` and `cargo lambda watch`.
Frontend on [http://localhost:5173](http://localhost:5173) and backend on :9000.

# Info

* TypeScript packages live in `packages/*`. `pnpm` workspaces are utilized. Use `corepack prepare --activate` to get started.
* Rust crates live in `crates/*`. Add new crates with `cargo new crates/<name> --(bin|lib)`.
    * `cargo test` assumes DynamoDB Local is running. Start it with `pnpm dev`
