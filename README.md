# 🔌 CFPROXY - The curseforge proxy server

![Issues](https://img.shields.io/github/issues/bmpm-mc/cfproxy)
![Last Commit](https://img.shields.io/github/last-commit/bmpm-mc/cfproxy)
![License](https://img.shields.io/github/license/bmpm-mc/cfproxy)

Curseforge has locked down their API and now restricts access without authentification. This spells trouble for application developers using the API  - They aren't allowed to ship their API Key to their users, and must resort to alternate methods. This is one of them.

## What does it do?

> CFPROXY is a HTTP server. Every request gets proxied to the Curseforge API with an API key attached - that way, developers for client side apps can use this as the endpoint for their CF requests, and the CF API key stays safe, only being used on the server.

Every request can be made as seen in [the Curseforge API docs](https://docs.curseforge.com/#getting-started) - you only have to switch out the base URL (`https://api.curseforge.com`) for the proxy's base url.

## How do I use it?

Two methods: Either use the "official" cfproxy, or run your own (it's open source, after all!)

- **If you want to use the "official" cfproxy**, use `https://cfproxy.fly.dev` as the base url - there's no authentication involved, but to prevent API abuse requests get rate limited heavily.
- **If you want to run your own proxy**, check out the [Building from source](#building-from-source) chapter below.

All requests along with their headers, body, path, and params should be forwarded to CF, if you notice something odd or think something doesn't get proxied properly, please open an issue.

## Building from source

If you want to run the server yourself, everything you need is an installation of [Rust](https://www.rust-lang.org/) and your Curseforge API Key (if you don't have one, [apply here](https://forms.monday.com/forms/dce5ccb7afda9a1c21dab1a1aa1d84eb?r=use1) - this will take a while).

- Clone the repository.
- Put your API key into an environment variable named `CF_API_KEY` (You can also put `CF_API_KEY = '..'` into an `.env` file. Don't forget the single quotes!)
- Run the server with `cargo run`
- You should see a message popping up: `Server starting at port 3000`. Success! You can now make requests to your server.

Additional options are configured through environment variables:

| Key | Value type | Meaning |
| --- | ---------- | ------- |
| `CF_API_KEY` | string | Your API key you got from Curseforge.
| `PORT` | number | The port at which to start up the server. Optional - defaults to `3000`.
| `REQ_LIMIT_PER_SEC` | number | How many requests per second per IP address are allowed. Optional - defaults to `6`.